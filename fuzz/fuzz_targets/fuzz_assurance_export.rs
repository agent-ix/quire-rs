#![no_main]
//! Fuzz the assurance-v1 reader's complete JSON/schema/premise boundary.
//!
//! The valid fixture is the healthy input oracle: accepted bytes must survive
//! read/serialize/read without changing the typed export. Arbitrary bytes only
//! exercise the fail-closed parser and must never panic.

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use quire_rs::{read_assurance_export, AcceptedAssurancePremises, AssuranceExport};

fn accepted() -> &'static AcceptedAssurancePremises {
    static ACCEPTED: OnceLock<AcceptedAssurancePremises> = OnceLock::new();
    ACCEPTED.get_or_init(|| {
        let bytes = include_bytes!("../../tests/fixtures/assurance/v1.json");
        let export: AssuranceExport = serde_json::from_slice(bytes).expect("valid assurance seed");
        AcceptedAssurancePremises::from_export(&export)
    })
}

fuzz_target!(|data: &[u8]| {
    if let Ok(export) = read_assurance_export(data, accepted()) {
        let encoded = export.to_json_bytes().expect("typed export serializes");
        let decoded = read_assurance_export(&encoded, accepted()).expect("round-trip remains valid");
        assert_eq!(decoded, export, "accepted export round-trip changed records");
    }
});
