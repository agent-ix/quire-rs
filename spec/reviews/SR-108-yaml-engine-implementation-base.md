---
id: SR-108
title: "Base review of the maintained YAML engine migration"
type: SpecReview
analysis: base
scope: "ADR-0012; NFR-009; TC-1820..TC-1826 and TC-1828; issue #414"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-009"
    type: reviews
---

## Summary

This QUOIN base review checks the final #414 requirements and implementation.
The change preserves the existing Rust YAML API while replacing the archived
package with exact `yaml_serde 0.10.7`. The review intentionally rejects a
permanent dual-parser utility, per-input result archive, and exact call-site
registry as unnecessary product and governance surface.

AP-201 does not apply: its declared scope is Quire's detection, trace binding,
row minting, diagnostics, and published schemas. This change alters none of
those behaviors. The owner-selected base review is therefore the applicable
review set.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-4141 | high | Closed: the product graph used archived `serde_yaml 0.9.34+deprecated`; the manifest and lock now select exact `yaml_serde 0.10.7` and `libyaml-rs 0.3.0`. | `Cargo.toml`; `Cargo.lock`; TC-1820 | implementation-bug-despite-evidence |
| FND-4142 | medium | Closed: the dependency audit previously allowed the wrong YAML package or patch and did not reject the deprecated packages in the lock. Twelve policy mutations now cover selected and forbidden package identities while a field-order control passes. | `scripts/audits/check_dep_pins.sh`; `scripts/tests/test_dep_pins.py`; NFR-009-AC-3 | correct-requirement-no-evidence |
| FND-4143 | high | Closed: the first implementation made a one-time migration comparator, about 34,000 lines of raw results, and a 95-call census permanent repository machinery. The disposable tool, raw payload, and brittle registry were removed; only the decision and aggregate evidence remain. | ADR-0012; `spec/evidence/yaml-migration/manifest-v1.json` | wrong-requirement |

## Base checklist

- IDs and links: NFR-009 uses sequential AC identifiers; every AC has a test
  matrix entry and every referenced ADR exists.
- Scope: this is a dependency maintenance decision. It does not change parser
  behavior, public API, detector scope, or consumer ownership.
- Happy and negative cases: the selected lock graph passes; wrong manifest
  declarations and selected/forbidden lock-package mutations fail.
- Compatibility: a one-time old/new comparison covered 775 frontmatter blocks
  and six semantic-risk cases. All 781 values/outcomes matched; an injected
  difference produced one difference and a non-zero exit.
- Existing coverage: frontmatter parity and typed manifest, clause, extraction,
  traceability, and lint suites run unchanged. NFR-002 retains its existing
  performance thresholds.

## Evidence retained

| Gate | Result |
| --- | --- |
| dependency graph | exact `yaml_serde 0.10.7` and `libyaml-rs 0.3.0`; deprecated pair absent |
| differential | 781 of 781 outcomes and complete JSON values equal; negative control detected |
| parser parity | 89 tests passed unchanged at the recorded implementation revision |
| typed consumers | full locked suite passed unchanged at the recorded implementation revision |
| performance | −0.4%, +1.4%, +0.4%, and +1.4%; all within the existing 10% limit |
| supply chain | license gate passed and no advisory was reported at the recorded implementation revision |
| final cleanup gates | format and all-target/all-feature Clippy passed; full locked Rust suite passed; script suite 156 passed/3 skipped; fuzz bins compiled; cargo-deny passed |

The aggregate evidence and exact observed revisions are retained in
`spec/evidence/yaml-migration/manifest-v1.json`. A future package or version
decision requires fresh evidence; it does not reuse the removed comparator.

## Review disposition

Pass with no open specification finding. The implementation still requires the
repository's independent pull-request review and local code gates before merge.
