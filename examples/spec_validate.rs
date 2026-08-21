// agent-ix/quire-rs#212: the `make validate` gate — the engine under test
// validates its own `spec/` tree.
//
// PR #204 corrupted a `spec/tests.md` row with a single-shot string replace,
// every local gate stayed green, and the corruption shipped inside v0.41.0:
// nothing in `make ci` ever ran structural validation against this repo's own
// matrix. This binary closes that gap with the engine in this working tree —
// not an installed `quire` CLI, which lags the branch under test.
//
// Run:  cargo run --quiet --example spec_validate        (wired as `make validate`)
//
// Scope is exactly `quire validate <file>` over every document in `spec/`:
// per-document structural validation (frontmatter schema + `body_extraction`
// asserts + heading uniqueness + grammar severity promotions) — the layer that
// catches a malformed matrix row. Deliberately NOT `BundlePosture::Strict`
// bundle validation: measured on main at 3ccdab3, the whole-bundle posture
// reports 208 pre-existing corpus-debt errors (dangling trace references,
// index-incomplete, unowned quality characteristics) that are real, tracked
// authoring debt but not this gate's regression surface — a gate that fails on
// clean main gates nothing.
use std::path::PathBuf;

use quire_rs::Registry;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // The archetype registry is module data. Resolve it exactly as the `quire`
    // CLI does — `IX_FILAMENT_MODULES_PATH` / `IX_SCHEMA_PATH`, else
    // `~/.ix/filament/modules/` — because this spec needs the full installed
    // module set (ISO archetypes + process archetypes), not one module dir.
    let registry = match Registry::from_env() {
        Ok(registry) => registry,
        Err(e) => {
            eprintln!("could not load the module set: {e}");
            std::process::exit(1);
        }
    };
    if registry.module_names().next().is_none() {
        eprintln!(
            "no modules found — set IX_FILAMENT_MODULES_PATH or install modules \
             under ~/.ix/filament/modules/"
        );
        std::process::exit(1);
    }

    let corpus = quire_rs::load_repo(&root.join("spec"));
    let mut documents = 0usize;
    let mut failed_documents = 0usize;
    let mut warning_count = 0usize;
    for doc in &corpus.documents {
        // Untyped/frontmatter-less files are bundle-level authoring debt, not
        // structural regressions; `quire validate` diagnoses them separately.
        let Some(archetype_name) = doc.concept_type().map(str::to_string) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(&doc.path) else {
            eprintln!("error: {}: unreadable", doc.path.display());
            failed_documents += 1;
            continue;
        };
        documents += 1;
        let Some(archetype) = registry.archetype(&archetype_name) else {
            eprintln!(
                "error: {}: unknown type '{archetype_name}' (no archetype in the module set)",
                doc.path.display()
            );
            failed_documents += 1;
            continue;
        };
        let result = quire_rs::validate_document_in_registry(&registry, archetype, &text);
        for w in &result.warnings {
            warning_count += 1;
            println!(
                "warning: {}: line {}: {}",
                doc.path.display(),
                w.line.map_or_else(|| "?".to_string(), |l| l.to_string()),
                w.message
            );
        }
        if !result.is_valid {
            failed_documents += 1;
            for e in &result.errors {
                println!(
                    "error: {}: line {}: {}",
                    doc.path.display(),
                    e.line.map_or_else(|| "?".to_string(), |l| l.to_string()),
                    e.message
                );
            }
        }
    }

    println!(
        "spec_validate: {documents} document(s), {failed_documents} failed, \
         {warning_count} warning(s)"
    );
    if failed_documents > 0 {
        std::process::exit(1);
    }
}
