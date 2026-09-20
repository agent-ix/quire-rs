//! PLAT-840: pre-replacement baseline for the hand-rolled Rust symbol scanner
//! (`src/symbols/rust.rs`), before PLAT-843 reimplements it over tree-sitter.
//!
//! This is the before-number: once `rust.rs` is replaced there is no way to
//! reconstruct what the line-structural scanner used to see. It is run through
//! the library API against real, on-disk checkouts (never the installed
//! `quire` CLI, which drifts against `main`), and its output is written as a
//! durable JSON artifact under `reports/` so it can be diffed against the
//! post-replacement (PLAT-843) run.
//!
//! ```text
//! cargo run --release --example plat840_rust_baseline_sweep > /tmp/run.txt
//! ```
//!
//! Every repo is measured with the **same** archetype module —
//! `~/dev/spec-artifacts-process/spec_artifacts_process` — because that is the
//! module `quire-rs`'s own `Makefile` (`BENCH_MODULE` / `CENSUS_MODULE`)
//! already treats as the org-shared traceability declaration, and every
//! target repo's `spec/functional/FR-*.md` frontmatter (`type: FR`) matches
//! its archetypes. `Registry::traceability()` is still checked per run rather
//! than assumed: a repo whose registry does not resolve a model is reported as
//! "cannot compute coverage", never silently skipped.
//!
//! **The module's declared `source_exclude` globs are applied**
//! (`extract_tree_scoped`, not `extract_tree_excluding`) so the symbol and
//! coverage figures here match what `quire coverage --scope <repo> --json`
//! would report for the same repo — the ticket names that as the source for
//! the per-repo number, and a harness that silently counted excluded fixture
//! trees would not be measuring the same thing.
//!
//! The JSON this writes is deterministic byte-for-byte (no wall-clock, no
//! random, `BTreeMap`/sorted-vec ordering throughout) so a second run over the
//! same commits is a meaningful diff, not just a repeated assertion.
//!
//! **A baseline is only meaningful on `main`.** Each target's default path is
//! its local `~/dev/<repo>` checkout, but a local checkout is frequently on a
//! feature branch (this agent's sandbox refuses to `git checkout` a sibling
//! repo — see [`git_head`]'s doc comment — so it cannot switch one in place).
//! `PLAT840_PATH_<REPO>` (e.g. `PLAT840_PATH_QUIRE_CONTRACT_IR`) overrides a
//! single target's path; point it at a clean, disposable `main` clone (e.g.
//! `gh repo clone agent-ix/<repo> <dir> -- --depth 1 --branch main`) rather
//! than measuring the feature branch. The JSON's `on_main` field per repo,
//! and `all_repos_on_main` at the top, record whether this was done — a
//! `false` there means that repo's numbers are not a `main` baseline and must
//! not be compared against a post-PLAT-843 run.
//!
//! **Every declared target is recorded, whether or not it could be measured.**
//! A target whose path is missing or has no `spec/` directory still gets a
//! `repos[]` entry, with `measured: false` and a `skip_reason` — the exact
//! "quietly skips a repo and reports a smaller total" failure this program
//! exists to catch must not also be this program's own failure mode. The
//! process exits non-zero unless every declared target was measured
//! (`all_targets_measured` in the JSON), so a broken `PLAT840_PATH_*` override
//! cannot silently produce a short, plausible-looking report — `println!` a
//! shell command's exit code to see this: it is `$?` after this binary runs.
//!
//! **`quire-rs` itself is one of the six repos.** Merging this PR adds a new
//! `.rs` file (this one) with the tracking tags used in its own doc comments,
//! to the repo the baseline measures as `quire-rs`. Re-running this harness
//! after merge over `main` (rather than pinning `08d39ea2c...`, the commit
//! this artifact was measured at) will show a small delta for that reason
//! alone — expected, not a regression to chase.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quire_rs::coverage::{self, CoverageError};
use quire_rs::symbols::{self, trace};
use quire_rs::traceability::SourceLanguage;
use quire_rs::{Registry, Spec};
use serde::Serialize;

struct TargetRepo {
    name: &'static str,
    default_path: &'static str,
    /// The environment variable that overrides `default_path` — set this to a
    /// clean `main` checkout when the repo under `~/dev` sits on a feature
    /// branch (see the module doc comment). The baseline is only meaningful
    /// on `main`: a feature branch's uncommitted-to-`main` changes would be
    /// folded into a number nobody could reproduce once that branch merges
    /// or is deleted.
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

#[derive(Serialize)]
struct AbandonedFile {
    path: String,
    language: &'static str,
    reason: String,
    /// Which of PLAT-163's two named diagnostic shapes this is: a stray `}`
    /// with nothing open, or N blocks left open at EOF.
    reason_class: &'static str,
}

#[derive(Serialize)]
struct UnboundPointer {
    path: String,
    line: usize,
    symbol: String,
}

/// The row-addressable form of a generic id-shaped token that no declared
/// form bound (`SymbolGraph::unmatched_tags`, FR-050-AC-39, #362) — every
/// record, not a count, per PLAT-840's "trace tags bound vs. unmatched"
/// capture requirement.
#[derive(Serialize)]
struct UnmatchedTagRecord {
    trace_id: String,
    language: String,
    path: String,
    line: usize,
    symbol: String,
}

/// A trace id written on a symbol whose *kind* cannot bind it — a container
/// or a plain function, not the tagged-candidate population
/// (`SymbolGraph::non_binding_tags`, #312). Every record, not a count.
#[derive(Serialize)]
struct NonBindingTagRecord {
    path: String,
    symbol: String,
    kind: String,
    trace_id: String,
    form: String,
    line: usize,
}

#[derive(Serialize, Default)]
struct BindingCensusRow {
    language: String,
    candidates: usize,
    tagged: usize,
    bound: usize,
    /// `tagged - bound`: evidence symbols that carry an id-shaped token but
    /// minted no `verifies` relation. This, not `abandoned_files`, is where
    /// PLAT-843's real Rust-loss signal lives.
    tagged_not_bound: usize,
    /// One concrete `tagged_not_bound` symbol (`BindingCensus::unmatched_example`).
    /// Not the whole population: the library retains one example per
    /// language, not a per-symbol classification of every `tagged_not_bound`
    /// candidate — reconstructing that would mean re-implementing
    /// `trace::bind`'s internal tag-detection logic outside the library. (The
    /// **fully enumerable** populations this harness *does* dump in full are
    /// `unmatched_tags` and `non_binding_tags` below, per repo — a different,
    /// row-addressable slice of the same binding process.)
    #[serde(skip_serializing_if = "Option::is_none")]
    tagged_not_bound_example: Option<UnboundPointer>,
    /// One concrete example of a candidate that bound nothing at all
    /// (`BindingCensus::unbound_example`) — a superset of
    /// `tagged_not_bound_example` when `tagged == 0`.
    #[serde(skip_serializing_if = "Option::is_none")]
    unbound_example: Option<UnboundPointer>,
}

#[derive(Serialize)]
struct CoverageNumbers {
    computed: bool,
    reason_not_computed: Option<String>,
    unbacked_rows: usize,
    status_lies: usize,
    backed: usize,
    total: usize,
}

#[derive(Serialize)]
struct RepoReport {
    name: String,
    path: String,
    /// `false` means every other field below is a placeholder (empty/zero) —
    /// read `skip_reason`, not the numbers, when this is `false`.
    measured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip_reason: Option<String>,
    head_sha: String,
    head_ref: String,
    on_main: bool,
    symbols_by_language_kind: BTreeMap<String, BTreeMap<String, usize>>,
    rust_symbols_total: usize,
    abandoned_files: Vec<AbandonedFile>,
    abandoned_files_count: usize,
    other_diagnostics_count: usize,
    /// Source files the module's declared `source_exclude` globs removed
    /// from the walk before symbols were ever extracted from them
    /// (`SymbolExtraction::excluded_source_files`, FR-050-AC-24, #215) — the
    /// reason `abandoned_files`/`symbols_by_language_kind` here match `quire
    /// coverage`'s figures rather than counting fixture trees the declared
    /// model excludes.
    excluded_source_files: usize,
    binding_census: Vec<BindingCensusRow>,
    non_binding_tags: Vec<NonBindingTagRecord>,
    unmatched_tags: Vec<UnmatchedTagRecord>,
    coverage: CoverageNumbers,
}

impl RepoReport {
    fn unmeasured(name: &str, path: &str, reason: String) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            measured: false,
            skip_reason: Some(reason),
            head_sha: String::new(),
            head_ref: String::new(),
            on_main: false,
            symbols_by_language_kind: BTreeMap::new(),
            rust_symbols_total: 0,
            abandoned_files: Vec::new(),
            abandoned_files_count: 0,
            other_diagnostics_count: 0,
            excluded_source_files: 0,
            binding_census: Vec::new(),
            non_binding_tags: Vec::new(),
            unmatched_tags: Vec::new(),
            coverage: CoverageNumbers {
                computed: false,
                reason_not_computed: Some("repo not measured".to_string()),
                unbacked_rows: 0,
                status_lies: 0,
                backed: 0,
                total: 0,
            },
        }
    }
}

#[derive(Serialize)]
struct Baseline {
    quire_rs_measuring_commit: String,
    module_path: String,
    module_commit: String,
    /// The module's declared `traceability.source_exclude` globs, applied to
    /// every repo's walk via `extract_tree_scoped` — recorded so the JSON is
    /// self-describing about what was excluded and why.
    source_exclude_globs: Vec<String>,
    /// `true` only if every repo below has `on_main == true`. `false` means
    /// at least one repo's figures are not a `main` baseline — read each
    /// repo's `on_main` field to see which, and do not compare that repo's
    /// numbers against a post-PLAT-843 run unless the rerun pins the same
    /// non-main commit.
    all_repos_on_main: bool,
    /// `true` only if every declared `TARGETS` entry has `measured == true`.
    /// `false` means at least one repo could not be measured — the process
    /// also exits non-zero in that case, so this field is for a reader of the
    /// JSON alone, without the exit code.
    all_targets_measured: bool,
    repos: Vec<RepoReport>,
    rust_rollup_by_kind: BTreeMap<String, usize>,
    rust_symbols_total_all_repos: usize,
    abandoned_files_total_all_repos: usize,
    abandoned_files_total_rust_only: usize,
    unbacked_rows_total_all_repos: usize,
    status_lies_total_all_repos: usize,
}

fn main() {
    let quire_rs_root = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    let module_path = std::env::var("PLAT840_MODULE").unwrap_or_else(|_| {
        "/home/peter/dev/spec-artifacts-process/spec_artifacts_process".to_string()
    });
    let registry = Registry::load_module(Path::new(&module_path))
        .unwrap_or_else(|e| panic!("load module {module_path}: {e}"));
    // The module directory itself (`.../spec_artifacts_process`) is a
    // subdirectory of its git checkout, not the checkout root — walk up one
    // level to find `.git`.
    let module_repo_root = Path::new(&module_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(&module_path));
    let (module_sha, _module_ref) = git_head(&module_repo_root);

    let model = registry.traceability();
    let source_exclude_globs: Vec<String> =
        model.map(|m| m.source_exclude.clone()).unwrap_or_default();

    let mut repos = Vec::new();
    let mut rust_rollup: BTreeMap<String, usize> = BTreeMap::new();
    let mut abandoned_total = 0usize;
    let mut abandoned_rust_total = 0usize;
    let mut unbacked_total = 0usize;
    let mut lies_total = 0usize;

    for target in TARGETS {
        let path =
            std::env::var(target.path_env).unwrap_or_else(|_| target.default_path.to_string());
        let root = PathBuf::from(&path);
        if !root.join("spec").is_dir() {
            let reason = format!("no spec/ directory at {path}");
            eprintln!("SKIPPED {}: {reason}", target.name);
            repos.push(RepoReport::unmeasured(target.name, &path, reason));
            continue;
        }
        let (head_sha, head_ref) = git_head(&root);
        let on_main = head_ref == "refs/heads/main";
        if !on_main {
            eprintln!(
                "WARNING {}: measured at {} ({}), not refs/heads/main — this repo's numbers \
                 are NOT a main baseline and must not be compared against a post-PLAT-843 run \
                 unless that run uses the same non-main commit.",
                target.name, head_ref, head_sha
            );
        }

        let spec = Spec::from_path(&root.join("spec"));
        // `extract_tree_scoped`, not `extract_tree_excluding`: applies the
        // module's declared `source_exclude` globs so these figures match
        // what `quire coverage` reports for the same repo (F3).
        let extraction =
            symbols::extract_tree_scoped(&root, &[Path::new("spec")], &source_exclude_globs);

        let mut by_lang_kind: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        for sym in &extraction.symbols {
            *by_lang_kind
                .entry(sym.language.as_str().to_string())
                .or_default()
                .entry(sym.kind.as_str().to_string())
                .or_default() += 1;
            if sym.language == SourceLanguage::Rust {
                *rust_rollup
                    .entry(sym.kind.as_str().to_string())
                    .or_default() += 1;
            }
        }
        let rust_symbols_total = by_lang_kind
            .get("rust")
            .map(|m| m.values().sum())
            .unwrap_or(0);

        let mut abandoned_files: Vec<AbandonedFile> = extraction
            .diagnostics
            .iter()
            .filter(|d| d.reason.starts_with("unbalanced braces"))
            .map(|d| {
                let language = symbols::language_of(Path::new(&d.path))
                    .map(|l| l.as_str())
                    .unwrap_or("unknown");
                let reason_class = if d.reason.contains("closes no block") {
                    "closes_no_block"
                } else {
                    "blocks_left_open"
                };
                AbandonedFile {
                    path: d.path.clone(),
                    language,
                    reason: d.reason.clone(),
                    reason_class,
                }
            })
            .collect();
        abandoned_files.sort_by(|a, b| a.path.cmp(&b.path));
        let abandoned_files_count = abandoned_files.len();
        let abandoned_rust_here = abandoned_files
            .iter()
            .filter(|f| f.language == "rust")
            .count();
        abandoned_total += abandoned_files_count;
        abandoned_rust_total += abandoned_rust_here;
        let other_diagnostics_count = extraction.diagnostics.len() - abandoned_files_count;

        let (binding_census, non_binding_tags, unmatched_tags, coverage_numbers) = match model {
            None => (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                CoverageNumbers {
                    computed: false,
                    reason_not_computed: Some(
                        "registry.traceability() is None for this module".to_string(),
                    ),
                    unbacked_rows: 0,
                    status_lies: 0,
                    backed: 0,
                    total: 0,
                },
            ),
            Some(model) => {
                let graph = trace::bind(&extraction, model);
                let census = graph
                    .binding_census
                    .iter()
                    .map(|c| BindingCensusRow {
                        language: c.language.clone(),
                        candidates: c.candidates,
                        tagged: c.tagged,
                        bound: c.bound,
                        tagged_not_bound: c.tagged.saturating_sub(c.bound),
                        tagged_not_bound_example: c.unmatched_example.as_ref().map(|e| {
                            UnboundPointer {
                                path: e.path.clone(),
                                line: e.line,
                                symbol: e.symbol.clone(),
                            }
                        }),
                        unbound_example: c.unbound_example.as_ref().map(|e| UnboundPointer {
                            path: e.path.clone(),
                            line: e.line,
                            symbol: e.symbol.clone(),
                        }),
                    })
                    .collect();
                let mut non_binding: Vec<NonBindingTagRecord> = graph
                    .non_binding_tags
                    .iter()
                    .map(|t| NonBindingTagRecord {
                        path: t.path.clone(),
                        symbol: t.symbol.clone(),
                        kind: t.kind.to_string(),
                        trace_id: t.trace_id.clone(),
                        form: t.form.clone(),
                        line: t.line,
                    })
                    .collect();
                non_binding.sort_by(|a, b| (&a.path, a.line).cmp(&(&b.path, b.line)));
                let mut unmatched: Vec<UnmatchedTagRecord> = graph
                    .unmatched_tags
                    .iter()
                    .map(|t| UnmatchedTagRecord {
                        trace_id: t.trace_id.clone(),
                        language: t.language.clone(),
                        path: t.path.clone(),
                        line: t.line,
                        symbol: t.symbol.clone(),
                    })
                    .collect();
                unmatched.sort_by(|a, b| (&a.path, a.line).cmp(&(&b.path, b.line)));
                let cov = match coverage::compute(&spec, &registry, &graph, &root) {
                    Ok(report) => {
                        unbacked_total += report.unbacked_rows.len();
                        lies_total += report.status_lies.len();
                        CoverageNumbers {
                            computed: true,
                            reason_not_computed: None,
                            unbacked_rows: report.unbacked_rows.len(),
                            status_lies: report.status_lies.len(),
                            backed: report.totals.backed,
                            total: report.totals.total,
                        }
                    }
                    Err(CoverageError::ModelUndeclared) => CoverageNumbers {
                        computed: false,
                        reason_not_computed: Some(CoverageError::ModelUndeclared.to_string()),
                        unbacked_rows: 0,
                        status_lies: 0,
                        backed: 0,
                        total: 0,
                    },
                };
                (census, non_binding, unmatched, cov)
            }
        };

        repos.push(RepoReport {
            name: target.name.to_string(),
            path: path.clone(),
            measured: true,
            skip_reason: None,
            head_sha,
            head_ref,
            on_main,
            symbols_by_language_kind: by_lang_kind,
            rust_symbols_total,
            abandoned_files,
            abandoned_files_count,
            other_diagnostics_count,
            excluded_source_files: extraction.excluded_source_files,
            binding_census,
            non_binding_tags,
            unmatched_tags,
            coverage: coverage_numbers,
        });
    }

    let (quire_rs_sha, _quire_rs_ref) = git_head(&quire_rs_root);
    let all_repos_on_main = repos.iter().filter(|r| r.measured).all(|r| r.on_main);
    let all_targets_measured = repos.len() == TARGETS.len() && repos.iter().all(|r| r.measured);
    let baseline = Baseline {
        quire_rs_measuring_commit: quire_rs_sha,
        module_path,
        module_commit: module_sha,
        source_exclude_globs,
        all_repos_on_main,
        all_targets_measured,
        rust_symbols_total_all_repos: rust_rollup.values().sum(),
        rust_rollup_by_kind: rust_rollup,
        abandoned_files_total_all_repos: abandoned_total,
        abandoned_files_total_rust_only: abandoned_rust_total,
        unbacked_rows_total_all_repos: unbacked_total,
        status_lies_total_all_repos: lies_total,
        repos,
    };

    let json = serde_json::to_string_pretty(&baseline).expect("baseline serializes");
    if let Ok(out_path) = std::env::var("PLAT840_OUTPUT") {
        std::fs::write(&out_path, format!("{json}\n")).unwrap_or_else(|e| {
            panic!("write {out_path}: {e}");
        });
        eprintln!("wrote {out_path}");
    }
    println!("{json}");

    // F1: a partial sweep must never exit 0 — a rerun driven by a broken
    // PLAT840_PATH_* override, or a repo that vanished, must be loud, not a
    // smaller total that looks complete.
    if !all_targets_measured {
        eprintln!(
            "FATAL: only {}/{} declared targets were measured — see `skip_reason` in the JSON",
            baseline.repos.iter().filter(|r| r.measured).count(),
            TARGETS.len()
        );
        std::process::exit(1);
    }
}

/// Read the current commit sha and symbolic ref (e.g. `refs/heads/main`)
/// directly from `.git/HEAD` and the matching ref file — never a `git`
/// subprocess. This agent's sandbox refuses to run `git` against any checkout
/// other than its own worktree, so every commit identity in this sweep is
/// read as a plain file the way the rest of the harness reads source files.
fn git_head(repo: &Path) -> (String, String) {
    // `.git` is a directory in an ordinary checkout, but a file
    // (`gitdir: <path>`) in a linked worktree (this agent's own checkout is
    // one) — resolve to the real git-dir either way. `HEAD` is always
    // per-worktree; branch refs live in the *common* dir (`commondir`, a
    // relative path from the worktree git-dir) unless packed.
    let git_dir = resolve_git_dir(repo);
    let common_dir = std::fs::read_to_string(git_dir.join("commondir"))
        .ok()
        .map(|c| normalize(&git_dir.join(c.trim())))
        .unwrap_or_else(|| git_dir.clone());
    let head = match std::fs::read_to_string(git_dir.join("HEAD")) {
        Ok(s) => s.trim().to_string(),
        Err(_) => return ("sha unknown (HEAD unreadable)".to_string(), String::new()),
    };
    let Some(refname) = head.strip_prefix("ref: ") else {
        // Detached HEAD: the file already holds the sha.
        return (head, String::new());
    };
    if let Ok(sha) = std::fs::read_to_string(common_dir.join(refname)) {
        return (sha.trim().to_string(), refname.to_string());
    }
    if let Ok(packed) = std::fs::read_to_string(common_dir.join("packed-refs")) {
        if let Some(line) = packed.lines().find(|l| l.ends_with(refname)) {
            if let Some(sha) = line.split_whitespace().next() {
                return (sha.to_string(), refname.to_string());
            }
        }
    }
    ("sha unknown".to_string(), refname.to_string())
}

fn resolve_git_dir(repo: &Path) -> PathBuf {
    let dot_git = repo.join(".git");
    if dot_git.is_dir() {
        return dot_git;
    }
    if let Ok(contents) = std::fs::read_to_string(&dot_git) {
        if let Some(path) = contents.trim().strip_prefix("gitdir: ") {
            return PathBuf::from(path);
        }
    }
    dot_git
}

/// Collapse `..` components (`std::fs::canonicalize` would also resolve
/// symlinks, which is more than this needs and can fail on a path that
/// doesn't exist in every intermediate form).
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}
