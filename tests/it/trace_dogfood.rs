//! FR-051 — this repository's own benchmark and fuzz tags actually bind
//! (CR-061).
//!
//! Two ways to silently un-back them, both of which this test catches:
//! rewriting the bench tag as `NFR-002-AC-4 / NFR-007-AC-1` (the declared list
//! separator is a comma, so the second id is dropped), or moving the fuzz tag
//! into the `//!` module header (which matches no declared marker form).
//!
//! The fixture `iso` module is used rather than a real `spec-artifacts-process`
//! checkout, so this runs in CI, where there is neither network nor a module.

use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::symbols::{extract_tree, trace};
use quire_rs::Registry;

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn graph_for(dir: &str) -> trace::SymbolGraph {
    let module = repo("tests/fixtures/traceability/iso");
    let model = Registry::load_module(&module)
        .expect("load iso fixture module")
        .traceability()
        .cloned()
        .expect("declared model");
    let extraction = extract_tree(&repo(dir));
    assert!(
        !extraction.symbols.is_empty(),
        "{dir} yielded no symbols at all"
    );
    trace::bind(&extraction, &model)
}

fn backed_in(dir: &str) -> Vec<String> {
    graph_for(dir)
        .backed_trace_ids()
        .into_iter()
        .map(str::to_string)
        .collect()
}

#[trace("FR-051-AC-17")]
// the criterion bench (CR-061)
// backs *every* id on its tag line — the comma-list form.
#[test]
fn the_validate_document_bench_backs_its_criteria() {
    let backed = backed_in("benches");
    for id in ["NFR-002-AC-4", "NFR-007-AC-1"] {
        assert!(
            backed.contains(&id.to_string()),
            "{id} is tagged on bench_validate_document and must bind: {backed:?}"
        );
    }
}

#[trace("FR-051-AC-17")]
// the fuzz target backs its criterion, from a tag (CR-061)
// several lines above the invocation — the span is the whole file.
#[test]
fn the_fuzz_target_backs_its_criterion() {
    let backed = backed_in("fuzz/fuzz_targets");
    assert!(
        backed.contains(&"NFR-019-AC-1".to_string()),
        "NFR-019-AC-1 is tagged in fuzz_validate_extract_query.rs and must bind: {backed:?}"
    );
}
