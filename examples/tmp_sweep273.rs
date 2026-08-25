use std::path::Path;

fn main() {
    let module = Path::new("/home/peter/dev/quire-rs/corpus/modules/ecosystem");
    let registry = quire_rs::Registry::load_from(&[module]).expect("load");
    let model = registry.traceability().expect("model");
    let dev = Path::new("/home/peter/dev");
    let mut repos: Vec<String> = std::fs::read_dir(dev)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| !n.starts_with('.') && dev.join(n).join("spec").is_dir())
        .collect();
    repos.sort();
    let (mut backed, mut total, mut untracked, mut moved) = (0usize, 0usize, 0usize, 0usize);
    for r in &repos {
        let root = dev.join(r);
        let spec = quire_rs::Spec::from_path(&root);
        let extraction = quire_rs::symbols::extract_tree(&root);
        let graph = quire_rs::symbols::trace::bind(&extraction, model);
        let Ok(rep) = quire_rs::coverage::compute(&spec, &registry, &graph, &root) else {
            continue;
        };
        if rep.totals.backed > 0 {
            moved += 1;
        }
        backed += rep.totals.backed;
        total += rep.totals.total;
        untracked += rep.untracked_symbols.len();
    }
    println!(
        "repos={} backed={backed} total={total} untracked={untracked} repos_with_backing={moved}",
        repos.len()
    );
}
