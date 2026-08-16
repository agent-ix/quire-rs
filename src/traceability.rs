//! The declarative traceability model (FR-050).
//!
//! Coverage is **not** an engine concept: quire knows nothing of "FR", "AC", or
//! "TC". A Filament module declares, in its `manifest.yaml` under
//! `traceability:`, which documents mint trace ids, which columns reference
//! them, what the status vocabulary means, and how source symbols carry trace
//! tags. Everything downstream — reference integrity (FR-049), symbol trace
//! binding (FR-051), and the coverage rollup (FR-050) — reads this model, so a
//! module with an entirely different vocabulary gets coverage with no engine
//! change.
//!
//! The model is the same spec-semantics-as-module-data pattern as
//! `body_extraction`, `lint_rules`, and `grammar_ref`: an absent section leaves
//! the model **undeclared** (consumers say so rather than inventing defaults),
//! and a malformed section fails module load like any other manifest shape
//! error.

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A source language an adapter and a trace-marker form can bind to (FR-051).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceLanguage {
    Rust,
    Python,
    #[serde(alias = "ts")]
    Typescript,
}

impl SourceLanguage {
    /// Stable machine-readable label, used in symbol identities (FR-051).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Typescript => "typescript",
        }
    }
}

/// The whole declared model. Every field is optional at the YAML layer so a
/// module can declare only the parts it needs (e.g. reference integrity
/// without a status vocabulary), but what it does declare must be coherent —
/// see [`TraceabilityModel::validate`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TraceabilityModel {
    /// Path globs holding **no traceable data** (CR-060): test fixtures,
    /// deliberately malformed samples, vendored examples. A document under one
    /// mints no trace ids, contributes no reference rows, and is not classified
    /// for criteria — so every declaration is scoped by it, and so is the
    /// CR-028 criteria walk, which has no declaration of its own to hang an
    /// exclusion on.
    ///
    /// Which paths hold test data is a property of the repository, not of one
    /// declaration — a `TraceTarget::exclude` says "these documents mint no
    /// ids for *me*", which is a different, still-supported statement.
    ///
    /// It scopes **traceability only**. An excluded document is still a
    /// document: `validate_bundle` schema- and grammar-checks it like any
    /// other, because a fixture being outside the coverage rollup is not a
    /// licence for it to be malformed in ways nobody reports.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// Documents/sections that mint trace ids.
    #[serde(default)]
    pub trace_targets: Vec<TraceTarget>,
    /// Columns whose cells reference minted trace ids.
    #[serde(default)]
    pub document_references: Vec<DocumentReference>,
    /// The status column and how its values class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<StatusVocabulary>,
    /// How source symbols carry trace ids (FR-051).
    #[serde(default)]
    pub trace_tags: TraceTagGrammar,
    /// Column vocabularies the module's matrix contract and this rollup must
    /// agree on (CR-015). Declaring them here makes the model the single
    /// source: a contract and its coverage computation cannot drift.
    #[serde(default)]
    pub vocabularies: ColumnVocabularies,
}

/// Declared vocabularies for matrix columns (CR-015).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ColumnVocabularies {
    /// Values the test-type column admits. The module declares the whole list;
    /// a consumer that wants the ecosystem core plus local additions declares
    /// both, exactly as `observable_verbs` layers over engine defaults.
    #[serde(default)]
    pub test_type: Vec<String>,
    /// Column carrying the test type on a reference row (CR-041). Needed only
    /// when `no_source_symbol` is declared — that is the one rule that reads a
    /// row's type rather than its status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_type_column: Option<String>,
    /// Test-type values whose verification method mints **no source symbol**
    /// (CR-041): an agent-behaviour eval, an inspection, a demonstration. A row
    /// verified this way can never carry a trace tag, so reporting it as a
    /// status lie asserts something its own declared method makes impossible.
    ///
    /// Module-declared, like every other vocabulary here: the engine has no
    /// opinion about which methods produce code.
    #[serde(default)]
    pub no_source_symbol: Vec<String>,
}

impl ColumnVocabularies {
    /// True when `value` names a declared method that mints no source symbol.
    /// Compared case-insensitively, as the column vocabularies are elsewhere.
    pub fn mints_no_symbol(&self, value: &str) -> bool {
        let v = value.trim();
        self.no_source_symbol
            .iter()
            .any(|declared| declared.trim().eq_ignore_ascii_case(v))
    }
}

/// One kind of trace id and where it is minted: from an archetype the corpus
/// walk already loads, from an **auxiliary source** — a document outside the
/// corpus (e.g. a Test Matrix at `spec/tests.md`) named by a scope-relative
/// path — or from both, when an archetype covers most of the minting documents
/// and one canonical filename the walk skips covers the rest (CR-038).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceTarget {
    /// Kind name, referenced by [`DocumentReference::targets`].
    pub name: String,
    /// Archetype whose documents mint these ids (corpus-minted targets).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archetype: Option<String>,
    /// Scope-relative path of an auxiliary minting document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<PathBuf>,
    /// Scope-relative globs whose matching documents mint nothing (CR-038).
    /// The engine has no idea which paths hold test data; a module says so.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// Heading of the section carrying the minting table.
    pub section: String,
    /// Table column holding the minted id.
    pub id_column: String,
}

/// One kind of reference from a document cell to minted trace ids. `pattern`
/// is a regex whose **first capture group** is the referenced id, so a cell
/// like `Test (TC-707)` and a bare `TC-707` are both module-declared shapes
/// rather than engine knowledge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentReference {
    /// Reference name, used in report entries and diagnostics.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archetype: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<PathBuf>,
    /// Scope-relative globs whose matching documents contribute no reference
    /// rows (CR-038).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// Heading of the section carrying the referencing table.
    pub section: String,
    /// Column whose cells carry the references.
    pub column: String,
    /// Column identifying the referencing row (the row id in report entries).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_id_column: Option<String>,
    /// Regex over the cell; capture group 1 is the referenced trace id.
    pub pattern: String,
    /// Names of the [`TraceTarget`]s these references resolve against.
    pub targets: Vec<String>,
    /// Expand a same-prefix range (`FR-001..FR-006`) into its concrete ids
    /// before extraction (CR-015). Off unless declared — the engine gains no
    /// behaviour a module has not asked for.
    #[serde(default)]
    pub expand_ranges: bool,
    /// Drop parenthetical spans before extraction, so a qualifier
    /// (`FR-022-AC-5 (superseded by FR-030)`) contributes one reference rather
    /// than two (CR-015). Off unless declared.
    #[serde(default)]
    pub strip_annotations: bool,
}

/// The status column and the values that class as complete / pending / failed.
/// The engine never assumes `✅` means anything — the module says so.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusVocabulary {
    /// Column holding the status value.
    pub column: String,
    #[serde(default)]
    pub complete: Vec<String>,
    #[serde(default)]
    pub pending: Vec<String>,
    #[serde(default)]
    pub failed: Vec<String>,
    /// Values marking a row as withdrawn rather than outstanding (CR-015).
    #[serde(default)]
    pub retired: Vec<String>,
}

/// How a status value classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Complete,
    Pending,
    Failed,
    /// Withdrawn rather than outstanding (CR-015).
    Retired,
    /// Declared nowhere in the vocabulary.
    Unknown,
}

impl StatusVocabulary {
    /// Class `value` per the declared vocabulary.
    ///
    /// Matching is on the **leading marker** (CR-015): the ecosystem authors
    /// `✅ Complete` and `⚠️ scale evidence deferred`, where the marker carries
    /// the class and the remainder carries why. An exact match wins first, so a
    /// vocabulary of whole words (`done`, `planned`) behaves as before.
    pub fn class_of(&self, value: &str) -> StatusClass {
        let v = value.trim();
        let exact = |set: &[String]| set.iter().any(|s| s.trim() == v);
        let leading = |set: &[String]| {
            set.iter().any(|s| {
                let s = s.trim();
                !s.is_empty()
                    && v.starts_with(s)
                    // The marker ends where the note begins: any non-alphanumeric
                    // boundary counts (`✅ Complete`, `done, verified`), so
                    // `doneish` is still not `done`.
                    && v[s.len()..]
                        .chars()
                        .next()
                        .map_or(true, |c| !c.is_alphanumeric())
            })
        };
        for (set, class) in [
            (&self.complete, StatusClass::Complete),
            (&self.pending, StatusClass::Pending),
            (&self.failed, StatusClass::Failed),
            (&self.retired, StatusClass::Retired),
        ] {
            if exact(set) {
                return class;
            }
        }
        for (set, class) in [
            (&self.complete, StatusClass::Complete),
            (&self.pending, StatusClass::Pending),
            (&self.failed, StatusClass::Failed),
            (&self.retired, StatusClass::Retired),
        ] {
            if leading(set) {
                return class;
            }
        }
        StatusClass::Unknown
    }
}

/// The trace-tag grammar (FR-051): framework-native **markers** are the
/// canonical form; the textual **legacy** forms are read only during migration
/// and their bindings carry `legacy` provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TraceTagGrammar {
    #[serde(default)]
    pub markers: Vec<TraceMarkerForm>,
    #[serde(default)]
    pub legacy: Vec<TraceLegacyForm>,
}

/// One canonical marker form — a statically parseable construct attached to a
/// test symbol (a pytest marker, a Rust attribute, a TS helper call). `pattern`
/// matches the construct; capture group 1 is the id-bearing argument list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceMarkerForm {
    pub name: String,
    pub language: SourceLanguage,
    pub pattern: String,
    /// Authoring template for the marker, e.g. `#[trace("{ids}")]`. Present
    /// only when the module wants mechanical rewrite suggestions from legacy
    /// forms: FR-051 emits a suggestion "where the equivalent marker is
    /// derivable", and this template is what makes it derivable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

/// One legacy textual form (bare id in a docstring, a `Trace:` line, a
/// line-comment id, a trace-embedding test name). `rewrite_to` names the
/// marker form a mechanical rewrite suggestion should target, when derivable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceLegacyForm {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<SourceLanguage>,
    pub pattern: String,
    /// Template rebuilding the trace id from the captures, e.g. `TC-{1}` for a
    /// test name like `tc741_extracts`. Absent means capture group 1 already
    /// *is* the id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rewrite_to: Option<String>,
}

impl TraceabilityModel {
    /// True when the module declared nothing at all — consumers report the
    /// model as undeclared rather than computing an empty rollup
    /// (FR-050-AC-2/AC-9).
    pub fn is_empty(&self) -> bool {
        self.trace_targets.is_empty()
            && self.document_references.is_empty()
            && self.status.is_none()
            && self.trace_tags.markers.is_empty()
            && self.trace_tags.legacy.is_empty()
            && self.vocabularies.test_type.is_empty()
    }

    /// Look up a declared target by name.
    pub fn target(&self, name: &str) -> Option<&TraceTarget> {
        self.trace_targets.iter().find(|t| t.name == name)
    }

    /// Shape-validate the declaration. Returns a caller-facing reason on the
    /// first problem, which the manifest loader turns into a module-load
    /// failure (FR-050-AC-2).
    pub fn validate(&self) -> Result<(), String> {
        check_excludes("model-level `exclude`", &self.exclude)?;

        let mut target_names: BTreeSet<&str> = BTreeSet::new();
        for target in &self.trace_targets {
            check_named("trace_targets", &target.name)?;
            if !target_names.insert(target.name.as_str()) {
                return Err(format!(
                    "traceability: duplicate trace_targets entry '{}'",
                    target.name
                ));
            }
            check_origin(
                "trace_targets",
                &target.name,
                target.archetype.as_deref(),
                target.document.as_deref(),
            )?;
            check_excludes(
                &format!("trace_targets entry '{}'", target.name),
                &target.exclude,
            )?;
            check_field("trace_targets", &target.name, "section", &target.section)?;
            check_field(
                "trace_targets",
                &target.name,
                "id_column",
                &target.id_column,
            )?;
        }

        let mut reference_names: BTreeSet<&str> = BTreeSet::new();
        for reference in &self.document_references {
            check_named("document_references", &reference.name)?;
            if !reference_names.insert(reference.name.as_str()) {
                return Err(format!(
                    "traceability: duplicate document_references entry '{}'",
                    reference.name
                ));
            }
            check_origin(
                "document_references",
                &reference.name,
                reference.archetype.as_deref(),
                reference.document.as_deref(),
            )?;
            check_excludes(
                &format!("document_references entry '{}'", reference.name),
                &reference.exclude,
            )?;
            check_field(
                "document_references",
                &reference.name,
                "section",
                &reference.section,
            )?;
            check_field(
                "document_references",
                &reference.name,
                "column",
                &reference.column,
            )?;
            check_capturing_pattern("document_references", &reference.name, &reference.pattern)?;
            if reference.targets.is_empty() {
                return Err(format!(
                    "traceability: document_references entry '{}' declares no targets",
                    reference.name
                ));
            }
            for target in &reference.targets {
                if !target_names.contains(target.as_str()) {
                    return Err(format!(
                        "traceability: document_references entry '{}' references undeclared \
                         target '{target}'",
                        reference.name
                    ));
                }
            }
        }

        if let Some(status) = &self.status {
            check_field("status", &status.column, "column", &status.column)?;
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            for value in status
                .complete
                .iter()
                .chain(&status.pending)
                .chain(&status.failed)
            {
                if !seen.insert(value.as_str()) {
                    return Err(format!(
                        "traceability: status value '{value}' is declared in more than one class"
                    ));
                }
            }
        }

        // CR-041: the exemption reads a column, so the column must be named,
        // and a value outside the declared vocabulary is a typo that would
        // silently exempt nothing.
        if !self.vocabularies.no_source_symbol.is_empty() {
            if self.vocabularies.test_type_column.is_none() {
                return Err("traceability: vocabularies.no_source_symbol needs \
                            vocabularies.test_type_column to say which column carries the value"
                    .to_string());
            }
            for value in &self.vocabularies.no_source_symbol {
                if !self
                    .vocabularies
                    .test_type
                    .iter()
                    .any(|t| t.trim().eq_ignore_ascii_case(value.trim()))
                {
                    return Err(format!(
                        "traceability: vocabularies.no_source_symbol value '{value}' is not in \
                         the declared test_type vocabulary"
                    ));
                }
            }
        }

        let mut marker_names: BTreeSet<&str> = BTreeSet::new();
        for marker in &self.trace_tags.markers {
            check_named("trace_tags.markers", &marker.name)?;
            if !marker_names.insert(marker.name.as_str()) {
                return Err(format!(
                    "traceability: duplicate trace_tags.markers entry '{}'",
                    marker.name
                ));
            }
            check_capturing_pattern("trace_tags.markers", &marker.name, &marker.pattern)?;
        }
        for legacy in &self.trace_tags.legacy {
            check_named("trace_tags.legacy", &legacy.name)?;
            check_capturing_pattern("trace_tags.legacy", &legacy.name, &legacy.pattern)?;
            if let Some(rewrite_to) = &legacy.rewrite_to {
                if !marker_names.contains(rewrite_to.as_str()) {
                    return Err(format!(
                        "traceability: trace_tags.legacy entry '{}' rewrites to undeclared \
                         marker '{rewrite_to}'",
                        legacy.name
                    ));
                }
            }
        }
        Ok(())
    }
}

fn check_named(section: &str, name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err(format!("traceability: {section} entry has an empty name"));
    }
    Ok(())
}

fn check_field(section: &str, name: &str, field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!(
            "traceability: {section} entry '{name}' has an empty `{field}`"
        ));
    }
    Ok(())
}

/// A target or reference is minted by an archetype the corpus walk loads, by an
/// auxiliary document, or by both — but never by neither.
///
/// The pair was rejected until CR-038. It is the natural way to say "every
/// document of this archetype, plus the one canonical filename the corpus walk
/// skips": `spec/tests.md` is on `DEFAULT_SKIP`, so archetype binding alone
/// cannot see the file 184 repos call their Test Matrix.
fn check_origin(
    section: &str,
    name: &str,
    archetype: Option<&str>,
    document: Option<&std::path::Path>,
) -> Result<(), String> {
    match (archetype, document) {
        (Some(a), _) if a.trim().is_empty() => Err(format!(
            "traceability: {section} entry '{name}' has an empty `archetype`"
        )),
        (Some(_), _) | (None, Some(_)) => Ok(()),
        (None, None) => Err(format!(
            "traceability: {section} entry '{name}' declares neither `archetype` nor `document`"
        )),
    }
}

/// Exclusion globs must compile; a typo that silently matched nothing would
/// quietly readmit the documents the module meant to keep out (CR-038).
///
/// `location` reads as the place the patterns were authored, because the same
/// rule now guards a model-level `exclude:` that belongs to no entry (CR-060).
fn check_excludes(location: &str, exclude: &[String]) -> Result<(), String> {
    for pattern in exclude {
        if pattern.trim().is_empty() {
            return Err(format!(
                "traceability: {location} has an empty `exclude` pattern"
            ));
        }
        globset::Glob::new(pattern).map_err(|e| {
            format!("traceability: {location} has an invalid `exclude` pattern: {e}")
        })?;
    }
    Ok(())
}

/// Patterns must compile and must capture the id they extract.
fn check_capturing_pattern(section: &str, name: &str, pattern: &str) -> Result<(), String> {
    let compiled = regex::Regex::new(pattern).map_err(|e| {
        format!("traceability: {section} entry '{name}' has an invalid pattern: {e}")
    })?;
    if compiled.captures_len() < 2 {
        return Err(format!(
            "traceability: {section} entry '{name}' pattern captures nothing; the first capture \
             group must be the trace id"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(yaml: &str) -> TraceabilityModel {
        serde_yaml::from_str(yaml).expect("parse")
    }

    const FULL: &str = r#"
trace_targets:
- name: acceptance-criterion
  archetype: FR
  section: Acceptance Criteria
  id_column: ID
- name: test-case
  document: tests.md
  section: Test Cases
  id_column: TC ID
document_references:
- name: verification
  archetype: FR
  section: Acceptance Criteria
  column: Verification
  row_id_column: ID
  pattern: '\((TC-\d+)\)'
  targets: [test-case]
status:
  column: Status
  complete: ["done"]
  pending: ["planned"]
  failed: ["broken"]
trace_tags:
  markers:
  - name: rust-trace-attribute
    language: rust
    pattern: '#\[trace\(([^)]*)\)\]'
  legacy:
  - name: rust-test-name
    language: rust
    pattern: 'fn (tc\d+)_'
    rewrite_to: rust-trace-attribute
"#;

    #[test]
    fn full_model_parses_and_validates() {
        let m = model(FULL);
        assert!(!m.is_empty());
        m.validate().expect("valid");
        assert_eq!(m.trace_targets.len(), 2);
        assert_eq!(
            m.target("test-case").unwrap().document,
            Some(PathBuf::from("tests.md"))
        );
        assert_eq!(m.document_references[0].targets, vec!["test-case"]);
        assert_eq!(
            m.status.as_ref().unwrap().class_of(" done "),
            StatusClass::Complete
        );
        assert_eq!(
            m.status.as_ref().unwrap().class_of("???"),
            StatusClass::Unknown
        );
        assert_eq!(m.trace_tags.markers[0].language, SourceLanguage::Rust);
    }

    #[test]
    fn default_model_is_undeclared() {
        assert!(TraceabilityModel::default().is_empty());
    }

    #[test]
    fn validation_rejects_incoherent_declarations() {
        let cases = [
            // reference to an undeclared target
            (
                r#"
document_references:
- name: r
  archetype: FR
  section: S
  column: C
  pattern: '(TC-\d+)'
  targets: [nope]
"#,
                "undeclared target",
            ),
            // neither archetype nor document
            (
                "trace_targets:\n- name: t\n  section: S\n  id_column: ID\n",
                "neither `archetype` nor `document`",
            ),
            // an archetype declared as an empty string names nothing
            (
                "trace_targets:\n- name: t\n  archetype: '  '\n  section: S\n  id_column: ID\n",
                "empty `archetype`",
            ),
            // an exclude pattern that cannot compile
            (
                "trace_targets:\n- name: t\n  archetype: FR\n  exclude: ['[bad']\n  section: S\n  id_column: ID\n",
                "invalid `exclude` pattern",
            ),
            // non-capturing pattern
            (
                r#"
trace_targets:
- name: t
  archetype: FR
  section: S
  id_column: ID
document_references:
- name: r
  archetype: FR
  section: S
  column: C
  pattern: 'TC-\d+'
  targets: [t]
"#,
                "captures nothing",
            ),
            // duplicate target names
            (
                "trace_targets:\n- name: t\n  archetype: FR\n  section: S\n  id_column: ID\n- name: t\n  archetype: NFR\n  section: S\n  id_column: ID\n",
                "duplicate trace_targets",
            ),
            // a status value in two classes
            (
                "status:\n  column: Status\n  complete: [\"x\"]\n  pending: [\"x\"]\n",
                "more than one class",
            ),
        ];
        for (yaml, expected) in cases {
            let err = model(yaml).validate().expect_err(yaml);
            assert!(err.contains(expected), "{err} != {expected}");
        }
    }

    /// CR-038: the pair used to be rejected. It is how a module says "every
    /// document of this archetype, plus the one canonical filename the corpus
    /// walk skips" in a single entry.
    #[test]
    fn archetype_and_document_may_be_declared_together() {
        let m = model(
            "trace_targets:\n- name: test-case\n  archetype: TestMatrix\n  document: spec/tests.md\n\
             \n  exclude: ['tests/fixtures/**']\n  section: Test Case Summary\n  id_column: Test ID\n",
        );
        m.validate().expect("the pair is a valid origin");
        let target = m.target("test-case").expect("declared");
        assert_eq!(target.archetype.as_deref(), Some("TestMatrix"));
        assert_eq!(target.document, Some(PathBuf::from("spec/tests.md")));
        assert_eq!(target.exclude, vec!["tests/fixtures/**".to_string()]);
    }

    #[test]
    fn exclude_defaults_to_empty_and_is_omitted_when_unset() {
        let m = model(FULL);
        assert!(m.target("test-case").unwrap().exclude.is_empty());
        // FR-050-AC-7: a model declaring no exclusions must serialize exactly
        // as it did before the field existed.
        let yaml = serde_yaml::to_string(&m).expect("serialize");
        assert!(!yaml.contains("exclude"), "{yaml}");
    }

    // TC-826 (CR-060): the model-level `exclude:` is a repository fact, so it
    // is not what makes a model *declared* — a module that says only "these
    // paths are not corpus data" has declared no model to reconcile against.
    #[test]
    fn model_level_exclude_alone_is_still_an_undeclared_model() {
        let m = model("exclude: ['spec/fixtures/**']\n");
        assert_eq!(m.exclude, vec!["spec/fixtures/**".to_string()]);
        assert!(m.is_empty());
        m.validate().expect("valid");

        // FR-050-AC-7: unset, it serializes exactly as before the field.
        let yaml = serde_yaml::to_string(&model(FULL)).expect("serialize");
        assert!(!yaml.contains("exclude"), "{yaml}");
    }

    // TC-826 (CR-060): the same compile check `exclude:` has carried since
    // CR-038 — a pattern that cannot compile would quietly readmit everything
    // the module meant to keep out.
    #[test]
    fn model_level_exclude_patterns_must_compile() {
        let bad = model("exclude: ['[bad']\n");
        let err = bad.validate().expect_err("must not validate");
        assert!(err.contains("invalid `exclude` pattern"), "{err}");
        assert!(
            err.contains("model-level"),
            "names where it was authored: {err}"
        );

        let empty = model("exclude: ['  ']\n");
        let err = empty.validate().expect_err("must not validate");
        assert!(err.contains("empty `exclude` pattern"), "{err}");
    }

    #[test]
    fn unknown_field_is_a_parse_error() {
        let bad = "trace_targets:\n- name: t\n  archetype: FR\n  section: S\n  id_column: ID\n  typo: x\n";
        assert!(serde_yaml::from_str::<TraceabilityModel>(bad).is_err());
    }
}
