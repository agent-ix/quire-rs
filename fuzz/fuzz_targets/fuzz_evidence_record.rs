#![no_main]
//! Fuzz retained evidence report-record deserialization and round trips.
//!
//! Both record variants are strict typed readers (`deny_unknown_fields`).
//! Any accepted record must serialize and deserialize identically; malformed
//! or unknown-field inputs are expected refusals, not crashes.

use libfuzzer_sys::fuzz_target;
use quire_rs::evidence::{EvidenceResult, RecordBinder};

fuzz_target!(|data: &[u8]| {
    if let Ok(record) = serde_json::from_slice::<EvidenceResult>(data) {
        let encoded = serde_json::to_vec(&record).expect("typed evidence result serializes");
        let decoded: EvidenceResult =
            serde_json::from_slice(&encoded).expect("evidence result round-trip parses");
        assert_eq!(decoded, record, "evidence result round-trip changed fields");
    }

    if let Ok(record) = serde_json::from_slice::<RecordBinder>(data) {
        let encoded = serde_json::to_vec(&record).expect("typed record binder serializes");
        let decoded: RecordBinder =
            serde_json::from_slice(&encoded).expect("record binder round-trip parses");
        assert_eq!(decoded, record, "record binder round-trip changed fields");
    }
});
