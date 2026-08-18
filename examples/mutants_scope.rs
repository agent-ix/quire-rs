// agent-ix/quoin#48, pilot half: compute the file set a mutation run should be
// scoped to for one requirement, instead of mutating the whole crate.
//
// The traced-file set for a requirement is **computable rather than guessed**
// (FR-051): the symbol extractor binds each evidence symbol to the trace ids its
// declared marker forms carry, so "which tests verify FR-026" is a query, not a
// grep.
//
// Run:  cargo run --release --example mutants_scope -- FR-026
//       cargo run --release --example mutants_scope -- FR-026 --files-only
//
// `--files-only` prints one mutable path per line, which is what
// `make mutants-fr` feeds to `cargo mutants --file`.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use quire_rs::symbols::{extract_tree, trace};
use quire_rs::Registry;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(requirement) = args.next() else {
        eprintln!("usage: mutants_scope <REQUIREMENT-ID> [--files-only]");
        std::process::exit(2);
    };
    let files_only = args.any(|a| a == "--files-only");

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // The traceability model is module data, so this needs a module checkout.
    // Overridable rather than hardcoded to one machine's layout: the default is
    // the conventional sibling checkout, and `QUIRE_PROCESS_MODULE` names it
    // anywhere else. It fails with the variable's name rather than a path
    // nobody else has (CR-072).
    let module = std::env::var("QUIRE_PROCESS_MODULE").unwrap_or_else(|_| {
        root.parent()
            .unwrap_or(Path::new(".."))
            .join("spec-artifacts-process/spec_artifacts_process")
            .to_string_lossy()
            .into_owned()
    });
    let module = Path::new(&module);
    if !module.is_dir() {
        eprintln!(
            "traceability module not found at {}\n\
             set QUIRE_PROCESS_MODULE to a spec-artifacts-process checkout",
            module.display()
        );
        std::process::exit(1);
    }
    let registry = match Registry::load_module(module) {
        Ok(registry) => registry,
        Err(e) => {
            eprintln!("could not load {}: {e}", module.display());
            std::process::exit(1);
        }
    };
    let Some(model) = registry.traceability() else {
        eprintln!(
            "{} declares no traceability model — the scope needs one to resolve \
             the requirement→test hop",
            module.display()
        );
        std::process::exit(1);
    };

    let extraction = extract_tree(&root);
    let graph = trace::bind(&extraction, model);

    // A requirement owns its own id and every sub-id beneath it: `FR-026`,
    // `FR-026-AC-3`, `FR-026-CON-1`. Prefix matching on the id boundary is what
    // makes "the tests for this FR" the union of its criteria's tests.
    let owned = |trace_id: &str| -> bool {
        trace_id == requirement
            || trace_id
                .strip_prefix(&requirement)
                .is_some_and(|rest| rest.starts_with('-'))
    };

    // **The hop the ticket assumed away.** quoin#48 says "scope each run to the
    // source files traced from a given FR … the traced-file set for a
    // requirement is computable". It is, but not in one step: `verifies` binds
    // symbols to **TC** ids, not to FR or AC ids (374 TC ids bind in this crate;
    // zero FR ids do). The requirement→test edge lives in the **Test Matrix**,
    // which is module data, so the scope is `FR → {AC} → {TC} → {file}`.
    //
    // Resolved through the module's own declared reference columns rather than
    // by hardcoding "Traces To": the whole point of FR-049/FR-050 is that
    // nothing about the column or the pattern is engine knowledge.
    let matrix_ids = tests_for(&spec_root(&root), model, &owned);

    let mut by_file: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for v in graph
        .verifies
        .iter()
        .filter(|v| owned(&v.trace_id) || matrix_ids.contains(&v.trace_id))
    {
        by_file
            .entry(v.path.clone())
            .or_default()
            .insert(v.trace_id.clone());
    }

    // `verifies` binds **evidence** symbols — a test, a benchmark, a fuzz target
    // (FR-051, CR-061). cargo-mutants mutates production code, so a `tests/`
    // file is a witness that the requirement is covered but is not itself a
    // mutable target. In this crate most tests are `#[cfg(test)]` modules inside
    // the file they cover, so a `src/` hit *is* the production file — which is
    // exactly why the pilot runs here and why the split is reported rather than
    // silently collapsed.
    let (mutable, evidence_only): (Vec<_>, Vec<_>) = by_file
        .keys()
        .cloned()
        .partition(|p| p.starts_with("src/") || p.contains("/src/"));

    if files_only {
        for path in &mutable {
            println!("{path}");
        }
        return;
    }

    println!("requirement: {requirement}");
    let ids: BTreeSet<&String> = by_file.values().flatten().collect();
    println!("bound trace ids: {}", ids.len());
    for id in &ids {
        println!("  {id}");
    }

    println!(
        "\nmutable files ({}) — cargo-mutants targets:",
        mutable.len()
    );
    for path in &mutable {
        println!("  {path}  [{}]", join(&by_file[path]));
    }

    println!(
        "\nevidence-only files ({}) — verify the requirement but are not mutated:",
        evidence_only.len()
    );
    for path in &evidence_only {
        println!("  {path}  [{}]", join(&by_file[path]));
    }

    if mutable.is_empty() {
        println!(
            "\nNo mutable target. Every symbol verifying {requirement} lives outside `src/`, so \
             the traced-file set names no production code to mutate. That is a real limit of \
             scoping by `verifies`: FR-051 binds evidence symbols, not the code under test, and \
             the two coincide only where tests are co-located with the code."
        );
    }
}

fn join(ids: &BTreeSet<String>) -> String {
    ids.iter().cloned().collect::<Vec<_>>().join(", ")
}

fn spec_root(root: &Path) -> PathBuf {
    root.join("spec")
}

/// The trace ids the Test Matrix associates with a requirement: every row whose
/// referenced ids include one the requirement owns contributes its **row id**,
/// which is the TC id a symbol can bind.
///
/// Declaration-driven end to end — the archetype carrying the rows, the section,
/// the reference column, the row-id column and the extraction pattern all come
/// from the active module's `document_references`.
fn tests_for(
    spec_root: &Path,
    model: &quire_rs::traceability::TraceabilityModel,
    owned: &dyn Fn(&str) -> bool,
) -> BTreeSet<String> {
    let spec = quire_rs::Spec::from_path(spec_root);
    let mut out = BTreeSet::new();

    for declaration in &model.document_references {
        let Some(row_id_column) = declaration.row_id_column.as_deref() else {
            continue; // a declaration with no row id cannot contribute one
        };
        let Ok(pattern) = regex::Regex::new(&declaration.pattern) else {
            continue;
        };
        for doc in spec.by_type(&declaration.archetype) {
            let Some(table) = quire_rs::table_from_section(doc.body(), &declaration.section) else {
                continue;
            };
            let column = |row: &BTreeMap<String, String>, name: &str| -> Option<String> {
                row.iter()
                    .find(|(header, _)| header.eq_ignore_ascii_case(name))
                    .map(|(_, v)| v.clone())
            };
            for row in &table.rows {
                let cells: BTreeMap<String, String> = table
                    .headers
                    .iter()
                    .cloned()
                    .zip(row.iter().cloned())
                    .collect();
                let Some(referenced) = column(&cells, &declaration.column) else {
                    continue;
                };
                let hit = pattern
                    .captures_iter(&referenced)
                    .filter_map(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
                    .any(|id| owned(&id));
                if !hit {
                    continue;
                }
                if let Some(row_id) = column(&cells, row_id_column) {
                    out.insert(row_id.trim().to_string());
                }
            }
        }
    }
    out
}
