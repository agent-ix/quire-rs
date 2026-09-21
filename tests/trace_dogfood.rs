//! FR-051 — this repository's own benchmark and fuzz tags actually bind
//! (TC-828, CR-061).
//!
//! CR-058 measured TC-577, TC-579 and TC-502 as unbackable by construction and
//! marked them 🚧 as the least-wrong option. CR-061 made a benchmark and a fuzz
//! target bindable, and the two rows are ✅ again — which is a claim about
//! *these files, as authored*, not only about the engine.
//!
//! Two ways to silently un-back them again, both of which this test catches:
//! rewriting the bench tag as `NFR-002-AC-4 / TC-577` (the declared list
//! separator is a comma, so the second id is dropped), or moving the fuzz tag
//! into the `//!` module header (which matches no declared legacy form).
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

#[trace("TC-828", "FR-051-AC-17")]
// the criterion bench backs its row, and it (CR-061)
// backs *every* id on its tag line — the comma-list form.
#[test]
fn tc828_the_validate_document_bench_backs_tc577() {
    let backed = backed_in("benches");
    for id in ["TC-577", "NFR-002-AC-4", "NFR-007-AC-1"] {
        assert!(
            backed.contains(&id.to_string()),
            "{id} is tagged on bench_validate_document and must bind: {backed:?}"
        );
    }
}

#[trace("TC-828", "FR-051-AC-17")]
// the fuzz target backs its row, from a tag (CR-061)
// several lines above the invocation — the span is the whole file.
#[test]
fn tc828_the_fuzz_target_backs_tc579() {
    let backed = backed_in("fuzz/fuzz_targets");
    assert!(
        backed.contains(&"TC-579".to_string()),
        "TC-579 is tagged in fuzz_validate_extract_query.rs and must bind: {backed:?}"
    );
}

/// PLAT-868 PR #479 review, F1: `TC-1880` (`FR-051-AC-25`) was marked ✅ in
/// `spec/tests.md` with three nominated tests in `python.rs`, but none of
/// them actually bound it — a doc-comment mention of an id-shaped token is
/// not a declared trace form (this repo's own `trace_tags` grammar), so
/// `TC-1880` bound nowhere while the matrix read green: exactly the defect
/// class FR-051/TC-1044 exists to catch, caught here on this repo's own tree
/// instead. Fixed by adding `#[trace("TC-1880", "FR-051-AC-25")]` to each of
/// the three; this test is the regression pin so it cannot silently recur.
///
/// Checks `graph.verifies` directly, by `(symbol, trace_id)` — not
/// `unmatched_tags`, which `src/symbols/python.rs`'s own test fixtures make
/// noisy on their own account: several already-correct tests in that file
/// embed id-shaped literals as fixture *payload* (e.g. `tc800`'s `"TC-028"`,
/// `"TC-029"`), and every one of those payload literals that is not itself
/// the symbol's own bound id lands in `unmatched_tags` too, regardless of
/// whether that test's real tag bound. `unmatched_tags` is real signal for
/// this repo's own health, just not a per-symbol pass/fail check.
#[trace("TC-1880", "FR-051-AC-25")]
#[test]
fn tc1880_the_python_decorator_wrap_tests_actually_bind() {
    let graph = graph_for("src/symbols");
    let backed = graph.backed_trace_ids();
    assert!(
        backed.contains("TC-1880"),
        "TC-1880 must bind from python.rs's own tests: {backed:?}"
    );

    for symbol in [
        "tests::plat234_a_black_wrapped_multiline_decorator_reaches_leading_line",
        "tests::a_second_wrapped_decorator_between_the_tag_and_def_does_not_move_leading_line",
        "tests::tc800_wrapped_signature_span_reaches_the_docstring",
    ] {
        assert!(
            graph
                .verifies
                .iter()
                .any(|r| r.symbol.contains(symbol) && r.trace_id == "TC-1880"),
            "{symbol} must carry a verifies relation to TC-1880: {:?}",
            graph
                .verifies
                .iter()
                .filter(|r| r.symbol.contains(symbol))
                .collect::<Vec<_>>()
        );
    }
}
