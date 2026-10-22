// agent-ix/quire-rs#212: the `make validate` gate — the engine under test
// validates its own `spec/` tree.
//
// A past PR corrupted a `spec/tests.md` row with a single-shot string replace,
// every local gate stayed green, and the corruption shipped:
// nothing in `make ci` ever ran structural validation against this repo's own
// matrix. This binary closes that gap with the engine in this working tree —
// not an installed `quire` CLI, which lags the branch under test.
//
// Run: cargo run --locked --quiet --example spec_validate --
//        --process-root <repo> --iso-root <repo>  (wired as `make validate`)
//
// Scope is exactly `quire validate <file>` over every document in `spec/`:
// per-document structural validation (frontmatter schema + `body_extraction`
// asserts + heading uniqueness + grammar severity promotions) — the layer that
// catches a malformed matrix row. Deliberately NOT `BundlePosture::Strict`
// bundle validation: the whole-bundle posture reports pre-existing
// corpus-debt errors (dangling trace references,
// index-incomplete, unowned quality characteristics) that are real, tracked
// authoring debt but not this gate's regression surface — a gate that fails on
// clean main gates nothing.
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use quire_rs::Registry;

struct ProviderRoots {
    process: PathBuf,
    iso: PathBuf,
}

fn parse_provider_roots(args: impl IntoIterator<Item = OsString>) -> Result<ProviderRoots, String> {
    let mut args = args.into_iter();
    let mut process = None;
    let mut iso = None;
    while let Some(option) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("{} requires a path", option.to_string_lossy()))?;
        match option.to_str() {
            Some("--process-root") if process.is_none() => process = Some(PathBuf::from(value)),
            Some("--iso-root") if iso.is_none() => iso = Some(PathBuf::from(value)),
            _ => {
                return Err(format!(
                    "unknown or duplicate option: {}",
                    option.to_string_lossy()
                ))
            }
        }
    }
    Ok(ProviderRoots {
        process: process.ok_or("missing --process-root")?,
        iso: iso.ok_or("missing --iso-root")?,
    })
}

fn provider_module(root: &Path, directory: &str, label: &str) -> Result<PathBuf, String> {
    let path = root.join(directory);
    let resolved = path.canonicalize().map_err(|error| {
        format!(
            "spec_validate: {label}: module directory missing: {}: {error}",
            path.display()
        )
    })?;
    if !resolved.is_dir() {
        return Err(format!(
            "spec_validate: {label}: module directory missing: {}",
            path.display()
        ));
    }
    Ok(resolved)
}

fn load_provider_registry(providers: &ProviderRoots) -> Result<Registry, String> {
    let process = provider_module(
        &providers.process,
        "spec_artifacts_process",
        "spec-artifacts-process",
    )?;
    let iso = provider_module(&providers.iso, "spec_artifacts_iso", "spec-artifacts-iso")?;
    // ISO sorts before process in the former wrapper's isolated directory.
    let registry = Registry::load_module_set(&[&iso, &process])
        .map_err(|error| format!("could not load the module set: {error}"))?;
    if registry.module_names().next().is_none() {
        return Err("no modules found in the declared schema providers".to_string());
    }
    Ok(registry)
}

fn validation_exclusions(
    root: &std::path::Path,
) -> Result<std::collections::BTreeMap<PathBuf, String>, String> {
    let path = root.join("quality/spec-validation-exclusions.json");
    let text =
        std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    if value.get("schemaVersion").and_then(|v| v.as_str())
        != Some("quire-spec-validation-exclusions-v1")
    {
        return Err(format!("{}: unsupported schemaVersion", path.display()));
    }
    let object = value
        .as_object()
        .ok_or_else(|| format!("{}: root must be an object", path.display()))?;
    if object.len() != 2 || !object.contains_key("entries") || !object.contains_key("schemaVersion")
    {
        return Err(format!("{}: unexpected top-level fields", path.display()));
    }
    let entries = value
        .get("entries")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{}: entries must be an array", path.display()))?;
    let mut result = std::collections::BTreeMap::new();
    for entry in entries {
        let entry = entry
            .as_object()
            .ok_or_else(|| format!("{}: exclusion must be an object", path.display()))?;
        if entry.len() != 2 || !entry.contains_key("path") || !entry.contains_key("reason") {
            return Err(format!(
                "{}: exclusion has unexpected fields",
                path.display()
            ));
        }
        let relative = PathBuf::from(
            entry
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("{}: exclusion path must be a string", path.display()))?,
        );
        let reason = entry
            .get("reason")
            .and_then(|v| v.as_str())
            .filter(|reason| !reason.trim().is_empty())
            .ok_or_else(|| format!("{}: exclusion reason must be non-empty", path.display()))?;
        if relative.is_absolute()
            || relative
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
            || !relative.starts_with("spec/assurance")
        {
            return Err(format!(
                "{}: exclusion must be a safe spec/assurance path: {}",
                path.display(),
                relative.display()
            ));
        }
        if result
            .insert(relative.clone(), reason.to_string())
            .is_some()
        {
            return Err(format!(
                "{}: duplicate exclusion: {}",
                path.display(),
                relative.display()
            ));
        }
    }
    Ok(result)
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let providers = match parse_provider_roots(std::env::args_os().skip(1)) {
        Ok(providers) => providers,
        Err(error) => {
            eprintln!("spec_validate: {error}");
            std::process::exit(2);
        }
    };

    // The user explicitly deferred Phase-7 engineering-assurance workflows.
    // Those already-authored AP/MP documents target a provider revision no
    // longer available from its authoritative remote. They are quarantined by
    // exact path, not by directory or type: an added/renamed assurance artifact
    // therefore fails unknown-type validation instead of silently shrinking the
    // gate. Remove entries when that separately-scoped migration is performed.
    let mut exclusions = match validation_exclusions(&root) {
        Ok(exclusions) => exclusions,
        Err(error) => {
            eprintln!("could not load validation exclusions: {error}");
            std::process::exit(1);
        }
    };

    // The two provider revisions are the complete module set for this gate.
    let registry = match load_provider_registry(&providers) {
        Ok(registry) => registry,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    let corpus = quire_rs::load_repo(&root.join("spec"));
    let mut documents = 0usize;
    let mut failed_documents = 0usize;
    let mut warning_count = 0usize;
    let mut excluded_documents = 0usize;
    let mut skipped_untyped: Vec<String> = Vec::new();
    let mut loaded_paths: std::collections::BTreeSet<PathBuf> = std::collections::BTreeSet::new();
    for doc in &corpus.documents {
        loaded_paths.insert(doc.path.clone());
        // Untyped/frontmatter-less files are bundle-level authoring debt, not
        // structural regressions; `quire validate` diagnoses them separately.
        // NAMED in the summary rather than silently dropped (SR-051): a gate
        // whose population can shrink without a trace goes green on exactly
        // the corruption class it exists to catch.
        let Some(archetype_name) = doc.concept_type().map(str::to_string) else {
            skipped_untyped.push(doc.path.display().to_string());
            continue;
        };
        let relative = doc.path.strip_prefix(&root).unwrap_or(&doc.path);
        if let Some(reason) = exclusions.remove(relative) {
            if matches!(
                archetype_name.as_str(),
                "AssuranceProfile" | "MeasurementPlan"
            ) {
                excluded_documents += 1;
                println!("excluded (Phase 7): {}: {reason}", relative.display());
                continue;
            }
            eprintln!(
                "error: {}: governed exclusion may only name AssuranceProfile/MeasurementPlan",
                relative.display()
            );
            failed_documents += 1;
            continue;
        }
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

    // ── Population integrity (SR-051 FND: the gate's blind spot) ──
    // A one-character corruption of a document's frontmatter fence drops the
    // file from `load_repo`'s corpus entirely, so it never reaches the loop
    // above and the gate shrank by one document, green. Measured: breaking
    // FR-050's opening `---` yielded `126 document(s), 0 failed`, exit 0.
    // The reconciliation: every `.md` on disk under `spec/` is either a
    // loaded document or lives under `spec/assets/` (the two frontmatter-less
    // notes); anything else is a file the engine could not read as a document
    // — the #204 corruption class, one tier up — and fails the gate.
    let mut on_disk: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.join("spec")];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                on_disk.push(path);
            }
        }
    }
    on_disk.sort();
    let assets = root.join("spec").join("assets");
    for path in on_disk {
        if !loaded_paths.contains(&path) && !path.starts_with(&assets) {
            eprintln!(
                "error: {}: on disk but not loaded as a document — corrupt or \
                 missing frontmatter? (spec/ documents must load; move \
                 deliberately frontmatter-less notes under spec/assets/)",
                path.display()
            );
            failed_documents += 1;
        }
    }

    for path in &skipped_untyped {
        println!("skipped (untyped, not validated): {path}");
    }
    for (path, _) in exclusions {
        eprintln!(
            "error: {}: governed validation exclusion does not name a loaded document",
            path.display()
        );
        failed_documents += 1;
    }
    println!(
        "spec_validate: {documents} document(s), {failed_documents} failed, \
         {warning_count} warning(s), {} untyped skipped, {excluded_documents} Phase-7 excluded",
        skipped_untyped.len(),
    );
    if failed_documents > 0 {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{load_provider_registry, parse_provider_roots};
    use std::ffi::OsString;
    use std::path::Path;

    fn write_module(root: &Path, name: &str, archetype: &str) {
        std::fs::create_dir_all(root.join("schemas")).expect("module directory");
        std::fs::write(
            root.join("manifest.yaml"),
            format!("name: {name}\nartifact_types:\n- name: {archetype}\n  frontmatter_schema_ref: schemas/schema.json\n"),
        )
        .expect("module manifest");
        std::fs::write(root.join("schemas/schema.json"), r#"{"type":"object"}"#)
            .expect("module schema");
    }

    /// Trace: FR-013-AC-17. The gate loads only its two named providers.
    #[test]
    fn spec_validation_uses_only_declared_provider_modules() {
        let scratch = tempfile::tempdir().expect("scratch providers");
        let process_root = scratch.path().join("process-repo");
        let iso_root = scratch.path().join("iso-repo");
        write_module(
            &process_root.join("spec_artifacts_process"),
            "spec-artifacts-process",
            "ProcessOnly",
        );
        write_module(
            &iso_root.join("spec_artifacts_iso"),
            "spec-artifacts-iso",
            "IsoOnly",
        );
        write_module(
            &iso_root.join("ambient-extra"),
            "ambient-extra",
            "ExtraOnly",
        );

        let args = [
            OsString::from("--process-root"),
            process_root.into_os_string(),
            OsString::from("--iso-root"),
            iso_root.into_os_string(),
        ];
        let providers = parse_provider_roots(args).expect("two explicit provider roots");
        let registry = load_provider_registry(&providers).expect("load selected providers");
        assert!(registry.archetype("ProcessOnly").is_some());
        assert!(registry.archetype("IsoOnly").is_some());
        assert!(
            registry.archetype("ExtraOnly").is_none(),
            "ambient sibling is excluded"
        );

        assert!(parse_provider_roots(Vec::<OsString>::new()).is_err());
        std::fs::remove_dir_all(providers.iso.join("spec_artifacts_iso"))
            .expect("remove required provider");
        assert!(load_provider_registry(&providers)
            .err()
            .expect("missing named provider must fail")
            .contains("module directory missing"));
    }
}
