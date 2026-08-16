// WHY: FR-025's lazy body tier (CR-047). The corpus parses headers eagerly at
// construction and materialises each document's body (`QuireDocument`) on
// first access, exactly once — concurrent first accessors receive the
// identical value. This cell is per-document state behind `Arc<SpecInner>`,
// NOT walk state: the FR-024 rayon fan-out never touches it (the walk builds
// every `LoadedDocument` with an empty cell and no body parse), so the
// walk-is-lock-free invariant (FR-024-AC-9) is untouched. It is the named
// audit exemption in `scripts/audits/check_no_shared_mutable.sh`; its
// exactly-once + agreed-value contract is proven by the NFR-017 loom model
// (TC-815) and raced for real under TSAN in `tests/corpus_concurrency.rs`
// (TC-816).

use crate::ast::QuireDocument;
use crate::parser::{parse_body, Header};

/// A once-init body cell: empty until the first [`get_or_parse`]
/// (`LazyBody::get_or_parse`), then holds the parsed body forever. The
/// underlying primitive is a std once-lock, whose `get_or_init` guarantees
/// the init closure runs at most once and every caller observes the one
/// stored value — external immutability holds: no query ever returns a
/// different answer twice (NFR-006).
pub(crate) struct LazyBody(std::sync::OnceLock<QuireDocument>);

impl LazyBody {
    /// An unparsed cell — the walk's shape (no body work at load, FR-025-AC-7).
    pub(crate) fn empty() -> Self {
        LazyBody(Default::default())
    }

    /// A pre-seeded cell — the [`from_parsed`](super::walk::LoadedDocument::from_parsed)
    /// shape, where the caller already holds the parsed document.
    pub(crate) fn seeded(doc: QuireDocument) -> Self {
        let cell = Self::empty();
        let _ = cell.0.set(doc);
        cell
    }

    /// The cached body, parsing it on first touch via
    /// [`parse_body`] under `header`. `text` must be the verbatim input the
    /// header came from (the same contract as `parse_body` itself).
    pub(crate) fn get_or_parse(&self, text: &str, header: &Header) -> &QuireDocument {
        self.0.get_or_init(|| parse_body(text, header))
    }

    /// The cached body, when one has been parsed or seeded.
    pub(crate) fn get(&self) -> Option<&QuireDocument> {
        self.0.get()
    }

    /// Whether the body tier has been materialised (test observability,
    /// TC-816/TC-817).
    pub(crate) fn is_parsed(&self) -> bool {
        self.0.get().is_some()
    }
}

/// Manual so the clone carries the cached value when one exists — a clone
/// never re-parses what its source already parsed, and an unparsed clone
/// stays unparsed.
impl Clone for LazyBody {
    fn clone(&self) -> Self {
        match self.0.get() {
            Some(doc) => Self::seeded(doc.clone()),
            None => Self::empty(),
        }
    }
}
