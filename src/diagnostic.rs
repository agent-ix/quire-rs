//! Load-time + extract-time non-fatal diagnostics.
//!
//! Consumers inspect load warnings via the `&[Diagnostic]` slice
//! exposed on [`crate::registry::Registry::diagnostics`] and
//! [`crate::extract::ExtractionResult::diagnostics`].

use std::path::PathBuf;

/// One advisory note from a loader or extract call.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Diagnostic {
    DuplicateModuleName {
        name: String,
        paths: Vec<PathBuf>,
    },
    DuplicateArchetype {
        name: String,
        modules: Vec<String>,
    },
    ManifestMissingName {
        path: PathBuf,
        derived_name: String,
    },
    SearchPathNotADirectory {
        path: PathBuf,
    },
    SearchPathMissing {
        path: PathBuf,
    },
    SearchPathUnreadable {
        path: PathBuf,
        reason: String,
    },
    SymlinkLoop {
        path: PathBuf,
    },
    IterateRootMissing {
        path: Vec<String>,
    },
    FallbackLocatorUsed {
        key: String,
        position: usize,
        locator: String,
    },
    /// A document loaded by `load_repo` has no frontmatter `uuid`
    /// (CR-002). Non-fatal — the document still loads; its durable
    /// catalog id is simply absent until quire authors one.
    MissingUuid {
        path: PathBuf,
    },
    /// A file discovered by `load_repo` could not be read (missing,
    /// permission denied, or not valid UTF-8). Non-fatal: the file is
    /// skipped and the rest of the repo loads (FR-024-AC-2).
    DocumentUnreadable {
        path: PathBuf,
        reason: String,
    },
    /// Two or more loaded documents share the same artifact key
    /// (FR-025-AC-3). First occurrence wins for lookup; the duplicate
    /// is recorded here. Construction does not fail.
    DuplicateArtifactId {
        id: String,
        paths: Vec<PathBuf>,
    },
    /// A reference (frontmatter `relationships` entry or `ix://` body
    /// link) whose target id is absent from the loaded set (FR-026-AC-3).
    /// Non-fatal — resolution never reaches outside the corpus.
    DanglingReference {
        source: String,
        target: String,
        edge_type: String,
    },
    /// A loaded document has no frontmatter `type`
    /// (FR-027-AC-9). It is never returned by `by_type` and is reachable
    /// only via `by_id`. Non-fatal.
    UntypedArtifact {
        id: String,
        path: PathBuf,
    },
    /// An `edge_types` verb is declared with differing bodies by more
    /// than one module (FR-040-AC-2). First-wins; advisory, mirrors
    /// `DuplicateArchetype`. `load_strict` escalates.
    DuplicateEdgeType {
        name: String,
        modules: Vec<String>,
    },
    /// A `roles` name is declared with differing bodies by more than one
    /// module (FR-040-AC-2). First-wins; advisory. `load_strict`
    /// escalates.
    DuplicateRole {
        name: String,
        modules: Vec<String>,
    },
    /// A `lexicon` term is declared with differing bodies by more than one
    /// module (FR-043). First-wins; advisory, mirrors `DuplicateEdgeType`.
    /// `load_strict` escalates.
    DuplicateLexiconTerm {
        name: String,
        modules: Vec<String>,
    },
    /// A `grammar_severity` key is declared with differing levels by more
    /// than one module (FR-048-AC-2). First-wins; non-fatal, mirrors
    /// `DuplicateLexiconTerm`.
    DuplicateGrammarSeverity {
        name: String,
        modules: Vec<String>,
    },
    /// A `verification_catalog` method id is declared with differing bodies by
    /// more than one module (FR-054-AC-2). First-wins; non-fatal, mirrors
    /// `DuplicateLexiconTerm`.
    DuplicateVerificationMethod {
        name: String,
        modules: Vec<String>,
    },
    /// Two distinct forward `edge_types` verbs declare the **same**
    /// `inverse:` label (FR-041-AC-3). First-wins (the lexicographically
    /// first forward verb owns the label); advisory, mirrors
    /// `DuplicateEdgeType`. `load_strict` escalates.
    DuplicateInverseEdge {
        name: String,
        forwards: Vec<String>,
    },
    /// An archetype's `allowed_links` references a verb absent from the
    /// merged `edge_types` registry (FR-040-AC-3). Advisory; the
    /// vocabulary is open until declared. `load_strict` escalates.
    UnknownEdgeType {
        archetype: String,
        edge_type: String,
    },
    /// An archetype's `roles:` list, or an `allowed_links` target token,
    /// references a role absent from the merged `roles` registry
    /// (FR-040-AC-3). Advisory. `load_strict` escalates.
    UnknownRole {
        archetype: String,
        role: String,
    },
    /// A markdown file under the document root carries no usable
    /// frontmatter block, so it is not a document and contributes nothing
    /// to the corpus (CR-048, inverting CR-044's silent drop). Warning
    /// tier, non-fatal: the run's exit code is unchanged. Inside `spec/`
    /// this is almost certainly an authoring mistake — a draft that never
    /// got its front block, a malformed `---` fence, a note filed in the
    /// wrong place — and silence was a real error nobody ever saw.
    /// `malformed` distinguishes a complete fence block that is not a
    /// YAML mapping (a sharper finding) from an absent/unterminated block.
    DocumentWithoutFrontmatter {
        path: PathBuf,
        malformed: bool,
    },
    /// A caller-supplied path argument failed a path-safety check.
    ///
    /// Surfaced by consumers (CLIs, services) that resolve user-controlled
    /// path strings before handing them to the loader / extractor. The
    /// engine itself does not currently emit this variant, but exposing
    /// it here keeps per-consumer path-safety enums collapsed into a
    /// single `Diagnostic` channel rather than each downstream tool
    /// inventing a parallel `PathSafetyViolation` shape.
    PathTraversal {
        /// The caller-facing argument name (`--module`, `path`, etc.).
        argument: String,
        /// The offending path as supplied (pre-canonicalization).
        path: PathBuf,
        /// Why the path was rejected.
        reason: PathTraversalReason,
    },
}

/// Why a [`Diagnostic::PathTraversal`] was emitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathTraversalReason {
    /// The path contains one or more `..` segments after normalization.
    DotDotSegment,
    /// The path resolves through a symlink that points outside the
    /// allowed root.
    SymlinkEscape,
    /// The canonicalized path is not contained within the module root
    /// the caller declared.
    EscapesModuleRoot,
}

impl PathTraversalReason {
    /// Stable, machine-readable discriminator string used in JSON output.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DotDotSegment => "DotDotSegment",
            Self::SymlinkEscape => "SymlinkEscape",
            Self::EscapesModuleRoot => "EscapesModuleRoot",
        }
    }
}

impl std::fmt::Display for PathTraversalReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let human = match self {
            Self::DotDotSegment => "contains .. segment",
            Self::SymlinkEscape => "symlink escapes allowed root",
            Self::EscapesModuleRoot => "escapes declared module root",
        };
        f.write_str(human)
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateModuleName { name, paths } => write!(
                f,
                "DuplicateModuleName: '{}' declared at {} path(s); first-wins",
                name,
                paths.len()
            ),
            Self::DuplicateArchetype { name, modules } => write!(
                f,
                "DuplicateArchetype: '{}' contributed by modules {:?}; first-wins",
                name, modules
            ),
            Self::ManifestMissingName { path, derived_name } => write!(
                f,
                "ManifestMissingName at {}: using parent-dir name '{}'",
                path.display(),
                derived_name
            ),
            Self::SearchPathNotADirectory { path } => {
                write!(f, "SearchPathNotADirectory: {} is a file", path.display())
            }
            Self::SearchPathMissing { path } => {
                write!(f, "SearchPathMissing: {}", path.display())
            }
            Self::SearchPathUnreadable { path, reason } => {
                write!(f, "SearchPathUnreadable {}: {}", path.display(), reason)
            }
            Self::SymlinkLoop { path } => write!(f, "SymlinkLoop broken at {}", path.display()),
            Self::IterateRootMissing { path } => {
                write!(f, "IterateRootMissing: section_path {:?} not found", path)
            }
            Self::FallbackLocatorUsed {
                key,
                position,
                locator,
            } => write!(
                f,
                "FallbackLocatorUsed: key '{key}' resolved via fallback position {position} ({locator})"
            ),
            Self::MissingUuid { path } => {
                write!(f, "MissingUuid: {} has no frontmatter uuid", path.display())
            }
            Self::DocumentUnreadable { path, reason } => {
                write!(f, "DocumentUnreadable {}: {}", path.display(), reason)
            }
            Self::DuplicateArtifactId { id, paths } => write!(
                f,
                "DuplicateArtifactId: '{}' at {} path(s); first-wins",
                id,
                paths.len()
            ),
            Self::DanglingReference {
                source,
                target,
                edge_type,
            } => write!(
                f,
                "DanglingReference: {source} --{edge_type}--> {target} (target not in loaded set)"
            ),
            Self::UntypedArtifact { id, path } => write!(
                f,
                "UntypedArtifact: '{}' ({}) has no type field",
                id,
                path.display()
            ),
            Self::DuplicateEdgeType { name, modules } => write!(
                f,
                "DuplicateEdgeType: '{}' contributed by modules {:?}; first-wins",
                name, modules
            ),
            Self::DuplicateRole { name, modules } => write!(
                f,
                "DuplicateRole: '{}' contributed by modules {:?}; first-wins",
                name, modules
            ),
            Self::DuplicateLexiconTerm { name, modules } => write!(
                f,
                "DuplicateLexiconTerm: '{}' contributed by modules {:?}; first-wins",
                name, modules
            ),
            Self::DuplicateGrammarSeverity { name, modules } => write!(
                f,
                "DuplicateGrammarSeverity: '{}' contributed by modules {:?}; first-wins",
                name, modules
            ),
            Self::DuplicateVerificationMethod { name, modules } => write!(
                f,
                "DuplicateVerificationMethod: '{}' contributed by modules {:?}; first-wins",
                name, modules
            ),
            Self::DuplicateInverseEdge { name, forwards } => write!(
                f,
                "DuplicateInverseEdge: inverse label '{}' declared by verbs {:?}; first-wins",
                name, forwards
            ),
            Self::UnknownEdgeType {
                archetype,
                edge_type,
            } => write!(
                f,
                "UnknownEdgeType: archetype '{archetype}' allowed_links uses '{edge_type}' which is not in any edge_types registry"
            ),
            Self::UnknownRole { archetype, role } => write!(
                f,
                "UnknownRole: archetype '{archetype}' references role '{role}' which is not in any roles registry"
            ),
            Self::DocumentWithoutFrontmatter { path, malformed } => {
                if *malformed {
                    write!(
                        f,
                        "DocumentWithoutFrontmatter: {} has a frontmatter block that is not a YAML mapping — not a document, not loaded",
                        path.display()
                    )
                } else {
                    write!(
                        f,
                        "DocumentWithoutFrontmatter: {} is under the document root but has no frontmatter block — not a document, not loaded",
                        path.display()
                    )
                }
            }
            Self::PathTraversal {
                argument,
                path,
                reason,
            } => write!(
                f,
                "PathTraversal: argument '{argument}' rejected for {} ({reason})",
                path.display()
            ),
        }
    }
}

impl Diagnostic {
    /// Render this diagnostic as a JSON object suitable for structured
    /// logging or wire transport. Each object carries a `kind`
    /// discriminator plus the variant's payload.
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        match self {
            Self::DuplicateModuleName { name, paths } => json!({
                "kind": "DuplicateModuleName",
                "name": name,
                "paths": paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            }),
            Self::DuplicateArchetype { name, modules } => json!({
                "kind": "DuplicateArchetype",
                "name": name,
                "modules": modules,
            }),
            Self::ManifestMissingName { path, derived_name } => json!({
                "kind": "ManifestMissingName",
                "path": path.display().to_string(),
                "derived_name": derived_name,
            }),
            Self::SearchPathNotADirectory { path } => json!({
                "kind": "SearchPathNotADirectory",
                "path": path.display().to_string(),
            }),
            Self::SearchPathMissing { path } => json!({
                "kind": "SearchPathMissing",
                "path": path.display().to_string(),
            }),
            Self::SearchPathUnreadable { path, reason } => json!({
                "kind": "SearchPathUnreadable",
                "path": path.display().to_string(),
                "reason": reason,
            }),
            Self::SymlinkLoop { path } => json!({
                "kind": "SymlinkLoop",
                "path": path.display().to_string(),
            }),
            Self::IterateRootMissing { path } => json!({
                "kind": "IterateRootMissing",
                "path": path,
            }),
            Self::FallbackLocatorUsed {
                key,
                position,
                locator,
            } => json!({
                "kind": "FallbackLocatorUsed",
                "key": key,
                "position": position,
                "locator": locator,
            }),
            Self::MissingUuid { path } => json!({
                "kind": "MissingUuid",
                "path": path.display().to_string(),
            }),
            Self::DocumentUnreadable { path, reason } => json!({
                "kind": "DocumentUnreadable",
                "path": path.display().to_string(),
                "reason": reason,
            }),
            Self::DuplicateArtifactId { id, paths } => json!({
                "kind": "DuplicateArtifactId",
                "id": id,
                "paths": paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            }),
            Self::DanglingReference {
                source,
                target,
                edge_type,
            } => json!({
                "kind": "DanglingReference",
                "source": source,
                "target": target,
                "edge_type": edge_type,
            }),
            Self::UntypedArtifact { id, path } => json!({
                "kind": "UntypedArtifact",
                "id": id,
                "path": path.display().to_string(),
            }),
            Self::DuplicateEdgeType { name, modules } => json!({
                "kind": "DuplicateEdgeType",
                "name": name,
                "modules": modules,
            }),
            Self::DuplicateRole { name, modules } => json!({
                "kind": "DuplicateRole",
                "name": name,
                "modules": modules,
            }),
            Self::DuplicateLexiconTerm { name, modules } => json!({
                "kind": "DuplicateLexiconTerm",
                "name": name,
                "modules": modules,
            }),
            Self::DuplicateGrammarSeverity { name, modules } => json!({
                "kind": "DuplicateGrammarSeverity",
                "name": name,
                "modules": modules,
            }),
            Self::DuplicateVerificationMethod { name, modules } => json!({
                "kind": "DuplicateVerificationMethod",
                "name": name,
                "modules": modules,
            }),
            Self::DuplicateInverseEdge { name, forwards } => json!({
                "kind": "DuplicateInverseEdge",
                "name": name,
                "forwards": forwards,
            }),
            Self::UnknownEdgeType {
                archetype,
                edge_type,
            } => json!({
                "kind": "UnknownEdgeType",
                "archetype": archetype,
                "edge_type": edge_type,
            }),
            Self::UnknownRole { archetype, role } => json!({
                "kind": "UnknownRole",
                "archetype": archetype,
                "role": role,
            }),
            Self::DocumentWithoutFrontmatter { path, malformed } => json!({
                "kind": "DocumentWithoutFrontmatter",
                "path": path.display().to_string(),
                "malformed": malformed,
            }),
            Self::PathTraversal {
                argument,
                path,
                reason,
            } => json!({
                "kind": "PathTraversal",
                "argument": argument,
                "path": path.display().to_string(),
                "reason": reason.as_str(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_strings_carry_variant_name_and_identifier() {
        let cases: Vec<(Diagnostic, &[&str])> = vec![
            (
                Diagnostic::DuplicateModuleName {
                    name: "iso".into(),
                    paths: vec![PathBuf::from("/a"), PathBuf::from("/b")],
                },
                &["DuplicateModuleName", "iso"],
            ),
            (
                Diagnostic::DuplicateArchetype {
                    name: "fr".into(),
                    modules: vec!["mod-a".into(), "mod-b".into()],
                },
                &["DuplicateArchetype", "fr", "mod-a", "mod-b"],
            ),
            (
                Diagnostic::ManifestMissingName {
                    path: PathBuf::from("/x/manifest.yaml"),
                    derived_name: "x".into(),
                },
                &["ManifestMissingName", "/x/manifest.yaml", "x"],
            ),
            (
                Diagnostic::SearchPathNotADirectory {
                    path: PathBuf::from("/etc/passwd"),
                },
                &["SearchPathNotADirectory", "/etc/passwd"],
            ),
            (
                Diagnostic::SearchPathMissing {
                    path: PathBuf::from("/nope"),
                },
                &["SearchPathMissing", "/nope"],
            ),
            (
                Diagnostic::SearchPathUnreadable {
                    path: PathBuf::from("/locked"),
                    reason: "permission denied".into(),
                },
                &["SearchPathUnreadable", "/locked", "permission denied"],
            ),
            (
                Diagnostic::SymlinkLoop {
                    path: PathBuf::from("/loopy"),
                },
                &["SymlinkLoop", "/loopy"],
            ),
            (
                Diagnostic::IterateRootMissing {
                    path: vec!["Algorithms".into(), "Sort".into()],
                },
                &["IterateRootMissing", "Algorithms", "Sort"],
            ),
            (
                Diagnostic::FallbackLocatorUsed {
                    key: "k".into(),
                    position: 2,
                    locator: "heading".into(),
                },
                &["FallbackLocatorUsed", "k", "2", "heading"],
            ),
            (
                Diagnostic::MissingUuid {
                    path: PathBuf::from("/spec/functional/FR-099.md"),
                },
                &["MissingUuid", "/spec/functional/FR-099.md"],
            ),
            (
                Diagnostic::DocumentUnreadable {
                    path: PathBuf::from("/spec/bad.md"),
                    reason: "stream did not contain valid UTF-8".into(),
                },
                &["DocumentUnreadable", "/spec/bad.md", "valid UTF-8"],
            ),
            (
                Diagnostic::DuplicateArtifactId {
                    id: "FR-023".into(),
                    paths: vec![PathBuf::from("/a/FR-023.md"), PathBuf::from("/b/FR-023.md")],
                },
                &["DuplicateArtifactId", "FR-023"],
            ),
            (
                Diagnostic::DanglingReference {
                    source: "FR-023".into(),
                    target: "StR-099".into(),
                    edge_type: "implements".into(),
                },
                &["DanglingReference", "FR-023", "StR-099", "implements"],
            ),
            (
                Diagnostic::UntypedArtifact {
                    id: "X-1".into(),
                    path: PathBuf::from("/spec/x.md"),
                },
                &["UntypedArtifact", "X-1", "/spec/x.md"],
            ),
        ];
        for (d, needles) in cases {
            let s = d.to_string();
            for needle in needles {
                assert!(s.contains(needle), "{d:?} Display missing '{needle}': {s}");
            }
        }
    }

    #[test]
    fn diagnostic_satisfies_trait_bounds() {
        fn assert_full<T: Send + Sync + std::fmt::Debug + Clone + Eq>() {}
        assert_full::<Diagnostic>();
    }

    /// Path-safety violations route through the shared Diagnostic
    /// channel rather than a per-consumer parallel enum. Both the
    /// human (Display) and JSON renderings MUST carry the variant
    /// name, argument, path, and reason discriminator.
    #[test]
    fn path_traversal_variant_renders_human_and_json() {
        let d = Diagnostic::PathTraversal {
            argument: "--module".into(),
            path: PathBuf::from("/tmp/evil/../etc"),
            reason: PathTraversalReason::DotDotSegment,
        };
        let s = d.to_string();
        assert!(s.contains("PathTraversal"), "{s}");
        assert!(s.contains("--module"), "{s}");
        assert!(s.contains("/tmp/evil/../etc"), "{s}");
        assert!(s.contains("contains .. segment"), "{s}");

        let v = d.to_json();
        assert_eq!(v["kind"], "PathTraversal");
        assert_eq!(v["argument"], "--module");
        assert_eq!(v["path"], "/tmp/evil/../etc");
        assert_eq!(v["reason"], "DotDotSegment");

        // All reason discriminators round-trip stably.
        let escape = Diagnostic::PathTraversal {
            argument: "module".into(),
            path: PathBuf::from("/a"),
            reason: PathTraversalReason::SymlinkEscape,
        };
        assert_eq!(escape.to_json()["reason"], "SymlinkEscape");
        let escapes_root = Diagnostic::PathTraversal {
            argument: "module".into(),
            path: PathBuf::from("/b"),
            reason: PathTraversalReason::EscapesModuleRoot,
        };
        assert_eq!(escapes_root.to_json()["reason"], "EscapesModuleRoot");
    }
}
