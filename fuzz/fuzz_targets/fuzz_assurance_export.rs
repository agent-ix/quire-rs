#![no_main]
//! Fuzz the assurance-v1 reader's complete JSON/schema/premise boundary.
//!
//! The valid fixture is the healthy control: before any fuzzed input is judged,
//! it is fed through the reader as input and must be accepted and survive
//! read/serialize/read unchanged. Accepted fuzzed bytes get the same round-trip
//! oracle; arbitrary bytes otherwise exercise the fail-closed parser and must
//! never panic.

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use quire_rs::{read_assurance_export, AcceptedAssurancePremises, AssuranceExport};

fn round_trip(data: &[u8], accepted: &AcceptedAssurancePremises) -> Option<AssuranceExport> {
    let export = read_assurance_export(data, accepted).ok()?;
    let encoded = export.to_json_bytes().expect("typed export serializes");
    let decoded = read_assurance_export(&encoded, accepted).expect("round-trip remains valid");
    assert_eq!(decoded, export, "accepted export round-trip changed records");
    Some(export)
}

fn accepted() -> &'static AcceptedAssurancePremises {
    static ACCEPTED: OnceLock<AcceptedAssurancePremises> = OnceLock::new();
    ACCEPTED.get_or_init(|| {
        let bytes = include_bytes!("../../tests/fixtures/assurance/v1.json");
        let export: AssuranceExport = serde_json::from_slice(bytes).expect("valid assurance seed");
        let accepted = AcceptedAssurancePremises::from_export(&export);
        assert!(
            round_trip(bytes, &accepted).is_some(),
            "healthy fixture must be accepted as fuzz input"
        );
        accepted
    })
}

fuzz_target!(|data: &[u8]| {
    let _ = round_trip(data, accepted());
});
