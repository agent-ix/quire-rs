---
id: FR-005
title: "parse_document API and QuireDocument Shape"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-002"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-003"
    type: "implements"
    cardinality: "1:1"
---

## Description

The crate SHALL export the two-tier parse surface (CR-046):

```rust
pub fn parse_header(markdown: &str) -> Option<Header>;    // cheap tier
pub fn parse_body(markdown: &str, header: &Header) -> QuireDocument; // expensive tier
pub fn parse_document(markdown: &str) -> QuireDocument;   // composes both

pub struct Header {
    pub id: String,            // frontmatter `id`; empty string when absent
    pub type_: Option<String>, // frontmatter `type`
    pub uuid: Option<Uuid>,    // frontmatter `uuid`; None when absent/unparseable
    pub frontmatter: serde_json::Map<String, serde_json::Value>, // the full mapping
    // + a private body offset so parse_body never re-extracts
}
```

- **`parse_header`** reads the front block only: one frontmatter extraction,
  no body work, no copy of the input. `None` means *not a document* — the
  CR-044 membership rule (`extract_frontmatter().frontmatter.is_some()`)
  stated as a type. Membership and identity are decided entirely here; the
  body contributes nothing to either.
- **`parse_body`** runs the body pipeline (headings, byte-exact section
  slices, tree assembly) under an already-parsed header. It is **total in its
  header**: `Header` is owned, so a caller can pair one with a string it did
  not come from, and that pair SHALL yield a document describing the string it
  was given rather than panicking (CR-050).
- **`parse_document`** composes the tiers with unchanged signature and
  semantics — every existing caller and the PyO3/wasm surfaces are untouched.

and the types:

```rust
pub struct QuireSection {
    pub id: String,
    pub heading: String,
    pub level: u8,
    pub content: String,
    pub children: Vec<QuireSection>,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct QuireDocument {
    pub preamble: Option<String>,
    pub sections: Vec<QuireSection>,
    pub raw: String,
    pub frontmatter: Option<serde_json::Map<String, serde_json::Value>>,
}
```

Both structs SHALL `derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)`.

`parse_document` SHALL:

1. Be pure — no IO, no panics, no global mutable state.
2. Accept arbitrary UTF-8 input including empty string.
3. For empty input, return `QuireDocument { preamble: None, sections: vec![], raw: "".into(), frontmatter: None }`.
4. Be re-entrant from any thread.

The purity clause (1) is a property of the whole exported surface, not of
`parse_document` alone: `parse_header` and `parse_body` SHALL likewise accept
arbitrary UTF-8 without panicking, for any argument pair (CR-050).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-005-AC-1 | `parse_document("")` returns the empty-document value above. | Test |
| FR-005-AC-2 | `parse_document("preamble only")` returns `QuireDocument { preamble: Some("preamble only"), sections: vec![], raw: "preamble only", frontmatter: None }`. | Test |
| FR-005-AC-3 | For the canonical TS fixture (`## Parent\nparent content\n### Child\nchild content\n## Sibling\nsibling content`), the returned document has 2 top-level sections; `Parent` has 1 child `Child`; section IDs are `parent-L0`, `child-L2`, `sibling-L4` (or the equivalent with the TS slug rule applied — see [FR-009](./FR-009-slug-line-id.md)). | Test |
| FR-005-AC-4 | A proptest checks `parse_document` does not panic on 10000 random UTF-8 strings. | Test |
| FR-005-AC-5 | `parse_header` returns `None` for a frontmatter-less, unterminated-fence, or non-mapping input without entering the body pipeline, and for a document returns `id` (empty string when absent), `type`, `uuid` and the full frontmatter mapping — identity read, never derived (CR-046). | Test (TC-812) |
| FR-005-AC-6 | `parse_body` under a `parse_header` header equals `parse_document` byte-for-byte — on named fixtures (BOM, CRLF, empty body, no headings) and on arbitrary UTF-8 input by proptest (CR-046). | Test (TC-813) |
| FR-005-AC-7 | `parse_body` is total in its header: for every pair of arbitrary UTF-8 inputs `(a, b)` where `parse_header(a)` is a document, `parse_body(b, &header)` returns a document whose `raw` is `b` — no panic when the header's body offset is past the end of `b` or inside one of its multi-byte characters (CR-050). | Test (TC-819) |

> **CR-046 note (2026-08-15):** `parse_document` previously fused the cheap
> frontmatter read with the expensive body parse — no caller could buy the
> discriminator alone, even though membership and identity are decided
> entirely by frontmatter (`id`/`type`/`uuid` are the only keys anything
> reads for them). Worse, CR-044 implemented the membership check by calling
> `extract_frontmatter` a *second* time after the full parse, so a
> non-document was read, line-split, heading-walked, section-built, wholly
> copied into `raw`, frontmatter-extracted twice, then discarded. The split
> makes the walk's membership/identity read one cheap operation
> (`walk::parse_one` now calls `parse_header`, retiring its `read_identity`
> and the duplicate `is_document` call), and is the enabler for the lazy body
> tier on `Spec` (FR-025, agent-ix/quire-rs#93). `Header` deliberately
> carries the **full** frontmatter mapping, not just the three identity keys:
> edge resolution (FR-026) and frontmatter validation read the map and must
> be able to do so without a body parse. `parse_document`'s signature,
> semantics and outputs are unchanged (agent-ix/quire-rs#92, umbrella #90).

> **CR-050 note (2026-08-15):** CR-046 split the parse surface but stated its
> purity clause against `parse_document` only, and `Header` carries a private
> byte offset into the input it was parsed from. Nothing binds the two: a
> `Header` is owned and stored *beside* an owned text (`LoadedDocument`), so it
> cannot borrow from its input, and `parse_body(other, &header)` is
> constructible from safe, public, PyO3/wasm-reachable API. It sliced that
> offset unchecked, so a mismatched pair panicked — out of bounds when the
> other string is shorter, char-boundary when the offset lands inside one of
> its multi-byte characters. AC-7 states the totality the doc comment already
> claimed; the implementation re-derives the offset from the string it was
> actually given, which costs one `is_char_boundary` on the correct path.
> A pair that happens to be in bounds and on a boundary is undetectable and
> stays the caller's contract — the guarantee is *no panic*, not *no misuse*
> (agent-ix/quire-rs#107, umbrella #106).

## Dependencies

- **Upstream**: [US-002](../usecase/US-002-developer-parses-spec-doc.md), [StR-003](../stakeholder/StR-003-parse-parity-with-quire.md)
- **Downstream**: [FR-006](./FR-006-frontmatter-with-fallback.md), [FR-007](./FR-007-fenced-block-heading-walk.md), [FR-008](./FR-008-byte-exact-slicing.md), [FR-009](./FR-009-slug-line-id.md) (parse internals that build the `QuireDocument`)
