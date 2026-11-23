//! Small generated properties for the retained evidence and export boundaries.

use ix_trace_rs::trace;
use proptest::prelude::*;
use quire_rs::evidence::{CauseCode, EvidenceResult, RecordBinder, RetainedLocator, TargetLocus};

fn cause_code() -> impl Strategy<Value = CauseCode> {
    (0usize..CauseCode::all().len()).prop_map(|index| CauseCode::all()[index])
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// FR-078-AC-6/AC-19: the public retained-result vocabulary survives its
    /// JSON report boundary without losing the original request or cause.
    #[test]
    #[trace("FR-078-AC-6", "FR-078-AC-19")]
    fn tc1799_evidence_result_json_round_trip_preserves_cause_and_locators(
        requested_target in ".{0,48}",
        detail in ".{0,64}",
        record in ".{0,24}",
        run in ".{0,24}",
        entry in ".{0,24}",
        cause in cause_code(),
    ) {
        let result = EvidenceResult {
            target: None,
            requested_target,
            locator: Some(RetainedLocator { record, run, entry }),
            cause,
            detail,
        };
        let bytes = serde_json::to_vec(&result).expect("typed evidence result serializes");
        let decoded: EvidenceResult = serde_json::from_slice(&bytes).expect("typed result parses");
        prop_assert_eq!(decoded, result);
    }

    /// FR-078-AC-6/AC-10: a qualified binder's complete identity and provenance
    /// survive the same retained JSON boundary.
    #[test]
    #[trace("FR-078-AC-6", "FR-078-AC-10")]
    fn tc1799_record_binder_json_round_trip_preserves_identity(
        source in ".{1,32}",
        document in ".{1,32}",
        id in ".{1,32}",
        origin in ".{1,24}",
        record in ".{1,24}",
        run in ".{1,24}",
        entry in ".{1,24}",
        method in ".{1,24}",
        actor in ".{1,24}",
        provenance in proptest::collection::vec(".{1,24}", 0..5),
    ) {
        let binder = RecordBinder {
            target: TargetLocus { source, document, id, origin, row: None, line: None },
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

#[trace("FR-078-AC-6", "FR-078-AC-19")]
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
        assert_eq!(serde_json::from_str::<CauseCode>(&encoded).unwrap(), *code);
    }
}
