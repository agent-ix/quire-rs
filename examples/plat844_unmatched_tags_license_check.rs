//! PLAT-844 cross-check, requested by the coordinator: does the existing,
//! load-bearing `unmatched_tags` population (PLAT-840's `tagged_not_bound`)
//! already contain license-shaped false positives such as `AGPL-3`?
//!
//! `find_mentions` (FR-077) reuses the same generic pattern `generic_tags()`
//! uses for `unmatched_tags` — the question is whether a file's SPDX header
//! can land inside an evidence symbol's LEADING ANNOTATION BLOCK (the window
//! `generic_tags()` scans, `[leading_line, line)`), which is plausible only
//! for the first symbol in a file where nothing separates the header from the
//! declaration.
//!
//! This is a measurement only — `generic_tags`/`unmatched_tags`/
//! `BindingCensus` are untouched by this program and by PLAT-844 generally.
//!
//! ```text
//! cargo run --release --example plat844_unmatched_tags_license_check
//! ```
use std::path::Path;

use quire_rs::symbols::{self, trace};
use quire_rs::Registry;

struct TargetRepo {
    name: &'static str,
    default_path: &'static str,
    path_env: &'static str,
}

const TARGETS: &[TargetRepo] = &[
    TargetRepo {
        name: "quire-rs",
        default_path: "/home/peter/dev/quire-rs",
        path_env: "PLAT840_PATH_QUIRE_RS",
    },
    TargetRepo {
        name: "quire-code-rs",
        default_path: "/home/peter/dev/quire-code-rs",
        path_env: "PLAT840_PATH_QUIRE_CODE_RS",
    },
    TargetRepo {
        name: "quire-contract-ir",
        default_path: "/home/peter/dev/quire-contract-ir",
        path_env: "PLAT840_PATH_QUIRE_CONTRACT_IR",
    },
    TargetRepo {
        name: "quire-protocol",
        default_path: "/home/peter/dev/quire-protocol",
        path_env: "PLAT840_PATH_QUIRE_PROTOCOL",
    },
    TargetRepo {
        name: "filament-ide-rs",
        default_path: "/home/peter/dev/filament-ide-rs",
        path_env: "PLAT840_PATH_FILAMENT_IDE_RS",
    },
    TargetRepo {
        name: "ecaz",
        default_path: "/home/peter/dev/ecaz",
        path_env: "PLAT840_PATH_ECAZ",
    },
];

const SHARED_MODULE: &str = "/home/peter/dev/spec-artifacts-process/spec_artifacts_process";

/// License-SPDX-shaped, not exhaustively — matches the same `[A-Z]{2,4}-\d+`
/// generic id shape but with a known license-identifier prefix. Reported
/// alongside the raw list per repo so a reader can judge for themself rather
/// than trust this allowlist alone.
const LICENSE_PREFIXES: &[&str] = &["AGPL", "GPL", "LGPL", "MPL", "BSD", "MIT", "ISC", "APACHE"];

fn looks_like_a_license_id(trace_id: &str) -> bool {
    LICENSE_PREFIXES
        .iter()
        .any(|p| trace_id.to_uppercase().starts_with(p))
}

fn main() {
    let module = Registry::load_module(Path::new(SHARED_MODULE)).expect("load shared module");
    let model = module
        .traceability()
        .expect("shared module declares traceability")
        .clone();

    let mut total_unmatched = 0usize;
    let mut total_license_shaped = 0usize;

    println!("PLAT-844 cross-check: license-shaped tokens inside `unmatched_tags`");
    println!("=====================================================================");

    for target in TARGETS {
        let path = std::env::var(target.path_env).unwrap_or_else(|_| target.default_path.into());
        let root = Path::new(&path);
        if !root.exists() {
            eprintln!("{}: MISSING at {path}, skipped", target.name);
            continue;
        }
        // extract_tree_scoped (not extract_tree): PLAT-840's own published
        // `tagged_not_bound` figures were measured this way (declared
        // `source_exclude` globs applied), so this is the only invocation
        // whose `unmatched_tags` population is actually comparable to those
        // numbers — an unscoped walk counts a much larger, differently-shaped
        // population (fixture trees included) that answers a different
        // question.
        let extraction =
            symbols::extract_tree_scoped(root, &[Path::new("spec")], &model.source_exclude);
        let graph = trace::bind(&extraction, &model);

        let license_shaped: Vec<_> = graph
            .unmatched_tags
            .iter()
            .filter(|t| looks_like_a_license_id(&t.trace_id))
            .collect();

        total_unmatched += graph.unmatched_tags.len();
        total_license_shaped += license_shaped.len();

        let rust_census = graph.binding_census.iter().find(|c| c.language == "rust");
        let (tagged, bound, tagged_not_bound) = rust_census
            .map(|c| (c.tagged, c.bound, c.tagged.saturating_sub(c.bound)))
            .unwrap_or((0, 0, 0));

        println!(
            "  {:<20} unmatched_tags={:<6} license_shaped={:<4} rust: tagged={tagged} bound={bound} tagged_not_bound={tagged_not_bound}",
            target.name,
            graph.unmatched_tags.len(),
            license_shaped.len()
        );
        for hit in &license_shaped {
            let symbol_is_bound = graph
                .verifies
                .iter()
                .any(|v| v.path == hit.path && v.symbol == hit.symbol);
            println!(
                "      {}:{}  symbol={}  trace_id={}  symbol_is_bound_by_something_else={}",
                hit.path, hit.line, hit.symbol, hit.trace_id, symbol_is_bound
            );
        }
    }

    println!();
    println!("TOTAL unmatched_tags={total_unmatched}  license_shaped={total_license_shaped}");
}
