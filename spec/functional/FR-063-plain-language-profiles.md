---
id: FR-063
title: "Plain-language profiles over reader-visible prose"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-036"
    type: "extends"
---
# FR-063: Plain-language profiles over reader-visible prose

## Description

`quire-rs` SHALL expose a source-located view of the prose a reader sees and
evaluate three bounded, advisory checks over it: `sentence-length`,
`heading-skip`, and `undefined-acronym`.

The view is part of the existing Markdown engine. It is not a second parser and
does not reconstruct a rendered document. It recognizes the authored block
shapes needed by these checks, removes metadata and code, joins visually wrapped
prose, and retains the first source line of every block. Quoin and other callers
consume the resulting facts instead of rescanning Markdown.

Thresholds and accepted acronyms are never engine policy. A caller supplies a
typed, named, versioned `PlainLanguageProfile` directly, or selects one from the
merged module registry. A profile is therefore attributable configuration, not
a claim that one universal threshold fits every audience.

The public names in this feature are project-owned. Neither a profile, finding,
nor clean report asserts conformance with an external publication.

## Inputs

- authored Markdown;
- a `PlainLanguageProfile` containing a profile version, optional document-type
  applicability, sentence-word limit, maximum heading-level step, and
  known-acronym vocabulary and intentional-uppercase exclusions;
- for a batch run, a bounded document root read by the FR-024 corpus loader.

## Outputs

- ordered `ReaderBlock` values carrying block kind, normalized reader text and
  1-based source line;
- warning-only `PlainLanguageFinding` values carrying rule id, path, line,
  message and excerpt;
- a serializable `PlainLanguageReport` carrying profile id/version, a stable
  configuration fingerprint, documents and blocks examined, findings, and
  explicitly skipped inputs.

## Behavior

Reader prose SHALL include headings, paragraphs, list items, block quotes,
alerts and table cells. Wrapped lines belonging to one prose block SHALL be
joined before sentence measurement. YAML frontmatter, fenced and indented code,
inline code, table delimiter rows, link destinations, markup delimiters and HTML
comments SHALL NOT contribute words or acronym candidates.

`sentence-length` SHALL report a sentence whose Unicode word-token count is
strictly greater than the profile limit. `heading-skip` SHALL report a heading
whose level increases by more than the profile's maximum step from the previous
reader-visible heading. `undefined-acronym` SHALL report the first use of an
uppercase acronym in a document unless it appears in the profile vocabulary or
is introduced in reader prose as an expansion followed by the acronym in
parentheses.

Every check SHALL run independently. Every finding SHALL carry
`LintSeverity::Warning`. No profile field promotes a finding to an error.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-063-CON-1 | Plain-language checks SHALL NOT change structural validation, requirement grammar, extraction, coverage, or writeback results. | Architecture | Test |
| FR-063-CON-2 | The engine SHALL carry no default profile and no global threshold. A missing profile means the caller has asked no plain-language question. | Architecture | Test |
| FR-063-CON-3 | Profile names, versions, thresholds and vocabulary SHALL contribute to the reported configuration fingerprint. | Data integrity | Test |
| FR-063-CON-4 | Findings SHALL remain advisory until a separately recorded corpus study establishes precision and a user approves promotion outside this engine surface. | Process | Inspection |
| FR-063-CON-5 | Rule ids and public feature names SHALL use project-owned names that make no external-standard conformance claim. | Licensing | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-063-AC-1 | The reader view excludes valid frontmatter, fenced code, indented code, inline code and HTML comments while preserving visible prose with 1-based document lines. | Test (TC-970) |
| FR-063-AC-2 | Nested list items, wrapped paragraphs, block quotes and alert bodies become independently located reader blocks without losing their visible text. | Test (TC-971) |
| FR-063-AC-3 | Table header/data cells are reader blocks, delimiter rows are not, and malformed Markdown returns a deterministic view or an explicit skipped-input record without a panic. | Test (TC-972) |
| FR-063-AC-4 | A sentence above the configured word limit reports `sentence-length`; one at and below the boundary does not, and code/link destinations do not inflate the count. | Test (TC-973) |
| FR-063-AC-5 | A heading increase above the configured step reports `heading-skip`; equal, descending and first-heading levels do not. | Test (TC-974) |
| FR-063-AC-6 | An unknown acronym reports `undefined-acronym` once at first use; a profile-known acronym, an inline definition and code-only text do not. | Test (TC-975) |
| FR-063-AC-7 | A module manifest loads a typed named/versioned profile, and malformed names, versions, zero thresholds or invalid acronym entries fail module load with an actionable reason. | Test (TC-976) |
| FR-063-AC-8 | Profiles merge first-wins by id and are available through a registry accessor; an undeclared id returns `None`, never an implicit default. | Test (TC-977) |
| FR-063-AC-9 | A batch report distinguishes zero findings over readable blocks from zero readable blocks and lists unreadable, non-document and prose-empty inputs with stable reason tokens. | Test (TC-978) |
| FR-063-AC-10 | Profile id/version and every effective threshold/vocabulary change alter the stable configuration fingerprint; repeated runs are byte-identical. | Test (TC-979) |
| FR-063-AC-11 | Every finding carries warning severity, path, 1-based line and excerpt; the three project-owned rule ids are the only ids this feature emits. | Test (TC-980) |
| FR-063-AC-12 | Enabling and running a profile leaves structural validation, requirement-grammar findings, extraction and a representative writeback result byte-identical. | Test (TC-981) |

## Dependencies

- **Upstream**: FR-005 (single parser boundary), FR-024 (bounded corpus read),
  and FR-036 (advisory lint posture).
- **Downstream**: a language-owning module may publish profiles after this
  contract releases; Quoin may adapt the report into review evidence without
  parsing Markdown itself.
