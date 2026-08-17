// FR-056 ecosystem fit check (CR-014 discipline): measure each new check over
// the whole ~/dev corpus BEFORE deciding anything about severity.
use std::collections::BTreeMap;
use std::path::Path;

fn main() {
    let module = Path::new("/home/peter/dev/spec-artifacts-iso/spec_artifacts_iso");
    let registry = quire_rs::Registry::load_module(module).expect("load iso module");
    let mut per_check: BTreeMap<String, usize> = BTreeMap::new();
    let mut docs_total = 0usize;
    let mut docs_hit = 0usize;
    let mut examples: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let only = std::env::args().nth(1);
    let mut repos: Vec<_> = std::fs::read_dir("/home/peter/dev")
        .expect("dev")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("spec").is_dir())
        .collect();
    repos.sort();
    // Dedupe worktree copies the way scripts/sweep_coverage.py does.
    repos.retain(|p| {
        let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if let Some(only) = &only {
            return n == only;
        }
        !n.contains("-task") && n != "worktrees"
    });

    for repo in &repos {
        let spec = repo.join("spec");
        for entry in walk(&spec) {
            let Ok(text) = std::fs::read_to_string(&entry) else {
                continue;
            };
            let Some(ty) = frontmatter_type(&text) else {
                continue;
            };
            if !matches!(ty.as_str(), "FR" | "NFR" | "StR") {
                continue;
            }
            let Some(archetype) = registry.archetype(&ty) else {
                continue;
            };
            docs_total += 1;
            let result = quire_rs::validate_document_in_registry(&registry, archetype, &text);
            let mut hit = false;
            for w in result.warnings.iter() {
                if let Some(check) = w.message.strip_prefix("[quality:") {
                    let name = check.split(']').next().unwrap_or("?").to_string();
                    *per_check.entry(name.clone()).or_default() += 1;
                    hit = true;
                    let ex = examples.entry(name).or_default();
                    if ex.len() < 4 {
                        ex.push(format!(
                            "{}: {}",
                            entry.display(),
                            w.message.chars().take(150).collect::<String>()
                        ));
                    }
                }
            }
            if hit {
                docs_hit += 1
            }
        }
    }
    println!("repos scanned: {}", repos.len());
    println!("FR/NFR/StR documents: {docs_total}");
    println!(
        "documents with >=1 quality finding: {docs_hit} ({:.1}%)",
        100.0 * docs_hit as f64 / docs_total.max(1) as f64
    );
    for (check, n) in &per_check {
        println!("  quality:{check}: {n} findings");
    }
    println!("\n--- samples ---");
    for (check, ex) in &examples {
        println!("[{check}]");
        for e in ex {
            println!("   {e}");
        }
    }
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in rd.filter_map(|e| e.ok()) {
        let p = e.path();
        if p.is_dir() {
            out.extend(walk(&p))
        } else if p.extension().is_some_and(|x| x == "md") {
            out.push(p)
        }
    }
    out
}

fn frontmatter_type(text: &str) -> Option<String> {
    let rest = text.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    for line in rest[..end].lines() {
        if let Some(v) = line.strip_prefix("type:") {
            return Some(v.trim().trim_matches('"').to_string());
        }
    }
    None
}
