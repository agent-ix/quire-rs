#[path = "src/anomaly_sources.rs"]
mod anomaly_sources;

use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let anomaly_dir = manifest_dir.join("spec/anomalies");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("output dir"));
    let output_path = out_dir.join("embedded_anomalies.rs");

    println!("cargo:rerun-if-changed={}", anomaly_dir.display());

    let sources = anomaly_sources::anomaly_sources(&anomaly_dir)
        .unwrap_or_else(|error| panic!("failed to discover anomaly documents: {error}"));
    let mut generated = String::from("static ANOMALY_DOCUMENTS: &[EmbeddedAnomalyDocument] = &[\n");

    for source in sources {
        println!("cargo:rerun-if-changed={}", source.display());
        let relative_path = source
            .strip_prefix(&manifest_dir)
            .expect("anomaly document is inside the repository")
            .to_string_lossy()
            .replace('\\', "/");
        generated.push_str(&format!(
            "    EmbeddedAnomalyDocument {{ path: {relative_path:?}, content: include_str!({source:?}) }},\n"
        ));
    }

    generated.push_str("];\n");
    fs::write(output_path, generated).expect("write generated anomaly source set");
}
