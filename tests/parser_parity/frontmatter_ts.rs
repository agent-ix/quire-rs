//! Port of `~/dev/quire/tests/core/frontmatter.test.ts` (FR-005 + FR-027).
//!
//! Each `#[test]` corresponds to one `it(...)` block in the TS suite.
//!
//! Divergences (see `tests/parser_parity/divergences.md`):
//! - "FR-027-AC-5: null input throws TypeError" — Rust takes `&str`,
//!   the null-input case is unrepresentable. Skipped.
//! - "handles empty frontmatter" — TS returns `null` for the empty
//!   block `---\n---\n`. Rust's `serde_yaml` parses the empty payload
//!   as `null` which fails the "must be a map" filter, so the Rust
//!   port also returns `None`. Behaviour matches TS.

use quire_rs::{extract_frontmatter, FrontmatterResult};
use serde_json::{json, Value};

/// Convenience: convert the parsed map into a `serde_json::Value::Object`
/// so we can compare with the `json!(...)` macro.
fn fm_as_value(r: &FrontmatterResult) -> Option<Value> {
    r.frontmatter.as_ref().map(|m| Value::Object(m.clone()))
}

#[test]
fn fr_005_ac_1_extracts_yaml_frontmatter() {
    let md = "---\nfoo: bar\n---\nbody content";
    let result = extract_frontmatter(md);
    assert_eq!(fm_as_value(&result), Some(json!({ "foo": "bar" })));
    assert_eq!(result.body, "body content");
}

#[test]
fn fr_005_ac_2_no_frontmatter_markers_returns_null() {
    let md = "just body content";
    let result = extract_frontmatter(md);
    assert!(result.frontmatter.is_none());
    assert_eq!(result.body, "just body content");
}

#[test]
fn fr_005_ac_3_parses_array_values() {
    let md = "---\nstandards_alignment: [iso-29148, ieee-828]\n---\nbody";
    let result = extract_frontmatter(md);
    assert_eq!(
        fm_as_value(&result),
        Some(json!({ "standards_alignment": ["iso-29148", "ieee-828"] }))
    );
}

#[test]
fn fr_005_ac_4_parses_related_standards_array() {
    let md = "---\nrelated_standards: [cloudevents]\n---\nbody";
    let result = extract_frontmatter(md);
    assert_eq!(
        fm_as_value(&result),
        Some(json!({ "related_standards": ["cloudevents"] }))
    );
}

#[test]
fn parses_boolean_values() {
    let md = "---\nenabled: true\ndisabled: false\n---\nbody";
    let result = extract_frontmatter(md);
    assert_eq!(
        fm_as_value(&result),
        Some(json!({ "enabled": true, "disabled": false }))
    );
}

#[test]
fn parses_numeric_values() {
    let md = "---\ncount: 42\n---\nbody";
    let result = extract_frontmatter(md);
    assert_eq!(fm_as_value(&result), Some(json!({ "count": 42 })));
}

#[test]
fn parses_quoted_strings() {
    let md = "---\nname: \"hello world\"\n---\nbody";
    let result = extract_frontmatter(md);
    assert_eq!(fm_as_value(&result), Some(json!({ "name": "hello world" })));
}

#[test]
fn fr_027_ac_2_malformed_yaml_returns_null_frontmatter() {
    let md = "---\n{invalid yaml!!!\n---\nbody";
    let result = extract_frontmatter(md);
    assert!(result.frontmatter.is_none());
    assert_eq!(result.body, md);
}

// Skipped: FR-027-AC-5 null-input TypeError. Rust signature is
// `extract_frontmatter(&str)`; the null case is unrepresentable.
// See divergences.md.

#[test]
fn handles_no_closing_marker() {
    let md = "---\nfoo: bar\nno closing marker";
    let result = extract_frontmatter(md);
    assert!(result.frontmatter.is_none());
    assert_eq!(result.body, md);
}

#[test]
fn handles_empty_frontmatter() {
    let md = "---\n---\nbody";
    let result = extract_frontmatter(md);
    // TS: empty YAML block → null. Rust: serde_yaml yields `null`,
    // which is not an object, so the "must be Map" guard returns None.
    // Same observable behaviour (frontmatter is None).
    assert!(result.frontmatter.is_none());
}
