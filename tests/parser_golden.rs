//! Byte-identity of `parse_document` against a **pre-CR-046** capture.
//!
//! CR-046 split `parse_document` into a header tier and a body tier and
//! claimed the output was unchanged. Nothing pinned that claim: the
//! composition proptest asserts `parse_body(s, &parse_header(s)) ==
//! parse_document(s)`, but after the refactor both sides funnel into the same
//! `parse_body_at` with `body_offset` computed by the identical expression, so
//! an off-by-one in the shared pipeline passes on both sides. It compares via
//! `PartialEq`, and it has no pre-refactor reference at all.
//!
//! `tests/fixtures/parser_golden/expected.json` is that reference: the
//! serialized `parse_document` output over the fixture corpus, captured by
//! running the engine at `7b1db82` — the commit *before* CR-046 landed
//! (`3140b4f`). Every refactor of the parse pipeline since is measured
//! against an engine that predates it.
//!
//! **Do not regenerate this snapshot to make a test pass.** Its whole value
//! is that the current engine did not produce it. A deliberate, intended
//! change to parser output is a spec change: state it as a CR note and record
//! the diff there (FR-005-AC-8 / TC-821).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quire_rs::{parse_body, parse_document, parse_header};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parser_golden")
}

/// Fixture file names, sorted — the same order the snapshot was captured in.
fn fixture_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("read fixture dir")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".md"))
        .collect();
    names.sort();
    assert!(!names.is_empty(), "the golden corpus must not be empty");
    names
}

/// Render the corpus exactly as the capture did: a name → document map,
/// pretty-printed with a trailing newline.
fn render(dir: &Path, names: &[String], parse: impl Fn(&str) -> serde_json::Value) -> String {
    let mut map: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for name in names {
        let text = std::fs::read_to_string(dir.join(name)).expect("read fixture");
        map.insert(name.clone(), parse(&text));
    }
    format!(
        "{}\n",
        serde_json::to_string_pretty(&map).expect("render snapshot")
    )
}

/// TC-821 (FR-005-AC-8): the corpus still carries the byte shapes it exists
/// to cover. A checkout that normalized line endings would rewrite the
/// fixtures, and the snapshot mismatch that follows would read as parser
/// drift; this says what actually happened. (`.gitattributes` in the fixture
/// directory marks them `-text` to prevent it.)
#[test]
fn tc821_golden_corpus_keeps_its_byte_shapes() {
    let dir = fixture_dir();
    let crlf = std::fs::read(dir.join("bom_crlf.md")).expect("read CRLF fixture");
    assert!(
        crlf.starts_with(b"\xEF\xBB\xBF"),
        "bom_crlf.md lost its BOM — the checkout rewrote the fixture, \
         this is not parser drift"
    );
    assert!(
        crlf.windows(2).any(|w| w == b"\r\n"),
        "bom_crlf.md lost its CRLF line endings — the checkout rewrote the \
         fixture, this is not parser drift"
    );
}

/// TC-821 (FR-005-AC-8): `parse_document` over the golden corpus is
/// byte-identical to the snapshot captured from the pre-CR-046 engine.
#[test]
fn tc821_parse_document_matches_the_pre_cr046_snapshot() {
    let dir = fixture_dir();
    let names = fixture_names(&dir);
    let expected = std::fs::read_to_string(dir.join("expected.json")).expect("read snapshot");

    let actual = render(&dir, &names, |text| {
        serde_json::to_value(parse_document(text)).expect("serialize")
    });

    assert_eq!(
        actual, expected,
        "parse_document output drifted from the pre-CR-046 capture \
         (tests/fixtures/parser_golden/expected.json). If the change is \
         intended, it is a spec change: record it as a CR note and update the \
         snapshot deliberately — do not regenerate it to make this pass."
    );
}

/// TC-821 (FR-005-AC-8): and the two-tier path lands on the same **bytes**,
/// not merely on a `PartialEq`-equal value — which is the composition claim
/// CR-046 actually made. Fixtures that are not documents (`parse_header` is
/// `None`) have no two-tier path and are covered by the test above.
#[test]
fn tc821_tier_composition_is_byte_identical_on_the_golden_corpus() {
    let dir = fixture_dir();
    let names: Vec<String> = fixture_names(&dir)
        .into_iter()
        .filter(|n| {
            let text = std::fs::read_to_string(dir.join(n)).expect("read fixture");
            parse_header(&text).is_some()
        })
        .collect();
    assert!(
        names.len() >= 2,
        "the golden corpus must hold documents, not just non-documents"
    );

    let composed = render(&dir, &names, |text| {
        let header = parse_header(text).expect("filtered to documents");
        serde_json::to_value(parse_body(text, &header)).expect("serialize")
    });
    let whole = render(&dir, &names, |text| {
        serde_json::to_value(parse_document(text)).expect("serialize")
    });

    assert_eq!(
        composed, whole,
        "parse_body under parse_header must serialize byte-for-byte to parse_document"
    );
}
