---
id: SR-108
title: "Base review of the maintained YAML engine migration"
type: SpecReview
analysis: base
scope: "ADR-0012; NFR-009; TC-1820..TC-1831; issue #414 implementation at 397ed175bf5518a5d3f16befea7636b447cc4d97"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-009"
    type: reviews
---

## Summary

This QUOIN base review checks the #414 implementation against the accepted
ADR-0012 boundary and every NFR-009 migration criterion. The product and fuzz
manifests retain the `serde_yaml` import name while selecting exact
`yaml_serde 0.10.7`; the root lock selects `libyaml-rs 0.3.0` and contains
neither retired package. Differential, behavioral, build, supply-chain,
static, performance, and provenance gates pass with no accepted semantic
difference.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-4141 | high | Closed: the product graph used the archived `serde_yaml 0.9.34+deprecated` package and its `unsafe-libyaml` backend; the exact aliased dependency and root lock now select the maintained YAML Organization line. | `Cargo.toml`; `Cargo.lock`; TC-1820 | implementation-bug-despite-evidence |
| FND-4142 | medium | Closed: the inherited dependency audit rejected only wildcards, so a wrong package, caret YAML range, wrong patch, or weakened validator pin passed. Eight mutation tests now prove each governed failure class closes the gate. | `scripts/audits/check_dep_pins.sh`; `scripts/tests/test_dep_pins.py`; NFR-009-AC-3 | correct-requirement-no-evidence |
| FND-4143 | medium | Closed: the exploratory differential retained neither exact inputs nor raw results. The committed comparator now records producer and engine identities, five source revisions, every input digest and normalized value, exclusions, complete populations, the aggregate input-manifest digest, and a nonzero negative control. | `spec/evidence/yaml-migration/differential-v1.json`; `negative-control-v1.json`; TC-1821; TC-1827 | correct-requirement-no-evidence |
| FND-4144 | medium | Closed: stacking the YAML spec after #417 removed NFR-022 from its index while leaving the requirement file present. Restoring both catalog references makes `check_spec_structure.sh` pass and preserves the independently governed compiler policy. | `spec/non-functional/index.md`; `spec/spec.md`; NFR-022 | implementation-bug-despite-evidence |

## Differential and census evidence

| Gate | Result |
| --- | --- |
| engines | exact `serde_yaml 0.9.34` versus exact `yaml_serde 0.10.7`, using one `serde_json::Value` type |
| governed roots | quire-rs `397ed175bf5518a5d3f16befea7636b447cc4d97`; qa-corpus `7442f2770880a4ade303fb23d725804bdef454db`; TypeScript Quire `9446b283c0d0f882710670dca3bfd987a447d857`; process module `61a20e010d5e758f52864ad3152ccdb304a39d27`; ISO module `6686f112f2c38602c9d39c88e8134a945c34bbd6` |
| population | 804 Markdown documents; 775 complete leading frontmatter blocks; six focused semantic cases; 781 comparisons |
| result | zero outcome or complete-value differences; input manifest SHA-256 `6201f5aebf5d34f7ac85b6216e99d803837542ea9fea725ac450ef7c6a65a381` |
| negative control | injected duplicate-key difference exits 1 and records exactly one difference |
| raw identity | `differential-v1.json` SHA-256 `1196afdf7d39655042db8fae2584dc4a988676775b91c1d2c013f6d7e05c0502`; negative control SHA-256 `bab44fe29444e338167a12b7c4e3324d98418c81c0cb08da675addcb0966e9a9` |
| call-site census | 95 calls classified; three production calls cover frontmatter, module manifest, clause set, extraction DSL, traceability model, and lint rule; zero missing or unexpected class |

The standalone comparator has its own workspace and lock so the old engine is
available only for migration reproduction. It is absent from the root product,
development, fuzz, and lock graphs. The census explicitly excludes that audit
tool and mutation tests prove an unclassified product/fuzz/test call fails.

## Qualification evidence

| Criterion | Evidence | Result |
| --- | --- | --- |
| NFR-009-AC-1/3/5 | dependency audit, root lock inspection, and root `cargo tree` | pass; selected packages exact and retired pair absent |
| NFR-009-AC-6/12 | versioned comparator and retained raw evidence | pass; 781 of 781 equal, complete provenance retained |
| NFR-009-AC-7 | unchanged `tests/parser_parity.rs` suite | pass; 89 tests |
| NFR-009-AC-8/11 | full locked Rust 1.98.1 suite plus exact Python, WASM, fuzz-workspace, strict-doc, and all-feature release builds | pass; 609 library tests and every integration/doc suite; 37 WASM-feature semantic tests |
| NFR-009-AC-9 | locked `cargo deny check licenses`; freshly fetched RustSec database at `b50980aad8b8f14f77e25a97b32dd94bf008b0af` | pass; zero unwaived license or advisory finding; unused license allowances remain warnings |
| NFR-009-AC-10 | unsafe, property-purity, no-network, no-shellout, dependency, and all other static audits | pass; status agreement still reports 52 pre-existing advisory findings and is not represented as clean |
| NFR-009-AC-13 | same-runner Criterion baseline at `05e0e4d821888929c4a46d4b24ed95907e10e4f1` versus selected implementation | pass; changes −0.4%, +1.4%, +0.4%, and +1.4%, all below unchanged 10% threshold |
| NFR-009-AC-14 | governed census plus four mutation tests | pass; six of six production consumer classes represented |
| NFR-009-AC-15 | implementation diff and trace-form inspection | not applicable: no Rust test was added; Python mutation tests and the standalone comparator do not create a Rust acceptance-test symbol |

The all-in-one local `make ci` invocation passed through tests, licenses, and
static audits, then correctly refused ambient module checkouts at revisions
newer than its qualification lock. The unchanged validation command was rerun
against temporary detached worktrees at the exact locked revisions and passed:
182 documents, zero failures, 41 existing advisory warnings, and nine governed
Phase-7 exclusions. This review does not report the refused aggregate command
as a passing invocation.

## Review disposition

Pass with no open finding. ADR-0012 and TC-1820 through TC-1831 are satisfied
at the reviewed implementation revision. Merge remains gated on the upstream
#417 and #414 specification PRs and independent repository approval.
