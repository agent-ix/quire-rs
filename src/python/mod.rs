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

use std::path::Path;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyList};
use serde_json::Value;

use crate::ast::{QuireDocument, QuireSection};
use crate::corpus::resolve::Edge;
use crate::error::QuireError;

/// The `quire` Python module.
#[pymodule]
fn quire(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_document, m)?)?;
    m.add_function(wrap_pyfunction!(load_repo, m)?)?;
    m.add_class::<Spec>()?;
    m.add_class::<Registry>()?;
    Ok(())
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

fn quire_error_to_pyerr(e: QuireError) -> PyErr {
    PyValueError::new_err(e.to_string())
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
