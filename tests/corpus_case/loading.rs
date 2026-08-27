//! Corpus declaration and expectation loading.

use std::path::{Path, PathBuf};

use super::{Case, CaseExpect, CaseMeta};

impl Case {
    pub fn input(&self) -> PathBuf {
        self.dir.join("input")
    }
}

/// Load every module under a module search path without mutating process-wide
/// environment state (ADR-0006; history in #360).
pub(super) fn load_module_path(path: &Path, case_id: &str) -> quire_rs::Registry {
    quire_rs::Registry::load_from(&[path])
        .unwrap_or_else(|e| panic!("{case_id}: module path {} failed: {e}", path.display()))
}

/// Deserialize a merged declaration, naming the directory on failure.
fn parse_meta(value: &serde_yaml::Value, dir: &Path) -> CaseMeta {
    serde_yaml::from_value(value.clone())
        .unwrap_or_else(|e| panic!("{}: case.yaml: {e}", dir.display()))
}

/// Set a string key on a mapping, for the fields a set derives per variant.
fn set_str(value: &mut serde_yaml::Value, key: &str, text: &str) {
    if let serde_yaml::Value::Mapping(map) = value {
        map.insert(
            serde_yaml::Value::String(key.to_string()),
            serde_yaml::Value::String(text.to_string()),
        );
    }
}

/// A case's expectations, from the directory holding its `input/`.
fn read_expect(dir: &Path) -> CaseExpect {
    serde_yaml::from_str(
        &std::fs::read_to_string(dir.join("expect.yaml"))
            .unwrap_or_else(|e| panic!("{}: expect.yaml: {e}", dir.display())),
    )
    .unwrap_or_else(|e| panic!("{}: expect.yaml: {e}", dir.display()))
}

/// A case's FORWARD expectations, if it declares any.
fn read_expect_pending(dir: &Path) -> Option<CaseExpect> {
    let path = dir.join("expect-pending.yaml");
    if !path.is_file() {
        return None;
    }
    Some(
        serde_yaml::from_str(
            &std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{}: expect-pending.yaml: {e}", dir.display())),
        )
        .unwrap_or_else(|e| panic!("{}: expect-pending.yaml: {e}", dir.display())),
    )
}

/// The pinned corpus submodule.
pub fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus")
}

/// Every case in the corpus, discovered by walking `cases/`.
///
/// A tree walk, not an `include_str!` of one hardcoded file: adding a case is
/// adding a directory and costs no `.rs` edit (FR-065 Behavior, #267 AC-3).
pub fn load_cases() -> Vec<Case> {
    let root = corpus_root().join("cases");
    assert!(
        root.is_dir(),
        "the corpus submodule is not checked out at {}. Run `git submodule update --init`.",
        root.display()
    );

    let mut cases = Vec::new();
    let mut modes: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("read cases/")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    // Sorted at both levels so a run's order is a property of the data rather
    // than of the filesystem (NFR-006).
    modes.sort();
    for mode in modes {
        // Reporting fixtures exercise Quoin over static MeasurementRecords.
        // They share the corpus ratchet, but are not Quire coverage inputs and
        // therefore do not belong in this runner's recall population.
        if mode.file_name().and_then(|name| name.to_str()) == Some("reporting") {
            continue;
        }
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(&mode)
            .expect("read mode dir")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.join("case.yaml").is_file())
            .collect();
        dirs.sort();
        for dir in dirs {
            // Parsed as a VALUE first, then merged, then deserialized. A
            // language SET splits its declaration across two files — the shared
            // `case.yaml` carries no `language`, because that is what varies —
            // so deserializing the shared file alone fails on a required field.
            // `bounds.py` merges the same way; two readers of one corpus
            // disagreeing about what a case IS is the drift FR-065 prevents.
            let shared: serde_yaml::Value = serde_yaml::from_str(
                &std::fs::read_to_string(dir.join("case.yaml")).expect("read case.yaml"),
            )
            .unwrap_or_else(|e| panic!("{}: case.yaml: {e}", dir.display()));

            // Two layouts. A single-language case carries `input/` beside its
            // `case.yaml`; a LANGUAGE SET carries one `<language>/` directory
            // per language, sharing the case-level declaration. #268 authors
            // sixteen modes across three languages, and three sibling
            // directories with unrelated ids is not a set.
            if dir.join("input").is_dir() {
                // A directory in BOTH layouts is an error, not a silent read
                // as one of them. Only the Python loader rejected it, so this
                // reader would have taken the `input/` branch and dropped a
                // half-migrated case's language variants without a word.
                let strays: Vec<String> = std::fs::read_dir(&dir)
                    .expect("read case dir")
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.join("input").is_dir())
                    .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
                    .collect();
                assert!(
                    strays.is_empty(),
                    "{}: carries both an `input/` and {strays:?} — a case is one \
                     layout or the other, and reading it as one silently drops \
                     the other",
                    dir.display(),
                );
                cases.push(Case {
                    expect: read_expect(&dir),
                    expect_pending: read_expect_pending(&dir),
                    meta: parse_meta(&shared, &dir),
                    dir,
                });
                continue;
            }

            let mut variants: Vec<PathBuf> = std::fs::read_dir(&dir)
                .expect("read case dir")
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.join("input").is_dir())
                .collect();
            variants.sort();
            assert!(
                !variants.is_empty(),
                "{}: neither an `input/` nor any `<language>/input/`. A                  half-authored fixture read as an absent one would make                  `gap_count` mean something else, so this is an error rather                  than a skip.",
                dir.display()
            );
            for variant in variants {
                let language = variant
                    .file_name()
                    .and_then(|n| n.to_str())
                    .expect("language directory name")
                    .to_string();
                let mut merged = shared.clone();
                if variant.join("case.yaml").is_file() {
                    let per: serde_yaml::Value = serde_yaml::from_str(
                        &std::fs::read_to_string(variant.join("case.yaml"))
                            .expect("read case.yaml"),
                    )
                    .unwrap_or_else(|e| panic!("{}: case.yaml: {e}", variant.display()));
                    if let (serde_yaml::Value::Mapping(b), serde_yaml::Value::Mapping(o)) =
                        (&mut merged, &per)
                    {
                        for (k, v) in o {
                            b.insert(k.clone(), v.clone());
                        }
                    }
                }
                let base = merged
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| dir.file_name().and_then(|n| n.to_str()).unwrap_or(""))
                    .to_string();
                if merged.get("case").is_none() {
                    set_str(&mut merged, "case", &base);
                }
                // The variant's id is its OWN. One id across three variants is
                // indistinguishable in a pending list, and a duplicate-id check
                // would call them one case.
                set_str(&mut merged, "id", &format!("{base}-{language}"));
                set_str(&mut merged, "language", &language);

                cases.push(Case {
                    expect: read_expect(&variant),
                    expect_pending: read_expect_pending(&variant),
                    meta: parse_meta(&merged, &variant),
                    dir: variant,
                });
            }
        }
    }
    cases
}
