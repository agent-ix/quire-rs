//! Per-symbol enumeration for identity-matched differentials (PLAT-843).
//! `path\tqualified_name\tkind\tleading_line`, one line per symbol, sorted.
//!
//! `cargo run --release --example plat843_audit_list -- <root> [language]`
//!
//! `language` is optional and defaults to `rust` — PLAT-843's own
//! reproduction instructions
//! (`reports/2026-09-20-plat843-rust-ast-differential.md`) invoke this
//! binary with a single `<root>` argument and depend on that exact default;
//! changing it would make an already-merged report's reproduction steps
//! silently stop reproducing what they claim to. `python`/`typescript` are
//! added here (PLAT-851) for the pre-port baseline and for PLAT-868/869's
//! own future differentials against it; `all` emits every language's symbols
//! in one sorted list.
//!
//! **`source_exclude` is read from the declared module (PLAT-868)**, not
//! hardcoded — a PLAT-851 review finding (`AUDIT_LIST_MODULE`, same default
//! and env-var-override convention `examples/plat840_rust_baseline_sweep.rs`
//! already uses). The three globs this file hardcoded until now happened to
//! match `spec-artifacts-process/spec_artifacts_process/manifest.yaml`'s own
//! declared list, but nothing enforced that agreement: if the module's own
//! `source_exclude` ever changes, a hardcoded copy here would silently
//! disagree with the baseline this binary is used to reproduce, and that
//! disagreement would read as an extraction delta rather than as what it
//! actually is — this binary's own exclusion list going stale.

use std::path::Path;

use quire_rs::registry::Registry;
use quire_rs::traceability::SourceLanguage;

fn main() {
    let mut args = std::env::args().skip(1);
    let root = args
        .next()
        .expect("usage: plat843_audit_list <root> [rust|python|typescript|all]");
    let language_arg = args.next().unwrap_or_else(|| "rust".to_string());
    let language = match language_arg.as_str() {
        "rust" => Some(SourceLanguage::Rust),
        "python" => Some(SourceLanguage::Python),
        "typescript" => Some(SourceLanguage::Typescript),
        "all" => None,
        other => {
            panic!("unknown language `{other}` — expected one of rust, python, typescript, all")
        }
    };

    let module_path = std::env::var("AUDIT_LIST_MODULE").unwrap_or_else(|_| {
        "/home/peter/dev/spec-artifacts-process/spec_artifacts_process".to_string()
    });
    let source_exclude: Vec<String> = Registry::load_module(Path::new(&module_path))
        .unwrap_or_else(|e| panic!("load module {module_path}: {e}"))
        .traceability()
        .map(|m| m.source_exclude.clone())
        .unwrap_or_default();

    let out = quire_rs::symbols::extract_tree_scoped(
        Path::new(&root),
        &[Path::new("spec")],
        &source_exclude,
    );
    let mut lines: Vec<String> = out
        .symbols
        .iter()
        .filter(|s| language.is_none_or(|l| s.language == l))
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
