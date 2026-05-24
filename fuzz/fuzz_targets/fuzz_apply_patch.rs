#![no_main]
//! NFR-011 fuzz target — `apply_patch` on the demo archetype must not panic.

use libfuzzer_sys::fuzz_target;
use quire_rs::{apply_patch, Registry};
use serde_json::Value;
use std::sync::OnceLock;

fn registry() -> &'static Registry {
    static R: OnceLock<Registry> = OnceLock::new();
    R.get_or_init(|| {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("tests")
            .join("render_parity")
            .join("modules");
        Registry::load_from(&[p.as_path()]).expect("load")
    })
}

fuzz_target!(|data: &[u8]| {
    let s = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };
    let patch: Value = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(_) => return,
    };
    let r = registry();
    let arch = match r.archetype("demo-item") {
        Some(a) => a,
        None => return,
    };
    let _ = apply_patch(arch, &Value::Object(Default::default()), &patch);
});
