//! Compile-time resources for quire-rs-owned self-test cases.
//!
//! The quire CLI combines these resources with its own cases at build time;
//! the case files remain maintained in this repository (FR-047).

/// One file embedded for an owned self-test case.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct EmbeddedCaseFile {
    /// The file's normalized path relative to the case root, using `/` separators.
    pub path: &'static str,
    /// The exact file bytes embedded in the compiled library.
    pub bytes: &'static [u8],
}

/// A quire-rs-owned self-test case and its embedded files.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct EmbeddedCase {
    /// The stable case identifier, matching its directory under `selftest/cases/`.
    pub id: &'static str,
    /// The case files, addressed by paths relative to the case root.
    pub files: &'static [EmbeddedCaseFile],
}

static CASES: &[EmbeddedCase] = &[EmbeddedCase {
    id: "fr-032-placeholder-detection",
    files: &[
        EmbeddedCaseFile {
            path: "run.sh",
            bytes: include_bytes!("../selftest/cases/fr-032-placeholder-detection/run.sh"),
        },
        EmbeddedCaseFile {
            path: "required-tools",
            bytes: include_bytes!(
                "../selftest/cases/fr-032-placeholder-detection/required-tools"
            ),
        },
        EmbeddedCaseFile {
            path: "input/FR-001.md",
            bytes: include_bytes!(
                "../selftest/cases/fr-032-placeholder-detection/input/FR-001.md"
            ),
        },
        EmbeddedCaseFile {
            path: "input/module/manifest.yaml",
            bytes: include_bytes!(
                "../selftest/cases/fr-032-placeholder-detection/input/module/manifest.yaml"
            ),
        },
        EmbeddedCaseFile {
            path: "input/module/schemas/fr-frontmatter.schema.json",
            bytes: include_bytes!(
                "../selftest/cases/fr-032-placeholder-detection/input/module/schemas/fr-frontmatter.schema.json"
            ),
        },
        EmbeddedCaseFile {
            path: "expected/exit_code",
            bytes: include_bytes!(
                "../selftest/cases/fr-032-placeholder-detection/expected/exit_code"
            ),
        },
        EmbeddedCaseFile {
            path: "expected/stdout",
            bytes: include_bytes!(
                "../selftest/cases/fr-032-placeholder-detection/expected/stdout"
            ),
        },
    ],
}];

/// Returns every quire-rs-owned case embedded for use by the `quire` CLI.
#[must_use]
pub fn embedded_cases() -> &'static [EmbeddedCase] {
    CASES
}

/// One Quire-RS-owned anomaly document embedded for downstream validation runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct EmbeddedAnomalyDocument {
    /// The document's stable path relative to the repository root.
    pub path: &'static str,
    /// The exact UTF-8 source content embedded in the compiled library.
    pub content: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/embedded_anomalies.rs"));

/// Returns every Quire-RS-owned anomaly document embedded in this library.
#[must_use]
pub fn embedded_anomaly_documents() -> &'static [EmbeddedAnomalyDocument] {
    ANOMALY_DOCUMENTS
}

#[cfg(test)]
mod tests {
    use super::embedded_anomaly_documents;
    use ix_trace_rs::trace;
    use std::{fs, path::Path};

    mod anomaly_sources {
        include!("anomaly_sources.rs");
    }

    #[test]
    #[trace("FR-098-AC-1")]
    fn embedded_anomaly_documents_list_owned_paths_in_order() {
        let paths = embedded_anomaly_documents()
            .iter()
            .map(|document| document.path)
            .collect::<Vec<_>>();

        assert!(paths
            .iter()
            .all(|path| path.starts_with("spec/anomalies/AN-")));
        assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(paths.len(), embedded_anomaly_documents().len());
    }

    #[test]
    #[trace("FR-098-AC-1")]
    fn source_discovery_includes_every_anomaly_file_and_ignores_other_files() {
        let directory = tempfile::tempdir().expect("temporary anomaly directory");
        for name in [
            "AN-001-first.md",
            "AN-002-second.md",
            "notes.md",
            "AN-003-not-markdown.txt",
        ] {
            fs::write(directory.path().join(name), "fixture").expect("write fixture");
        }
        fs::create_dir(directory.path().join("AN-004-directory.md"))
            .expect("create directory named like an anomaly");

        let sources = anomaly_sources::anomaly_sources(directory.path())
            .expect("discover fixture anomaly documents");
        let names = sources
            .iter()
            .map(|source| source.file_name().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(names, ["AN-001-first.md", "AN-002-second.md"]);
    }

    #[test]
    #[trace("FR-098-AC-2")]
    fn embedded_anomaly_documents_preserve_source_content_and_ids() {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"));

        for document in embedded_anomaly_documents() {
            let source_path = repository_root.join(document.path);
            let source_content = fs::read_to_string(&source_path).unwrap_or_else(|error| {
                panic!("failed to read {}: {error}", source_path.display())
            });
            assert_eq!(document.content, source_content, "{}", document.path);

            let file_name = Path::new(document.path)
                .file_name()
                .and_then(|name| name.to_str())
                .expect("embedded anomaly path has a UTF-8 file name");
            let path_id = file_name.split('-').take(2).collect::<Vec<_>>().join("-");
            let content_id = document
                .content
                .lines()
                .find_map(|line| line.strip_prefix("id: "))
                .expect("anomaly frontmatter contains an id");

            assert_eq!(content_id, path_id, "{}", document.path);
        }
    }
}
