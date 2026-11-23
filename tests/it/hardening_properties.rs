//! Small generated properties for the retained evidence and export boundaries.

use ix_trace_rs::trace;
use proptest::prelude::*;
use quire_rs::symbols::extract_file;
use quire_rs::symbols::trace::{bind, trace_id_segments};
use quire_rs::traceability::{SourceLanguage, TraceMarkerForm, TraceabilityModel};
use quire_rs::evidence::{CauseCode, EvidenceResult, RecordBinder, RetainedLocator, TargetLocus};

fn cause_code() -> impl Strategy<Value = CauseCode> {
    (0usize..CauseCode::all().len()).prop_map(|index| CauseCode::all()[index])
}

fn target_locus() -> impl Strategy<Value = TargetLocus> {
    (
        ".{1,32}",
        ".{1,32}",
        ".{1,32}",
        ".{1,24}",
        proptest::option::of(any::<usize>()),
        proptest::option::of(any::<usize>()),
    )
        .prop_map(|(source, document, id, origin, row, line)| TargetLocus {
            source,
            document,
            id,
            origin,
            row,
            line,
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// FR-078-AC-6: the public retained-result vocabulary survives its JSON
    /// report boundary without losing the original request, locus or cause.
    #[test]
    #[trace("FR-078-AC-6")]
    fn tc1799_evidence_result_json_round_trip_preserves_cause_and_locators(
        requested_target in ".{0,48}",
        detail in ".{0,64}",
        record in ".{0,24}",
        run in ".{0,24}",
        entry in ".{0,24}",
        cause in cause_code(),
        target in proptest::option::of(target_locus()),
    ) {
        let result = EvidenceResult {
            target,
            requested_target,
            locator: Some(RetainedLocator { record, run, entry }),
            cause,
            detail,
        };
        let bytes = serde_json::to_vec(&result).expect("typed evidence result serializes");
        let decoded: EvidenceResult = serde_json::from_slice(&bytes).expect("typed result parses");
        prop_assert_eq!(decoded, result);
    }

    /// FR-078-AC-6: a qualified binder's complete identity and provenance
    /// survive the same retained JSON boundary.
    #[test]
    #[trace("FR-078-AC-6")]
    fn tc1799_record_binder_json_round_trip_preserves_identity(
        target in target_locus(),
        record in ".{1,24}",
        run in ".{1,24}",
        entry in ".{1,24}",
        method in ".{1,24}",
        actor in ".{1,24}",
        provenance in proptest::collection::vec(".{1,24}", 0..5),
    ) {
        let binder = RecordBinder {
            target,
            locator: RetainedLocator { record, run, entry },
            method,
            actor,
            provenance,
        };
        let bytes = serde_json::to_vec(&binder).expect("typed binder serializes");
        let decoded: RecordBinder = serde_json::from_slice(&bytes).expect("typed binder parses");
        prop_assert_eq!(decoded, binder);
    }
}

#[test]
fn tc1799_cause_code_names_are_unique_and_bidirectional() {
    let codes = CauseCode::all();
    for (index, code) in codes.iter().enumerate() {
        assert_eq!(CauseCode::from_code(code.as_str()), Some(*code));
        assert!(
            codes[..index]
                .iter()
                .all(|earlier| earlier.as_str() != code.as_str()),
            "duplicate machine cause code {}",
            code.as_str()
        );
        let encoded = serde_json::to_string(code).unwrap();
        assert_eq!(
            encoded,
            format!("\"{}\"", code.as_str()),
            "serde name and as_str drifted"
        );
        assert_eq!(serde_json::from_str::<CauseCode>(&encoded).unwrap(), *code);
    }
}

fn trailer_model() -> TraceabilityModel {
    let mut model = TraceabilityModel::default();
    model.trace_tags.implements.push(TraceMarkerForm {
        syntax: None,
        name: "rust-implements-trailer".to_string(),
        language: SourceLanguage::Rust,
        pattern: r"Implements: ([A-Za-z0-9-]+)".to_string(),
        template: None,
        entry_paths: None,
    });
    model
}

/// Healthy control for the `fuzz_trace_trailers` target: a valid trailer binds
/// its id, and the id fold splits on non-alphanumerics.
#[trace("FR-062-AC-1")]
#[test]
fn tc1799_trace_trailer_binds_its_id_and_ids_fold_to_segments() {
    let source = "/// Implements: FR-001\npub fn production() {}\n";
    let extraction = extract_file("lib.rs", SourceLanguage::Rust, source);
    let graph = bind(&extraction, &trailer_model());
    assert_eq!(graph.implements.len(), 1, "{:?}", graph.implements);
    assert_eq!(graph.implements[0].trace_id, "FR-001");
    assert_eq!(trace_id_segments("fr-001--ac-2"), ["FR", "001", "AC", "2"]);
}

/// Healthy control for the `fuzz_assurance_export` target: the valid v1 fixture
/// is accepted as reader input and round-trips unchanged.
#[trace("FR-078-AC-6")]
#[test]
fn tc1799_valid_assurance_fixture_is_accepted_and_round_trips() {
    let bytes = include_bytes!("../fixtures/assurance/v1.json");
    let export: quire_rs::AssuranceExport = serde_json::from_slice(bytes).unwrap();
    let accepted = quire_rs::AcceptedAssurancePremises::from_export(&export);
    let read = quire_rs::read_assurance_export(bytes, &accepted).expect("fixture accepted");
    let again = quire_rs::read_assurance_export(&read.to_json_bytes().unwrap(), &accepted)
        .expect("round trip accepted");
    assert_eq!(again, read);
}
