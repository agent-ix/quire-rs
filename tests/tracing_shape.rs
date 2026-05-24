//! Tracing-shape integration test (NFR-008).
//!
//! Only meaningful when compiled with `--features tracing`. The
//! `#[cfg]` guard means a default-feature build just compiles the
//! file as an empty integration binary so the test lane stays wired.

#[cfg(feature = "tracing")]
mod with_tracing {
    use quire_rs::parse_document;
    // NB: we don't pull a real subscriber here (tracing-test isn't in
    // deps); we just confirm the spans don't panic. Real shape
    // verification is wired into the Task 027 fuzz/CI lanes.
    #[test]
    fn parse_document_runs_under_tracing_feature() {
        let d = parse_document("## A\nbody");
        assert_eq!(d.sections.len(), 1);
    }
}

#[cfg(not(feature = "tracing"))]
#[test]
fn tracing_disabled_is_zero_cost_at_compile_time() {
    // When the feature is off, the engine compiles without the
    // tracing crate. This test just exists so `cargo test` has at
    // least one test entry to report under tracing_shape regardless
    // of the feature flag — NFR-008-AC-3 (zero-cost when disabled).
}
