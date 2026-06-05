//! Crate-wide error type (Task 019, NFR-005).
//!
//! `QuireError` is the single error type returned from every fallible
//! API in this crate. Each variant carries enough structured context
//! that downstream tooling (LLM editors per US-001, CI logs, IDE
//! plumbing) can act on it without parsing free-form strings.
//!
//! ## Field-keyed display
//!
//! Per NFR-005, schema violations carry the four-tuple
//! `(field_path, expected, observed, archetype)`. [`format_violation`]
//! is the single sink that renders them — it truncates the `observed`
//! preview at 80 chars and never leaks raw `serde_json::Error` /
//! validator debug strings.
//!
//! ## Variant growth
//!
//! The enum is `#[non_exhaustive]` so downstream crates can match
//! defensively. New variants are added as upstream FRs (loader,
//! validator, resolver) land. The shape stays the same: every variant
//! that points at *something specific* (an archetype, a module, a
//! field, a file path) names that thing in a typed field — never a
//! formatted-into-a-`String` opaque blob.

use std::path::PathBuf;

use thiserror::Error;

/// Maximum length of the value-preview rendered by [`format_violation`].
/// Longer values are truncated with an ellipsis to keep error messages
/// scannable in log streams (NFR-005-AC-1, observed-preview clause).
pub const VIOLATION_PREVIEW_MAX: usize = 80;

/// Top-level error type for the `quire-rs` crate.
///
/// `#[non_exhaustive]` — downstream `match`es should add a wildcard arm.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum QuireError {
    // ── Schema / data integrity ─────────────────────────────────────────
    /// A data document failed JSON-Schema validation. Carries the
    /// four-tuple required by NFR-005 so consumers (LLM retry loops,
    /// editors) can route the message back to the offending field.
    #[error(
        "SchemaViolation [{archetype}]: field {field_path} — expected {expected}; observed {observed}"
    )]
    SchemaViolation {
        /// Registered archetype name (e.g. `"fr"`).
        archetype: String,
        /// Dot-notated path from the typed root to the offending field
        /// (e.g. `data.relationships[0].target`).
        field_path: String,
        /// Human-readable constraint description
        /// (e.g. `pattern ^ix://`, `min length 1`, `enum FR | NFR`).
        expected: String,
        /// Value preview — truncated at [`VIOLATION_PREVIEW_MAX`] chars
        /// by [`format_violation`].
        observed: String,
    },

    // ── Registry / archetype resolution ─────────────────────────────────
    /// `schema_for` / `validate` called with an unregistered name.
    #[error("UnknownArchetype: '{name}' is not registered in this Registry")]
    UnknownArchetype { name: String },

    /// Two archetypes in the same Registry collide on `name`.
    #[error(
        "ArchetypeCollision: archetype '{name}' declared in both modules '{first_module}' and '{second_module}'"
    )]
    ArchetypeCollision {
        name: String,
        first_module: String,
        second_module: String,
    },

    /// Two modules in the same Registry collide on `name`.
    #[error(
        "ModuleCollision: module '{name}' declared at both '{}' and '{}'",
        first_path.display(),
        second_path.display()
    )]
    ModuleCollision {
        name: String,
        first_path: PathBuf,
        second_path: PathBuf,
    },

    // ── Loader ──────────────────────────────────────────────────────────
    /// Aggregated per-archetype load failures. The loader continues past
    /// individual failures and surfaces them all together so consumers
    /// can choose to load-strict or load-best-effort (FR-014).
    #[error("ArchetypeLoadError: {} archetype(s) failed to load", failures.len())]
    ArchetypeLoadError { failures: Vec<ArchetypeLoadFailure> },

    // ── Filesystem / search-path ────────────────────────────────────────
    /// A `manifest.yaml` failed to parse or referenced missing files.
    #[error("ManifestError at {}: {message}", path.display())]
    ManifestError { path: PathBuf, message: String },

    /// A search-path entry was unusable (file-not-directory, permission
    /// denied, or otherwise unreachable). Emitted as a warning at load
    /// time per FR-013 unless `load_strict` is requested.
    #[error("InvalidSearchPath {}: {reason}", path.display())]
    InvalidSearchPath { path: PathBuf, reason: String },

    // ── Extract / DSL ───────────────────────────────────────────────────
    /// A required `Locator` in a body-extraction DSL produced no value.
    /// `key` is the DSL field that was supposed to receive the value;
    /// `locator` is a short description of the locator that failed.
    #[error("MissingField: required DSL key '{key}' (locator: {locator})")]
    MissingField { key: String, locator: String },

    /// A `body_extraction` DSL failed structural validation at load
    /// time (`match` XOR `iterate_over`, unknown key, missing `from:`).
    #[error("DslValidationError [{archetype}]: {reason}")]
    DslValidationError { archetype: String, reason: String },
}

/// One per-archetype load failure, aggregated by
/// [`QuireError::ArchetypeLoadError`].
#[derive(Debug)]
pub struct ArchetypeLoadFailure {
    pub module: String,
    pub archetype: String,
    pub path: PathBuf,
    pub reason: String,
}

/// Render a [`QuireError::SchemaViolation`] into the canonical
/// field-keyed string per NFR-005. Truncates the `observed` preview at
/// [`VIOLATION_PREVIEW_MAX`] chars with an ellipsis.
///
/// Use this in place of `format!("{err}")` when you need the
/// truncation guarantee (CI logs, single-line error banners).
pub fn format_violation(err: &QuireError) -> String {
    match err {
        QuireError::SchemaViolation {
            archetype,
            field_path,
            expected,
            observed,
        } => {
            format!(
                "SchemaViolation [{}]: field {} — expected {}; observed {}",
                archetype,
                field_path,
                expected,
                truncate_preview(observed),
            )
        }
        other => other.to_string(),
    }
}

/// Truncate `value` to [`VIOLATION_PREVIEW_MAX`] chars with an `…`
/// suffix when over. Operates on chars (not bytes) to keep UTF-8 valid.
pub fn truncate_preview(value: &str) -> String {
    let mut end_byte: usize = value.len();
    for (count, (i, _)) in value.char_indices().enumerate() {
        if count == VIOLATION_PREVIEW_MAX {
            end_byte = i;
            break;
        }
    }
    if end_byte < value.len() {
        format!("{}…", &value[..end_byte])
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_violation() -> QuireError {
        QuireError::SchemaViolation {
            archetype: "fr".into(),
            field_path: "data.title".into(),
            expected: "min length 1".into(),
            observed: "".into(),
        }
    }

    // NFR-005-AC-1: SchemaViolation Display contains all four elements.
    #[test]
    fn schema_violation_display_contains_four_tuple() {
        let s = fixture_violation().to_string();
        assert!(s.contains("fr"), "{s}");
        assert!(s.contains("data.title"), "{s}");
        assert!(s.contains("min length 1"), "{s}");
        assert!(s.contains("observed"), "{s}");
    }

    // NFR-005-AC-2: no leaked validator/serde internal noise.
    #[test]
    fn schema_violation_display_does_not_leak_validator_internals() {
        let s = fixture_violation().to_string();
        for needle in [
            "ValidationError",
            "JSONPointer",
            "serde_json::Error",
            "RawValue",
            "instance_path",
        ] {
            assert!(!s.contains(needle), "leaked '{needle}' in: {s}");
        }
    }

    // NFR-005-AC-3-style snapshot: pin the canonical shape for the FR
    // archetype so accidental format-string changes are loud.
    #[test]
    fn format_violation_snapshot_for_fr_archetype() {
        let s = format_violation(&fixture_violation());
        assert_eq!(
            s,
            "SchemaViolation [fr]: field data.title — expected min length 1; observed "
        );
    }

    #[test]
    fn format_violation_truncates_long_observed_value() {
        let long: String = "x".repeat(VIOLATION_PREVIEW_MAX * 2);
        let err = QuireError::SchemaViolation {
            archetype: "fr".into(),
            field_path: "data.body".into(),
            expected: "max length 20".into(),
            observed: long,
        };
        let s = format_violation(&err);
        assert!(s.ends_with('…'), "expected ellipsis: {s}");
        assert!(s.contains("SchemaViolation [fr]"), "{s}");
    }

    #[test]
    fn truncate_preview_is_char_safe_for_multibyte() {
        let s: String = "é".repeat(VIOLATION_PREVIEW_MAX + 1);
        let truncated = truncate_preview(&s);
        assert!(truncated.ends_with('…'));
        assert!(truncated.is_char_boundary(0));
    }

    #[test]
    fn unknown_archetype_names_the_offender() {
        let e = QuireError::UnknownArchetype {
            name: "bogus".into(),
        };
        let s = e.to_string();
        assert!(s.contains("bogus"));
        assert!(s.contains("UnknownArchetype"));
    }

    #[test]
    fn archetype_load_error_counts_failures() {
        let failures = vec![
            ArchetypeLoadFailure {
                module: "iso".into(),
                archetype: "fr".into(),
                path: PathBuf::from("/x/y.json"),
                reason: "missing".into(),
            },
            ArchetypeLoadFailure {
                module: "iso".into(),
                archetype: "nfr".into(),
                path: PathBuf::from("/x/z.json"),
                reason: "malformed".into(),
            },
        ];
        let e = QuireError::ArchetypeLoadError { failures };
        let s = e.to_string();
        assert!(s.contains("2 archetype"), "{s}");
    }

    #[test]
    fn module_collision_names_both_paths() {
        let e = QuireError::ModuleCollision {
            name: "iso".into(),
            first_path: PathBuf::from("/a/iso"),
            second_path: PathBuf::from("/b/iso"),
        };
        let s = e.to_string();
        assert!(s.contains("/a/iso"));
        assert!(s.contains("/b/iso"));
    }

    #[test]
    fn quire_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<QuireError>();
    }

    /// NFR-005 "Error Path Rule": every QuireError variant must
    /// have a Display string that carries the variant name + the
    /// load-bearing identifier. Tautology checks (`!is_empty()`)
    /// don't satisfy this — pin the substrings consumers grep on.
    #[test]
    fn every_quire_error_variant_displays_variant_name_and_identifier() {
        let cases: Vec<(QuireError, &[&str])> = vec![
            (
                QuireError::ArchetypeCollision {
                    name: "fr".into(),
                    first_module: "iso".into(),
                    second_module: "app".into(),
                },
                &["ArchetypeCollision", "fr", "iso", "app"],
            ),
            (
                QuireError::ManifestError {
                    path: PathBuf::from("/m/manifest.yaml"),
                    message: "could not parse".into(),
                },
                &["ManifestError", "/m/manifest.yaml", "could not parse"],
            ),
            (
                QuireError::InvalidSearchPath {
                    path: PathBuf::from("/etc/passwd"),
                    reason: "file not directory".into(),
                },
                &["InvalidSearchPath", "/etc/passwd", "file not directory"],
            ),
            (
                QuireError::MissingField {
                    key: "purpose".into(),
                    locator: "section_body(Purpose)".into(),
                },
                &["MissingField", "purpose", "section_body(Purpose)"],
            ),
            (
                QuireError::DslValidationError {
                    archetype: "domain".into(),
                    reason: "match and iterate_over are mutually exclusive".into(),
                },
                &["DslValidationError", "domain", "mutually exclusive"],
            ),
        ];
        for (err, needles) in cases {
            let s = err.to_string();
            for needle in needles {
                assert!(
                    s.contains(needle),
                    "{err:?} Display missing '{needle}': {s}"
                );
            }
        }
    }
}
