//! Port of `~/dev/quire-py/tests/test_frontmatter.py` (FR-006).
//!
//! Divergences (see `tests/parser_parity/divergences.md`):
//! - `test_non_string_input_raises_typeerror` — Rust signature is
//!   `extract_frontmatter(&str)`; the null case is unrepresentable.
//!   Skipped.

use quire_rs::extract_frontmatter;
use serde_json::{json, Value};

fn fm_as_value(fm: &Option<serde_json::Map<String, Value>>) -> Option<Value> {
    fm.as_ref().map(|m| Value::Object(m.clone()))
}

#[test]
fn no_leading_dashes_returns_none_and_original_body() {
    let md = "## A heading\ncontent";
    let r = extract_frontmatter(md);
    assert!(r.frontmatter.is_none());
    assert_eq!(r.body, md);
}

#[test]
fn simple_frontmatter_parsed() {
    let md = "---\ntitle: Hello\ntype: doc\n---\nbody text";
    let r = extract_frontmatter(md);
    assert_eq!(
        fm_as_value(&r.frontmatter),
        Some(json!({"title": "Hello", "type": "doc"}))
    );
    assert_eq!(r.body, "body text");
}

#[test]
fn unclosed_frontmatter_returns_none() {
    let md = "---\nkey: value\nno closing marker\nstill no marker";
    let r = extract_frontmatter(md);
    assert!(r.frontmatter.is_none());
    assert_eq!(r.body, md);
}

#[test]
fn malformed_yaml_returns_none_and_original() {
    let md = "---\nkey: { incomplete\n---\nbody";
    let r = extract_frontmatter(md);
    assert!(r.frontmatter.is_none());
    assert_eq!(r.body, md);
}

#[test]
fn empty_frontmatter_returns_none() {
    let md = "---\n---\nbody";
    let r = extract_frontmatter(md);
    assert!(r.frontmatter.is_none());
}

#[test]
fn non_dict_yaml_returns_none() {
    let md = "---\njust a string\n---\nbody";
    let r = extract_frontmatter(md);
    assert!(r.frontmatter.is_none());
}

#[test]
fn array_value_in_frontmatter() {
    let md = "---\ntags:\n  - a\n  - b\n---\nbody";
    let r = extract_frontmatter(md);
    assert_eq!(
        fm_as_value(&r.frontmatter),
        Some(json!({"tags": ["a", "b"]}))
    );
    assert_eq!(r.body, "body");
}

#[test]
fn body_strips_leading_newline_after_closing_dashes() {
    let md = "---\nk: v\n---\n\nbody starts here";
    let r = extract_frontmatter(md);
    assert_eq!(r.body, "\nbody starts here");
}

// Skipped: `test_non_string_input_raises_typeerror`. Rust signature
// takes &str; the null case is unrepresentable.
