use std::path::Path;

fn main() {
    let root = std::env::args()
        .nth(1)
        .expect("usage: plat843_audit_list <root>");
    let out = quire_rs::symbols::extract_tree_scoped(
        Path::new(&root),
        &[Path::new("spec")],
        &[
            "tests/fixtures/**".to_string(),
            "tests_integration/fixtures/**".to_string(),
            "fixtures/**".to_string(),
        ],
    );
    let mut lines: Vec<String> = out
        .symbols
        .iter()
        .filter(|s| s.language == quire_rs::traceability::SourceLanguage::Rust)
        .map(|s| {
            format!(
                "{}\t{}\t{:?}\t{}",
                s.path, s.qualified_name, s.kind, s.leading_line
            )
        })
        .collect();
    lines.sort();
    for l in lines {
        println!("{l}");
    }
}
