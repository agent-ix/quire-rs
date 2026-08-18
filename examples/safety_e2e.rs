//! End-to-end: does a declared hazard with no mitigating requirement get
//! reported, with the engine holding no safety knowledge whatsoever?
fn main() {
    let module = std::env::args().nth(1).expect("module dir");
    let root = std::env::args().nth(2).expect("bundle dir");
    let reg = quire_rs::Registry::load_module(std::path::Path::new(&module))
        .unwrap_or_else(|e| panic!("load module: {e}"));
    let rep = quire_rs::validate_bundle_at(
        std::path::Path::new(&root),
        &reg,
        quire_rs::BundlePosture::Okf,
    );
    println!(
        "errors={} warnings={}",
        rep.errors.len(),
        rep.warnings.len()
    );
    for f in rep.errors.iter().chain(rep.warnings.iter()) {
        println!("  [{}] {}", f.reason, f.message);
    }
}
