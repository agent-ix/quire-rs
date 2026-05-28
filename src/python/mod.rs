//! PyO3 bindings (FR-023) — feature-gated behind `python`.
//!
//! Exposes the engine to Python as the `quire` module: parse a single
//! document, walk a repo, and query a `Spec` corpus. Results are
//! structured Python objects (dicts / lists / the `Spec` class), never
//! JSON strings or subprocess output (StR-005-AC-4). Heavy Rust work
//! releases the GIL (NFR-016).
//!
//! First-party code here contains **no** `unsafe` — PyO3's macro-
//! generated unsafe is upstream (NFR-003-AC-4).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use pyo3::exceptions::{PyException, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyList};
use serde_json::Value;

use crate::ast::{QuireDocument, QuireSection};
use crate::corpus::resolve::Edge;
use crate::corpus::walk::LoadedDocument;
use crate::error::QuireError;
use crate::loader::compile::CompiledArchetype;

pyo3::create_exception!(quire, QuireBaseError, PyException);
pyo3::create_exception!(quire, QuireRenderError, QuireBaseError);
pyo3::create_exception!(quire, QuireValidationError, QuireBaseError);
pyo3::create_exception!(quire, QuireSchemaError, QuireBaseError);
pyo3::create_exception!(quire, QuireParseError, QuireBaseError);

/// The `quire` Python module.
#[pymodule]
fn quire(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("QuireBaseError", py.get_type::<QuireBaseError>())?;
    m.add("QuireRenderError", py.get_type::<QuireRenderError>())?;
    m.add(
        "QuireValidationError",
        py.get_type::<QuireValidationError>(),
    )?;
    m.add("QuireSchemaError", py.get_type::<QuireSchemaError>())?;
    m.add("QuireParseError", py.get_type::<QuireParseError>())?;
    m.add_function(wrap_pyfunction!(parse_document, m)?)?;
    m.add_function(wrap_pyfunction!(load_repo, m)?)?;
    m.add_function(wrap_pyfunction!(render, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_function(wrap_pyfunction!(validate_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(extract, m)?)?;
    m.add_function(wrap_pyfunction!(extract_frontmatter, m)?)?;
    m.add_function(wrap_pyfunction!(harvest_edges, m)?)?;
    m.add_class::<Spec>()?;
    m.add_class::<Registry>()?;
    m.add_class::<ExtractionContext>()?;
    Ok(())
}

/// Render `archetype_name` from `module_root` against `data`. Returns
/// the rendered markdown string. Raises `QuireSchemaError` if the
/// module / archetype fails to load, `QuireRenderError` on template
/// failure.
#[pyfunction]
fn render(
    py: Python<'_>,
    archetype_name: &str,
    module_root: &str,
    data: &Bound<'_, PyAny>,
) -> PyResult<String> {
    let value = py_to_json(data)?;
    let module = module_root.to_string();
    let name = archetype_name.to_string();
    py.detach(|| -> PyResult<String> {
        let registry = crate::Registry::load_module(Path::new(&module))
            .map_err(quire_error_to_schema_pyerr)?;
        let out =
            crate::render_by_name(&registry, &name, &value).map_err(quire_error_to_render_pyerr)?;
        Ok(out.into_markdown())
    })
}

/// Validate `data` against `archetype_name` from `module_root`.
/// Raises `QuireValidationError` on schema violation (carrying field
/// path), `QuireSchemaError` if the module / archetype fails to load.
#[pyfunction]
fn validate(
    py: Python<'_>,
    archetype_name: &str,
    module_root: &str,
    data: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let value = py_to_json(data)?;
    let module = module_root.to_string();
    let name = archetype_name.to_string();
    py.detach(|| -> PyResult<()> {
        let registry = crate::Registry::load_module(Path::new(&module))
            .map_err(quire_error_to_schema_pyerr)?;
        let arch = registry
            .archetype(&name)
            .ok_or_else(|| QuireSchemaError::new_err(format!("unknown archetype: {name}")))?;
        crate::validate(arch, &value).map_err(quire_error_to_validation_pyerr)
    })
}

/// Validate an arbitrary `payload` (dict) against the JSON Schema at
/// `schema_path`. Uses the same `jsonschema` validator the engine
/// uses internally. Returns a list of structured violations
/// (`[]` means valid); `QuireSchemaError` is raised if the schema file
/// fails to load / compile.
#[pyfunction]
fn validate_manifest<'py>(
    py: Python<'py>,
    payload: &Bound<'_, PyAny>,
    schema_path: &str,
) -> PyResult<Bound<'py, PyList>> {
    let value = py_to_json(payload)?;
    let path = schema_path.to_string();
    let violations = py.detach(
        || -> PyResult<Vec<crate::validate::SchemaValidationDetail>> {
            let schema = crate::loader::compile::read_schema(Path::new(&path)).map_err(|m| {
                QuireSchemaError::new_err(format!("schema read failed at {path}: {m}"))
            })?;
            let validator = crate::loader::compile::compile_schema(&schema)
                .map_err(|m| QuireSchemaError::new_err(format!("schema compile failed: {m}")))?;
            if let Err(mut errors) = validator.validate(&value) {
                let details = errors
                    .by_ref()
                    .map(|e| crate::validate::schema_validation_detail(&e))
                    .collect();
                return Ok(details);
            }
            Ok(Vec::new())
        },
    )?;

    let out = PyList::empty(py);
    for detail in violations {
        let d = PyDict::new(py);
        d.set_item("path", detail.path)?;
        d.set_item("message", detail.message)?;
        d.set_item("schema_keyword", detail.schema_keyword)?;
        out.append(d)?;
    }
    Ok(out)
}

/// Parse `document_text`, evaluate `archetype_name`'s `body_extraction`
/// DSL, and return `{extraction: [...], edges: [{target, edge_type}, ...]}`.
/// Raises `QuireSchemaError` on module / archetype load failure,
/// `QuireParseError` if the archetype has no `body_extraction` DSL.
#[pyfunction]
fn extract<'py>(
    py: Python<'py>,
    archetype_name: &str,
    module_root: &str,
    document_text: &str,
) -> PyResult<Bound<'py, PyDict>> {
    let module = module_root.to_string();
    let name = archetype_name.to_string();
    let text = document_text.to_string();

    struct ExtractOut {
        records: Vec<serde_json::Map<String, Value>>,
        edges: Vec<(String, String)>,
    }

    let outcome: PyResult<ExtractOut> = py.detach(|| {
        let registry = crate::Registry::load_module(Path::new(&module))
            .map_err(quire_error_to_schema_pyerr)?;
        let arch = registry
            .archetype(&name)
            .ok_or_else(|| QuireSchemaError::new_err(format!("unknown archetype: {name}")))?;
        let dsl = arch.body_extraction().ok_or_else(|| {
            QuireParseError::new_err(format!("archetype {name} has no body_extraction DSL"))
        })?;
        let doc = crate::parse_document(&text);
        let result = crate::extract(&doc, dsl).map_err(quire_error_to_parse_pyerr)?;

        // Harvest edges off a LoadedDocument view.
        let loaded = LoadedDocument {
            path: std::path::PathBuf::new(),
            id: doc
                .frontmatter
                .as_ref()
                .and_then(|fm| fm.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            uuid: None,
            doc,
        };
        let edges = crate::harvest_edges(&loaded);
        Ok(ExtractOut {
            records: result.records,
            edges,
        })
    });
    let outcome = outcome?;

    let out = PyDict::new(py);
    let records = PyList::empty(py);
    for r in &outcome.records {
        records.append(json_to_py(py, &Value::Object(r.clone()))?)?;
    }
    out.set_item("extraction", records)?;
    let edge_list = PyList::empty(py);
    for (target, edge_type) in &outcome.edges {
        let d = PyDict::new(py);
        d.set_item("target", target)?;
        d.set_item("edge_type", edge_type)?;
        edge_list.append(d)?;
    }
    out.set_item("edges", edge_list)?;
    Ok(out)
}

/// Extract frontmatter and body from `text` using the Rust FR-006 parser.
/// Returns `{frontmatter: dict | None, body: str}`.
#[pyfunction]
fn extract_frontmatter<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyDict>> {
    let result = crate::extract_frontmatter(text);
    let out = PyDict::new(py);
    match result.frontmatter.as_ref() {
        None => out.set_item("frontmatter", py.None())?,
        Some(fm) => {
            let frontmatter = PyDict::new(py);
            for (k, v) in fm {
                frontmatter.set_item(k, json_to_py(py, v)?)?;
            }
            out.set_item("frontmatter", frontmatter)?;
        }
    }
    out.set_item("body", result.body)?;
    Ok(out)
}

/// Harvest `ix://` + frontmatter `relationships` edges off `doc` —
/// either a parsed document dict (from `parse_document`) or raw
/// markdown text. Returns `[{target, edge_type}, ...]` sorted.
#[pyfunction]
fn harvest_edges<'py>(py: Python<'py>, doc: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyList>> {
    // Accept either a string (parse it) or a dict (we re-extract raw
    // by serializing through parse_document). For dict input we read
    // `frontmatter`/`raw` if present; otherwise fall back to text.
    let loaded = if let Ok(s) = doc.extract::<String>() {
        let parsed = crate::parse_document(&s);
        LoadedDocument {
            path: std::path::PathBuf::new(),
            id: String::new(),
            uuid: None,
            doc: parsed,
        }
    } else if let Ok(d) = doc.cast::<PyDict>() {
        // Reconstruct a minimal QuireDocument view from the dict.
        let raw = d
            .get_item("raw")?
            .and_then(|v| v.extract::<String>().ok())
            .unwrap_or_default();
        let value = py_to_json(d.as_any())?;
        // Re-parse from raw if available (canonical); else build a
        // doc with frontmatter-only edges from the dict.
        if !raw.is_empty() {
            let parsed = crate::parse_document(&raw);
            LoadedDocument {
                path: std::path::PathBuf::new(),
                id: String::new(),
                uuid: None,
                doc: parsed,
            }
        } else {
            // Build a QuireDocument with frontmatter only.
            let mut fm: serde_json::Map<String, Value> = serde_json::Map::new();
            if let Value::Object(map) = &value {
                if let Some(Value::Object(fmap)) = map.get("frontmatter") {
                    for (k, v) in fmap {
                        fm.insert(k.clone(), v.clone());
                    }
                }
            }
            let qdoc = QuireDocument {
                preamble: None,
                sections: Vec::new(),
                raw: String::new(),
                frontmatter: if fm.is_empty() { None } else { Some(fm) },
            };
            LoadedDocument {
                path: std::path::PathBuf::new(),
                id: String::new(),
                uuid: None,
                doc: qdoc,
            }
        }
    } else {
        return Err(PyTypeError::new_err(
            "harvest_edges expects a str (markdown) or dict (parsed doc)",
        ));
    };

    let edges = crate::harvest_edges(&loaded);
    let out = PyList::empty(py);
    for (target, edge_type) in &edges {
        let d = PyDict::new(py);
        d.set_item("target", target)?;
        d.set_item("edge_type", edge_type)?;
        out.append(d)?;
    }
    Ok(out)
}

/// Parse one markdown document into a structured dict
/// (`{frontmatter, preamble, sections}`).
#[pyfunction]
fn parse_document<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyDict>> {
    let doc = crate::parse_document(text);
    document_to_py(py, &doc)
}

/// Walk `root`, parse every markdown file, and return a list of
/// `{path, id, uuid, doc}` dicts. The Rust walk runs with the GIL
/// released (NFR-016).
#[pyfunction]
fn load_repo<'py>(py: Python<'py>, root: &str) -> PyResult<Bound<'py, PyList>> {
    let load = py.detach(|| crate::load_repo(Path::new(root)));
    let list = PyList::empty(py);
    for d in &load.documents {
        let item = PyDict::new(py);
        item.set_item("path", d.path.to_string_lossy().into_owned())?;
        item.set_item("id", &d.id)?;
        item.set_item("uuid", d.uuid.map(|u| u.to_string()))?;
        item.set_item("doc", document_to_py(py, &d.doc)?)?;
        list.append(item)?;
    }
    Ok(list)
}

/// A loaded, resolved spec corpus (FR-025/026/027).
#[pyclass(name = "Spec")]
struct Spec {
    inner: crate::Spec,
}

#[pymethods]
impl Spec {
    /// Load + resolve a `spec/` tree. GIL released during the Rust work.
    #[staticmethod]
    fn from_path(py: Python<'_>, root: &str) -> Self {
        let inner = py.detach(|| crate::Spec::from_path(Path::new(root)));
        Spec { inner }
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Artifact ids of every document of `artifact_type`, sorted.
    fn by_type(&self, ty: &str) -> Vec<String> {
        self.inner
            .by_type(ty)
            .iter()
            .map(|d| d.id.clone())
            .collect()
    }

    /// `{id, uuid, path, type}` for one artifact, or `None`.
    fn by_id<'py>(&self, py: Python<'py>, id: &str) -> PyResult<Option<Bound<'py, PyDict>>> {
        match self.inner.by_id(id) {
            None => Ok(None),
            Some(d) => {
                let item = PyDict::new(py);
                item.set_item("id", &d.id)?;
                item.set_item("uuid", d.uuid.map(|u| u.to_string()))?;
                item.set_item("path", d.path.to_string_lossy().into_owned())?;
                item.set_item(
                    "type",
                    d.doc
                        .frontmatter
                        .as_ref()
                        .and_then(|fm| fm.get("artifact_type").or_else(|| fm.get("type")))
                        .and_then(|v| v.as_str()),
                )?;
                Ok(Some(item))
            }
        }
    }

    /// Resolved edges referencing `id` (reverse lookup), as
    /// `(source, edge_type)` tuples.
    fn referencing(&self, id: &str) -> Vec<(String, String)> {
        self.inner
            .referencing(id)
            .iter()
            .map(|e| (e.source.clone(), e.edge_type.clone()))
            .collect()
    }

    /// Ids of artifacts of `of_type` lacking a resolved
    /// `missing_edge_type` edge (optionally toward `toward_type`).
    #[pyo3(signature = (of_type, missing_edge_type, toward_type=None))]
    fn orphans(
        &self,
        of_type: &str,
        missing_edge_type: &str,
        toward_type: Option<&str>,
    ) -> Vec<String> {
        self.inner
            .orphans(of_type, missing_edge_type, toward_type)
            .iter()
            .map(|d| d.id.clone())
            .collect()
    }

    /// Dangling edges as `(source, target, edge_type)` tuples.
    fn dangling(&self) -> Vec<(String, String, String)> {
        self.inner.dangling().iter().map(edge_tuple).collect()
    }

    /// Human-readable load + resolution diagnostics.
    fn diagnostics(&self) -> Vec<String> {
        self.inner
            .diagnostics()
            .iter()
            .map(|d| d.to_string())
            .collect()
    }
}

fn edge_tuple(e: &&Edge) -> (String, String, String) {
    (e.source.clone(), e.target.clone(), e.edge_type.clone())
}

/// A loaded archetype registry (FR-013) — schema validation surface
/// for Python (FR-023).
#[pyclass(name = "Registry")]
struct Registry {
    inner: crate::Registry,
}

#[pymethods]
impl Registry {
    /// Load archetypes from one or more search paths.
    #[staticmethod]
    fn load_from(py: Python<'_>, paths: Vec<String>) -> PyResult<Self> {
        let inner = py
            .detach(|| {
                let refs: Vec<&Path> = paths.iter().map(Path::new).collect();
                crate::Registry::load_from(&refs)
            })
            .map_err(quire_error_to_pyerr)?;
        Ok(Registry { inner })
    }

    /// Load from `IX_SCHEMA_PATH` / `~/.ix/schemas/`.
    #[staticmethod]
    fn from_env(py: Python<'_>) -> PyResult<Self> {
        let inner = py
            .detach(crate::Registry::from_env)
            .map_err(quire_error_to_pyerr)?;
        Ok(Registry { inner })
    }

    /// Names of all loaded archetypes, sorted.
    fn archetype_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.inner.archetype_names().map(str::to_string).collect();
        names.sort();
        names
    }

    /// Validate `data` against `archetype`'s schema. Returns the list of
    /// violations (empty = valid); each is a dict with `archetype`,
    /// `field_path`, `expected`, `observed` (NFR-005). Raises if the
    /// archetype is unknown.
    fn validate<'py>(
        &self,
        py: Python<'py>,
        archetype: &str,
        data: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyList>> {
        let arch = self
            .inner
            .archetype(archetype)
            .ok_or_else(|| PyValueError::new_err(format!("unknown archetype: {archetype}")))?;
        let value = py_to_json(data)?;

        let violations = PyList::empty(py);
        if let Err(errors) = crate::validate_all(arch, &value) {
            for e in &errors {
                violations.append(violation_to_py(py, e)?)?;
            }
        }
        Ok(violations)
    }
}

fn violation_to_py<'py>(py: Python<'py>, e: &QuireError) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    match e {
        QuireError::SchemaViolation {
            archetype,
            field_path,
            expected,
            observed,
        } => {
            d.set_item("archetype", archetype)?;
            d.set_item("field_path", field_path)?;
            d.set_item("expected", expected)?;
            d.set_item("observed", observed)?;
        }
        other => {
            d.set_item("error", other.to_string())?;
        }
    }
    Ok(d)
}

/// Pure compiled extraction state built from caller-provided ObjectType
/// definitions. This is intentionally not a source registry: it does
/// not read `~/.ix`, module paths, environment variables, or HTTP.
#[pyclass(name = "ExtractionContext")]
struct ExtractionContext {
    objects: BTreeMap<String, Arc<CompiledArchetype>>,
}

#[pymethods]
impl ExtractionContext {
    /// Compile ObjectType-shaped dicts into an in-memory extraction
    /// context. Accepts either `[{...}]` or `{"items": [{...}]}`.
    #[staticmethod]
    fn from_object_types(py: Python<'_>, object_types: &Bound<'_, PyAny>) -> PyResult<Self> {
        let value = py_to_json(object_types)?;
        let objects = py.detach(|| compile_object_types(value))?;
        Ok(Self { objects })
    }

    fn object_type_names(&self) -> Vec<String> {
        self.objects.keys().cloned().collect()
    }

    fn validate<'py>(
        &self,
        py: Python<'py>,
        object_type: &str,
        data: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyList>> {
        let arch = self
            .objects
            .get(object_type)
            .ok_or_else(|| PyValueError::new_err(format!("unknown object type: {object_type}")))?;
        let value = py_to_json(data)?;

        let violations = PyList::empty(py);
        if let Err(errors) = crate::validate_all(arch, &value) {
            for e in &errors {
                violations.append(violation_to_py(py, e)?)?;
            }
        }
        Ok(violations)
    }

    /// Extract records for `object_type` from caller-provided document
    /// parts. `frontmatter` is the parsed YAML object supplied by the
    /// host workflow; `body` is the markdown body text.
    fn extract<'py>(
        &self,
        py: Python<'py>,
        object_type: &str,
        frontmatter: &Bound<'py, PyAny>,
        body: &str,
    ) -> PyResult<Bound<'py, PyDict>> {
        let arch = self
            .objects
            .get(object_type)
            .ok_or_else(|| PyValueError::new_err(format!("unknown object type: {object_type}")))?;
        let dsl = arch.body_extraction().ok_or_else(|| {
            QuireParseError::new_err(format!(
                "object type {object_type} has no body_extraction DSL"
            ))
        })?;
        let fm = py_to_json(frontmatter)?;
        let body = body.to_string();
        let arch = Arc::clone(arch);
        let dsl = dsl.clone();

        let outcome = py.detach(|| -> PyResult<crate::extract::ExtractionResult> {
            let mut doc = crate::parse_document(&body);
            doc.frontmatter = match fm {
                Value::Object(map) if !map.is_empty() => Some(map),
                _ => None,
            };
            let result = crate::extract(&doc, &dsl).map_err(quire_error_to_parse_pyerr)?;
            for record in &result.records {
                let value = Value::Object(record.clone());
                crate::validate(&arch, &value).map_err(quire_error_to_validation_pyerr)?;
            }
            Ok(result)
        })?;

        let out = PyDict::new(py);
        let records = PyList::empty(py);
        for r in &outcome.records {
            records.append(json_to_py(py, &Value::Object(r.clone()))?)?;
        }
        out.set_item("extraction", records)?;
        let edges = PyList::empty(py);
        for edge in &outcome.edges {
            let item = PyDict::new(py);
            item.set_item("record_index", edge.record_index)?;
            item.set_item("type", &edge.edge_type)?;
            item.set_item("target", &edge.target)?;
            edges.append(item)?;
        }
        out.set_item("edges", edges)?;
        let diagnostics = PyList::empty(py);
        for d in &outcome.diagnostics {
            diagnostics.append(d.to_string())?;
        }
        out.set_item("diagnostics", diagnostics)?;
        Ok(out)
    }
}

fn compile_object_types(value: Value) -> PyResult<BTreeMap<String, Arc<CompiledArchetype>>> {
    let items = match value {
        Value::Array(items) => items,
        Value::Object(mut map) => match map.remove("items") {
            Some(Value::Array(items)) => items,
            _ => {
                return Err(PyTypeError::new_err(
                    "object_types must be a list or {'items': list}",
                ))
            }
        },
        _ => {
            return Err(PyTypeError::new_err(
                "object_types must be a list or {'items': list}",
            ))
        }
    };
    let mut out = BTreeMap::new();
    for item in items {
        let Value::Object(mut map) = item else {
            return Err(PyTypeError::new_err("object type entries must be dicts"));
        };
        let name = map
            .remove("name")
            .and_then(|v| v.as_str().map(str::to_string))
            .ok_or_else(|| PyValueError::new_err("object type entry missing string name"))?;
        if out.contains_key(&name) {
            return Err(PyValueError::new_err(format!(
                "duplicate object type name: {name}"
            )));
        }
        let schema = map
            .remove("schema")
            .or_else(|| map.remove("data_schema"))
            .unwrap_or_else(|| serde_json::json!({"type": "object"}));
        let validator = crate::loader::compile::compile_schema(&schema).map_err(|m| {
            QuireSchemaError::new_err(format!("{name}: schema compile failed: {m}"))
        })?;
        let body_extraction = match map.remove("body_extraction") {
            Some(Value::Null) | None => None,
            Some(value) => {
                let dsl: crate::extract::dsl::ExtractionDsl = serde_json::from_value(value)
                    .map_err(|e| {
                        QuireParseError::new_err(format!(
                            "{name}: body_extraction parse failed: {e}"
                        ))
                    })?;
                crate::extract::dsl::validate_dsl(&name, &dsl)
                    .map_err(quire_error_to_parse_pyerr)?;
                Some(dsl)
            }
        };
        out.insert(
            name.clone(),
            Arc::new(CompiledArchetype {
                name,
                module: "provided".to_string(),
                raw_schema: Arc::new(schema),
                validator: Arc::new(validator),
                template_path: None,
                template_name: None,
                body_extraction,
            }),
        );
    }
    Ok(out)
}

fn quire_error_to_pyerr(e: QuireError) -> PyErr {
    // Route by variant; default to the base exception.
    match &e {
        QuireError::SchemaViolation { .. } | QuireError::MissingField { .. } => {
            QuireValidationError::new_err(e.to_string())
        }
        QuireError::UnknownArchetype { .. }
        | QuireError::ArchetypeCollision { .. }
        | QuireError::ModuleCollision { .. }
        | QuireError::ArchetypeLoadError { .. }
        | QuireError::ManifestError { .. }
        | QuireError::InvalidSearchPath { .. } => QuireSchemaError::new_err(e.to_string()),
        QuireError::TemplateError { .. } => QuireRenderError::new_err(e.to_string()),
        QuireError::DslValidationError { .. } => QuireParseError::new_err(e.to_string()),
    }
}

fn quire_error_to_validation_pyerr(e: QuireError) -> PyErr {
    QuireValidationError::new_err(e.to_string())
}

fn quire_error_to_render_pyerr(e: QuireError) -> PyErr {
    match &e {
        QuireError::TemplateError { .. } => QuireRenderError::new_err(e.to_string()),
        _ => quire_error_to_pyerr(e),
    }
}

fn quire_error_to_schema_pyerr(e: QuireError) -> PyErr {
    QuireSchemaError::new_err(e.to_string())
}

fn quire_error_to_parse_pyerr(e: QuireError) -> PyErr {
    QuireParseError::new_err(e.to_string())
}

/// Convert a native Python object to a `serde_json::Value` (no JSON
/// string round-trip). Bool is checked before int (Python `bool`
/// subclasses `int`).
fn py_to_json(obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if obj.is_none() {
        return Ok(Value::Null);
    }
    if let Ok(b) = obj.cast::<PyBool>() {
        return Ok(Value::Bool(b.is_true()));
    }
    if let Ok(i) = obj.extract::<i64>() {
        return Ok(Value::Number(i.into()));
    }
    if let Ok(f) = obj.extract::<f64>() {
        return Ok(serde_json::Number::from_f64(f).map_or(Value::Null, Value::Number));
    }
    if let Ok(s) = obj.extract::<String>() {
        return Ok(Value::String(s));
    }
    if let Ok(list) = obj.cast::<PyList>() {
        let mut arr = Vec::with_capacity(list.len());
        for item in list.iter() {
            arr.push(py_to_json(&item)?);
        }
        return Ok(Value::Array(arr));
    }
    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            map.insert(k.extract::<String>()?, py_to_json(&v)?);
        }
        return Ok(Value::Object(map));
    }
    Err(PyTypeError::new_err(
        "unsupported type for JSON conversion (expected None/bool/int/float/str/list/dict)",
    ))
}

fn document_to_py<'py>(py: Python<'py>, doc: &QuireDocument) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("preamble", doc.preamble.as_deref())?;
    match doc.frontmatter.as_ref() {
        Some(fm) => {
            let d = PyDict::new(py);
            for (k, v) in fm {
                d.set_item(k, json_to_py(py, v)?)?;
            }
            out.set_item("frontmatter", d)?;
        }
        None => out.set_item("frontmatter", py.None())?,
    }
    let sections = PyList::empty(py);
    for s in &doc.sections {
        sections.append(section_to_py(py, s)?)?;
    }
    out.set_item("sections", sections)?;
    Ok(out)
}

fn section_to_py<'py>(py: Python<'py>, s: &QuireSection) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("heading", &s.heading)?;
    d.set_item("level", s.level)?;
    d.set_item("block_id", s.block_id.as_deref())?;
    d.set_item("content", &s.content)?;
    let children = PyList::empty(py);
    for c in &s.children {
        children.append(section_to_py(py, c)?)?;
    }
    d.set_item("children", children)?;
    Ok(d)
}

/// Convert a `serde_json::Value` to a native Python object (no JSON
/// string round-trip).
fn json_to_py<'py>(py: Python<'py>, v: &Value) -> PyResult<Bound<'py, PyAny>> {
    let obj = match v {
        Value::Null => py.None().into_bound(py),
        Value::Bool(b) => pyo3::types::PyBool::new(py, *b).to_owned().into_any(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_pyobject(py)?.into_any()
            } else if let Some(u) = n.as_u64() {
                u.into_pyobject(py)?.into_any()
            } else {
                n.as_f64().unwrap_or(f64::NAN).into_pyobject(py)?.into_any()
            }
        }
        Value::String(s) => s.into_pyobject(py)?.into_any(),
        Value::Array(a) => {
            let list = PyList::empty(py);
            for item in a {
                list.append(json_to_py(py, item)?)?;
            }
            list.into_any()
        }
        Value::Object(o) => {
            let d = PyDict::new(py);
            for (k, val) in o {
                d.set_item(k, json_to_py(py, val)?)?;
            }
            d.into_any()
        }
    };
    Ok(obj)
}
