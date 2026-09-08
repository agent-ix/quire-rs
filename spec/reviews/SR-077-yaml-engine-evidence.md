---
id: SR-077
title: "Evidence review of the YAML engine maintenance decision (#414)"
type: SpecReview
analysis: evidence
scope: "ADR-0012, NFR-009, TC-330..TC-332; AP-201 required review"
review_set: subset
---

## Summary

The read-only differential supports `yaml_serde` 0.10.2 as a candidate, but it
is not yet reproducible decision evidence and the implementation gates are not
minted as obligations. The deterministic advisor reports eight NFR-009 method
mismatches; judgement confirms that the existing generic method labels should
be replaced by named static and integration methods rather than copied from the
advisor mechanically.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-430 | high | ADR-0012 requires six implementation evidence groups, but NFR-009 and the matrix mint no obligations for the two-engine corpus differential, unchanged TypeScript/Python frontmatter parity, typed manifest/clause/DSL/traceability/lint parsing, Rust 1.75, license/advisory/unsafe gates, or exact Cargo resolution. Evidence reported only in PR prose cannot discharge an unminted requirement. | ADR-0012 (Required implementation evidence); NFR-009; spec/tests.md TC-330..TC-332 | missing-requirement |
| FND-431 | medium | `quoin advise --repo . --json` reports mismatches for NFR-009-AC-1..4 and M-1..M-4. AC-1/AC-3 are executable static policy checks and should use a declared Static-producing method such as `static-quality`; AC-4 is an unchanged parity-suite run and should use `integration-testing`. The advisor recommendations based only on generic `example`, `universal`, or `quantified-threshold` shapes are not adopted as verdicts. | NFR-009-AC-1..4; NFR-009-M-1..M-4 | wrong-requirement |
| FND-432 | medium | The spike records counts and short revisions for quire-rs, but does not identify the exact TypeScript Quire revision, executable comparator, complete input manifest/digests, raw result artifact, Quire engine/module identities, or commands needed to reproduce the 575 comparisons. AP-201 requires source/corpus revisions, producer identity, module digest, raw payload and population counts before a zero difference can support acceptance. | ADR-0012 (Differential decision evidence); AP-201 (Evidence Expectations) | correct-requirement-no-evidence |

## Advisor Output

At Quoin 0.23.1 and Quire 0.31.0, the repository-wide advisor emitted 820
obligation records. Filtering to NFR-009 produced eight records, eight
mismatches, zero inconclusive results and zero uncatalogued authored methods.
The deterministic recommendations were inspected by matched rule/value before
the judgements in FND-431 were made.

## Required Evidence Strategy

| Decision obligation | Method | Artifact | Gate |
| --- | --- | --- | --- |
| Exact selected package/version and deprecated-package absence | static-quality | `Cargo.toml`, `Cargo.lock`, `cargo tree`, strengthened dependency audit | `make audit-static` and full CI |
| Old/new parser outcome and value equality over governed inputs | integration-testing | versioned corpus manifest, focused semantic fixtures, raw differential result | dedicated differential test in full CI |
| TypeScript/Python frontmatter compatibility | integration-testing | existing parity fixtures, unchanged expected outputs | existing parity jobs in full CI |
| Typed Rust YAML consumers | integration-testing | manifest, clause, extraction DSL, traceability and lint fixtures | affected Rust suites in full CI |
| Rust 1.75 compatibility | compile | locked MSRV build record | MSRV gate |
| License, advisory and unsafe dependency posture | static-quality | cargo-deny/audit output and dependency tree | repository security/static gates |

The requirement and matrix must name these obligations before implementation;
the review does not treat the exploratory spike as their discharge.

## Dispositions after specification amendment

| ID | Disposition |
| --- | --- |
| FND-430 | Fixed in the candidate specification: NFR-009-AC-5..10 and TC-1820..TC-1825 name the exact resolution, differential, cross-language and typed-consumer parity, MSRV, license/advisory, unsafe, and static gates. Evidence remains pending by design. |
| FND-431 | Method vocabulary fixed: every authored method is catalogued, including `compile-time-check` and `sca-sbom`. The rerun produced 17 NFR-009 records, 16 mismatches, 0 inconclusive, and 0 uncatalogued. AC-9 matches `sca-sbom`; review judgement recommends retaining the static/integration methods on the other rows because the alternatives matched only generic `example`, `universal`, or numeric-threshold shapes. Owner acceptance of that judgement remains pending. |
| FND-432 | Open as evidence, resolved as a specification omission. ADR-0012 and NFR-009 now require exact source/corpus/module revisions, input digests, comparator/producer versions, raw output, counts, exclusions, and differences; the implementation revision must produce them. |

The deterministic advisor was rerun after the amendment. No advisor mismatch
is hidden; the residue and the reason for the recommended human disposition are
recorded above.
