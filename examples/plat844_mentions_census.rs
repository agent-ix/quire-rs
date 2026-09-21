//! PLAT-844: a size measurement of the new `SymbolGraph.mentions` pass over
//! the same six-repo corpus PLAT-840/843 measured, taken **before** the
//! `mentions` design is treated as final.
//!
//! The coordinator's own concern, stated verbatim: widening the generic-id
//! scan to whole files including string literals will pick up **fixture
//! trees** on purpose (this ecosystem's fixture corpora are dense with
//! id-shaped tokens by construction), and a search for a common id could
//! return hundreds of hits from fixtures and drown a handful of real ones.
//! Fixtures are **not filtered** here — that would be exactly the silent drop
//! this design exists to avoid — this program only counts, so the coordinator
//! can decide whether the CLI needs an explicit, caller-controlled grouping
//! or path filter, informed by a real number rather than either party's guess.
//!
//! ```text
//! cargo run --release --example plat844_mentions_census
//! ```
//!
//! Reuses the exact target list and shared module `plat840_rust_baseline_sweep`
//! measured with, so this number is comparable to that report's population.
use std::collections::BTreeMap;
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

fn main() {
    let module = match Registry::load_module(Path::new(SHARED_MODULE)) {
        Ok(registry) => registry,
        Err(e) => {
            eprintln!("cannot load shared module {SHARED_MODULE}: {e}");
            std::process::exit(1);
        }
    };
    let model = match module.traceability() {
        Some(m) => m.clone(),
        None => {
            eprintln!("shared module declares no traceability model");
            std::process::exit(1);
        }
    };

    let mut total = 0usize;
    let mut by_repo: Vec<(&str, usize)> = Vec::new();
    // (repo, trace_id) -> count, so the worst single id is named, not just
    // its count.
    let mut per_id: BTreeMap<(&str, String), usize> = BTreeMap::new();
    let mut by_bucket: BTreeMap<&'static str, usize> = BTreeMap::new();

    for target in TARGETS {
        let path = std::env::var(target.path_env).unwrap_or_else(|_| target.default_path.into());
        let root = Path::new(&path);
        if !root.join("spec").exists() && !root.exists() {
            eprintln!(
                "{}: MISSING at {path}, skipped (not silently — named here)",
                target.name
            );
            continue;
        }
        // No source_exclude filtering here on purpose (extract_tree, not
        // extract_tree_scoped): the whole point of this measurement is
        // whether FIXTURE trees flood the count, so fixtures must stay in.
        let extraction = symbols::extract_tree(root);
        let graph = trace::bind(&extraction, &model);
        let count = graph.mentions.len();
        total += count;
        by_repo.push((target.name, count));
        for mention in &graph.mentions {
            *per_id
                .entry((target.name, mention.trace_id.clone()))
                .or_insert(0) += 1;
            *by_bucket.entry(mention.bucket.as_str()).or_insert(0) += 1;
        }
    }

    println!("PLAT-844 mentions census (unfiltered, fixtures included)");
    println!("==========================================================");
    for (name, count) in &by_repo {
        println!("  {name:<20} {count:>8}");
    }
    println!("  {:<20} {total:>8}", "TOTAL");
    println!();
    println!("by bucket:");
    for (bucket, count) in &by_bucket {
        println!("  {bucket:<25} {count:>8}");
    }
    println!();

    let mut worst: Vec<((&str, String), usize)> = per_id.into_iter().collect();
    worst.sort_by_key(|entry| std::cmp::Reverse(entry.1));
    println!("worst 10 single ids:");
    for ((repo, id), count) in worst.into_iter().take(10) {
        println!("  {count:>6}  {repo:<20} {id}");
    }
}
