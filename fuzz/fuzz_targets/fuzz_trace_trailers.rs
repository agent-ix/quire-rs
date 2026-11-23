#![no_main]
//! Fuzz `Implements:` trailer binding in the Rust symbol adapter.
//!
//! Source is parsed without being compiled or executed. Each input is extracted
//! and bound against a model carrying the `Implements:` trailer pattern, so the
//! trailer matcher and the id fold run on arbitrary text. Healthy control: a
//! valid trailer must bind its id before any fuzzed input is judged.

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use quire_rs::symbols::extract_file;
use quire_rs::symbols::trace::{bind, trace_id_segments};
use quire_rs::traceability::{SourceLanguage, TraceMarkerForm, TraceabilityModel};

fn model() -> &'static TraceabilityModel {
    static MODEL: OnceLock<TraceabilityModel> = OnceLock::new();
    MODEL.get_or_init(|| {
        let mut model = TraceabilityModel::default();
        model.trace_tags.implements.push(TraceMarkerForm {
            syntax: None,
            name: "rust-implements-trailer".to_string(),
            language: SourceLanguage::Rust,
            pattern: r"Implements: ([A-Za-z0-9-]+)".to_string(),
            template: None,
            entry_paths: None,
        });
        let healthy = extract_file(
            "healthy.rs",
            SourceLanguage::Rust,
            "/// Implements: FR-001\npub fn production() {}\n",
        );
        let graph = bind(&healthy, &model);
        assert_eq!(graph.implements.len(), 1, "healthy trailer must bind");
        assert_eq!(graph.implements[0].trace_id, "FR-001");
        model
    })
}

fuzz_target!(|data: &[u8]| {
    let model = model();
    let source = String::from_utf8_lossy(data);
    let extraction = extract_file("fuzzed.rs", SourceLanguage::Rust, &source);
    let graph = bind(&extraction, model);
    for relation in &graph.implements {
        let segments = trace_id_segments(&relation.trace_id);
        assert!(
            segments
                .iter()
                .all(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric())),
            "id fold produced a malformed segment"
        );
    }
    assert_eq!(graph, bind(&extraction, model), "binding is not deterministic");
});
