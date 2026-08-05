//! Source-symbol extraction (FR-051).
//!
//! Given a source tree, return its symbols with **stable identities** and the
//! spans needed to bind them to spec trace ids. One extraction feeds three
//! consumers: the coverage rollup (FR-050), semantic review, and
//! knowledge-graph ingestion.
//!
//! Adapters work at **syntax level** — no build, no type resolution, no
//! dependency installation, and never any execution of the extracted code
//! (FR-051-CON-1). They are line-structural rather than full parsers: that is
//! what "syntax level" buys, and it keeps the extractor dependency-free and
//! deterministic.
//!
//! Identity is `(language, repo-relative path, qualified name, kind)` — never
//! line numbers, byte offsets, or formatting, so reformatting a file leaves
//! every id unchanged (FR-051-AC-2). The current line is carried as a
//! non-identity attribute, alongside the span the trace-tag binder reads.

pub mod python;
pub mod rust;
pub mod trace;
pub mod typescript;

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::traceability::SourceLanguage;

/// What a symbol is. `kind` is part of the identity: a `Container` named `Foo`
/// and a function named `Foo` in the same file are different symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolKind {
    /// A plain function or method.
    Function,
    /// A function the language's test convention marks as a test.
    TestFunction,
    /// A container: Rust `struct`/`enum`/`trait`/`mod`, a Python class or
    /// module, a TypeScript class or module.
    Container,
}

impl SymbolKind {
    /// Stable machine-readable label (part of the identity digest).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::TestFunction => "test_function",
            Self::Container => "container",
        }
    }
}

/// One extracted symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub language: SourceLanguage,
    /// Repo-relative path, `/`-separated so identities are platform-stable.
    pub path: String,
    /// Qualified path within the file (`outer::inner::name`, `Class.method`,
    /// or a test registration's title).
    pub qualified_name: String,
    pub kind: SymbolKind,
    /// 1-based declaration line — a **non-identity** attribute.
    pub line: usize,
    /// 1-based first line of the attached annotation block (attributes,
    /// decorators, or leading comments). Equals `line` when nothing precedes.
    pub leading_line: usize,
    /// 1-based last line of the symbol's body.
    pub end_line: usize,
    /// Qualified name of the containing symbol, when nested.
    pub container: Option<String>,
    /// Stable SHA-256 digest of the identity (FR-045 record-id convention).
    pub id: String,
}

impl Symbol {
    /// The identity digest: language, path, qualified name, kind. Excludes
    /// every positional and formatting attribute (FR-051-AC-2).
    fn compute_id(
        language: SourceLanguage,
        path: &str,
        qualified_name: &str,
        kind: SymbolKind,
    ) -> String {
        stable_id(&[language.as_str(), path, qualified_name, kind.as_str()])
    }

    /// The source text attached to this symbol: its annotation block, its
    /// declaration, and its body. This is the span the trace-tag binder
    /// (FR-051 markers + legacy forms) matches against.
    pub fn attached_source<'s>(&self, source: &'s str) -> String {
        source
            .lines()
            .skip(self.leading_line.saturating_sub(1))
            .take(self.end_line.saturating_sub(self.leading_line) + 1)
            .collect::<Vec<&'s str>>()
            .join("\n")
    }
}

/// One source file the extractor read, retained so downstream trace binding
/// can slice symbol spans without re-reading the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub language: SourceLanguage,
    pub source: String,
}

/// A per-file problem. Extraction degrades per file: one unreadable or
/// unparseable file never aborts the tree (FR-051-CON-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolDiagnostic {
    pub path: String,
    pub reason: String,
}

/// The result of extracting a tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SymbolExtraction {
    /// Symbols, ordered by `(path, line, qualified_name)` — deterministic
    /// regardless of walk order (NFR-006).
    pub symbols: Vec<Symbol>,
    /// Files that yielded symbols, ordered by path.
    pub files: Vec<SourceFile>,
    /// Per-file diagnostics, ordered by path.
    pub diagnostics: Vec<SymbolDiagnostic>,
}

impl SymbolExtraction {
    /// The retained source of `path`, if the file was read.
    pub fn source_of(&self, path: &str) -> Option<&str> {
        self.files
            .iter()
            .find(|f| f.path == path)
            .map(|f| f.source.as_str())
    }
}

/// A symbol as an adapter produces it, before identity and path are attached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawSymbol {
    pub qualified_name: String,
    pub kind: SymbolKind,
    pub line: usize,
    pub leading_line: usize,
    pub end_line: usize,
    pub container: Option<String>,
}

/// The language an extension binds to, or `None` for a file the extractor
/// ignores. TypeScript covers `.ts`/`.tsx`; plain JavaScript is out of scope
/// (FR-051 names Rust, Python, and TypeScript).
pub fn language_of(path: &Path) -> Option<SourceLanguage> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => Some(SourceLanguage::Rust),
        Some("py") => Some(SourceLanguage::Python),
        Some("ts") | Some("tsx") => Some(SourceLanguage::Typescript),
        _ => None,
    }
}

/// Extract every symbol under `root`, walking gitignore-aware like the corpus
/// loader. `root` is the identity base: emitted paths are relative to it.
pub fn extract_tree(root: &Path) -> SymbolExtraction {
    let mut files: Vec<(String, SourceLanguage, std::path::PathBuf)> = Vec::new();
    for entry in ignore::WalkBuilder::new(root).build().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(language) = language_of(path) else {
            continue;
        };
        let rel = path.strip_prefix(root).unwrap_or(path);
        files.push((normalize_path(rel), language, path.to_path_buf()));
    }
    // Sort before reading so the output order never depends on walk order.
    files.sort();

    let mut out = SymbolExtraction::default();
    for (rel, language, abs) in files {
        match std::fs::read_to_string(&abs) {
            Ok(source) => out.extend_with_file(&rel, language, source),
            Err(e) => out.diagnostics.push(SymbolDiagnostic {
                path: rel,
                reason: format!("unreadable: {e}"),
            }),
        }
    }
    out
}

/// Extract one in-memory file — the unit-testable entry point, and what
/// [`extract_tree`] calls per file.
pub fn extract_file(path: &str, language: SourceLanguage, source: &str) -> SymbolExtraction {
    let mut out = SymbolExtraction::default();
    out.extend_with_file(path, language, source.to_string());
    out
}

impl SymbolExtraction {
    fn extend_with_file(&mut self, path: &str, language: SourceLanguage, source: String) {
        let parsed = match language {
            SourceLanguage::Rust => rust::parse(&source),
            SourceLanguage::Python => python::parse(path, &source),
            SourceLanguage::Typescript => typescript::parse(path, &source),
        };
        let raw = match parsed {
            Ok(raw) => raw,
            Err(reason) => {
                self.diagnostics.push(SymbolDiagnostic {
                    path: path.to_string(),
                    reason,
                });
                return;
            }
        };
        for r in raw {
            self.symbols.push(Symbol {
                language,
                path: path.to_string(),
                id: Symbol::compute_id(language, path, &r.qualified_name, r.kind),
                qualified_name: r.qualified_name,
                kind: r.kind,
                line: r.line,
                leading_line: r.leading_line,
                end_line: r.end_line,
                container: r.container,
            });
        }
        self.symbols.sort_by(|a, b| {
            (&a.path, a.line, &a.qualified_name).cmp(&(&b.path, b.line, &b.qualified_name))
        });
        self.files.push(SourceFile {
            path: path.to_string(),
            language,
            source,
        });
        self.files.sort_by(|a, b| a.path.cmp(&b.path));
    }
}

/// `/`-separated relative path, so identities do not vary by platform.
fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// NUL-separated SHA-256 over the identity parts — the FR-045 record-id
/// convention (mirrors `crate::filament`'s `stable_id`).
pub(crate) fn stable_id(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for (idx, part) in parts.iter().enumerate() {
        if idx > 0 {
            hasher.update([0]);
        }
        hasher.update(part.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// Shared adapter helper: the 1-based first line of the annotation block
/// attached to the declaration at `decl_idx` — the contiguous run of preceding
/// comment/attribute/decorator lines, skipping nothing else.
pub(crate) fn leading_block(
    lines: &[&str],
    decl_idx: usize,
    is_annotation: fn(&str) -> bool,
) -> usize {
    let mut start = decl_idx;
    while start > 0 {
        let candidate = lines[start - 1].trim();
        if candidate.is_empty() || !is_annotation(candidate) {
            break;
        }
        start -= 1;
    }
    start + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("symbols")
    }

    // TC-741 (FR-051-AC-1): each adapter extracts functions, test functions,
    // and containers from a fixture tree, and every symbol carries language,
    // repo-relative path, qualified path, kind, and a line attribute.
    #[test]
    fn tc741_adapters_extract_symbols_with_attributes() {
        let out = extract_tree(&fixture_root());
        assert!(
            out.diagnostics.iter().all(|d| d.path.contains("broken")),
            "only the deliberately broken fixture may diagnose: {:?}",
            out.diagnostics
        );

        for (path, language) in [
            ("rust/lib.rs", SourceLanguage::Rust),
            ("python/service.py", SourceLanguage::Python),
            ("typescript/service.ts", SourceLanguage::Typescript),
        ] {
            let file_symbols: Vec<&Symbol> =
                out.symbols.iter().filter(|s| s.path == path).collect();
            assert!(!file_symbols.is_empty(), "no symbols for {path}");
            assert!(file_symbols.iter().all(|s| s.language == language));
            assert!(file_symbols.iter().all(|s| !s.qualified_name.is_empty()));
            assert!(file_symbols.iter().all(|s| s.line >= 1));
            assert!(file_symbols.iter().all(|s| !s.id.is_empty()));
            for kind in [
                SymbolKind::Function,
                SymbolKind::TestFunction,
                SymbolKind::Container,
            ] {
                assert!(
                    file_symbols.iter().any(|s| s.kind == kind),
                    "{path} has no {kind:?} symbol: {:?}",
                    file_symbols
                        .iter()
                        .map(|s| (&s.qualified_name, s.kind))
                        .collect::<Vec<_>>()
                );
            }
        }

        // Ordering is deterministic and by (path, line).
        let keys: Vec<(&str, usize)> = out
            .symbols
            .iter()
            .map(|s| (s.path.as_str(), s.line))
            .collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
    }

    // TC-742 (FR-051-AC-2): reformatting changes no id; renaming changes only
    // the renamed symbol's id.
    #[test]
    fn tc742_identity_survives_reformatting() {
        let src = "fn alpha() {\n    let x = 1;\n}\n\nstruct Holder {\n    a: u8,\n}\n";
        let base = extract_file("src/lib.rs", SourceLanguage::Rust, src);

        // Reformat: blank lines, indentation, and every line number shift.
        let reformatted =
            "\n\n// a comment\nfn alpha()\n{\n        let x = 1;\n}\n\n\n\nstruct Holder {\n\n    a: u8,\n}\n";
        let after = extract_file("src/lib.rs", SourceLanguage::Rust, reformatted);
        let ids = |e: &SymbolExtraction| {
            let mut v: Vec<(String, String)> = e
                .symbols
                .iter()
                .map(|s| (s.qualified_name.clone(), s.id.clone()))
                .collect();
            v.sort();
            v
        };
        assert_eq!(ids(&base), ids(&after), "ids must survive reformatting");
        // …while the line attribute did move.
        let line_of = |e: &SymbolExtraction, name: &str| {
            e.symbols
                .iter()
                .find(|s| s.qualified_name == name)
                .unwrap()
                .line
        };
        assert_ne!(line_of(&base, "alpha"), line_of(&after, "alpha"));

        // Renaming one symbol changes only that id.
        let renamed = extract_file(
            "src/lib.rs",
            SourceLanguage::Rust,
            "fn beta() {\n    let x = 1;\n}\n\nstruct Holder {\n    a: u8,\n}\n",
        );
        let id_of = |e: &SymbolExtraction, name: &str| {
            e.symbols
                .iter()
                .find(|s| s.qualified_name == name)
                .map(|s| s.id.clone())
        };
        assert_eq!(id_of(&base, "Holder"), id_of(&renamed, "Holder"));
        assert!(id_of(&renamed, "alpha").is_none());
        assert_ne!(id_of(&base, "alpha"), id_of(&renamed, "beta"));
    }

    // TC-743 (FR-051-AC-3): each language's test convention classifies test
    // symbols; sibling non-test symbols stay plain functions.
    #[test]
    fn tc743_test_classification_per_language() {
        let out = extract_tree(&fixture_root());
        let kind_of = |name: &str| {
            out.symbols
                .iter()
                .find(|s| s.qualified_name == name)
                .unwrap_or_else(|| panic!("no symbol {name}"))
                .kind
        };

        // Rust: `#[test]`-family attributes.
        assert_eq!(kind_of("tests::tc741_extracts"), SymbolKind::TestFunction);
        assert_eq!(kind_of("tests::helper"), SymbolKind::Function);
        assert_eq!(kind_of("parse_config"), SymbolKind::Function);

        // Python: `test_`-prefixed functions and test-class methods.
        assert_eq!(kind_of("test_parses_config"), SymbolKind::TestFunction);
        assert_eq!(
            kind_of("TestService.test_rejects_empty"),
            SymbolKind::TestFunction
        );
        assert_eq!(kind_of("TestService.make_fixture"), SymbolKind::Function);
        assert_eq!(kind_of("parse_config"), SymbolKind::Function);

        // TypeScript: `test(...)` / `it(...)` registrations, titled.
        assert_eq!(kind_of("parses the config"), SymbolKind::TestFunction);
        assert_eq!(kind_of("rejects an empty config"), SymbolKind::TestFunction);
        assert_eq!(kind_of("parseConfig"), SymbolKind::Function);
    }

    // TC-749 (FR-051-AC-9, CON-2): an unparseable file yields a per-file
    // diagnostic while the rest of the tree extracts normally.
    #[test]
    fn tc749_unparseable_file_degrades_per_file() {
        let out = extract_tree(&fixture_root());
        let broken: Vec<&SymbolDiagnostic> = out
            .diagnostics
            .iter()
            .filter(|d| d.path.contains("broken"))
            .collect();
        assert_eq!(broken.len(), 1, "diagnostics: {:?}", out.diagnostics);
        assert!(!broken[0].reason.is_empty());
        // The broken file contributes no symbols…
        assert!(out.symbols.iter().all(|s| !s.path.contains("broken")));
        // …and its siblings extract normally.
        assert!(out.symbols.iter().any(|s| s.path == "rust/lib.rs"));
    }
}
