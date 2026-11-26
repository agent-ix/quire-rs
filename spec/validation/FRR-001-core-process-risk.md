---
id: FRR-001
title: "quire-rs supplier high process-risk ratings"
type: FeatureRiskRating
scope: supplier
rating: high_process_risk
status: proposed
relationships:
  - target: ix://agent-ix/quire-rs/IU-001
    type: uses_intended_use
  - target: ix://agent-ix/quire-rs/FR-002
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-026
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-032
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-033
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-034
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-036
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-038
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-039
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-040
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-042
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-047
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-049
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-050
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-051
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-058
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-059
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-060
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-062
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-069
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-077
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-078
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-084
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-085
    type: rates_feature
  - target: ix://agent-ix/quire-rs/FR-086
    type: rates_feature
---

# quire-rs supplier high process-risk ratings

## Rationale

Under IU-001, each feature below can itself produce a false verification, trace, discharge, or approval claim. These proposed feature ratings remain human decisions and are not calculated.

## Feature-specific reasons

| Feature | False-claim reason |
|---|---|
| FR-002 | schema validation can accept a document that violates its declared schema. |
| FR-026 | reference resolution can report a dangling or wrong-target reference as resolved. |
| FR-032 | document validation can return a clean result while a declared assertion fails. |
| FR-033 | locator assertions can accept content that violates its declared structural assertion. |
| FR-034 | field interpolation can compile the wrong constraint and thereby pass invalid content. |
| FR-036 | declarative linting can report a document clean when an enabled rule is violated. |
| FR-038 | OKF bundle validation can report an invalid bundle as valid. |
| FR-039 | unlinked-reference detection can omit a live unlinked reference and imply none remain. |
| FR-040 | object-edge vocabulary validation can accept an undeclared or invalid edge as valid. |
| FR-042 | EARS grammar checking can report a nonconforming requirement as conforming. |
| FR-047 | acceptance-criteria grammar checking can report malformed criteria as conforming. |
| FR-049 | verification-reference integrity can report an invalid verification binding as intact. |
| FR-050 | declarative coverage computation can report a requirement covered when no valid declared source supports it. |
| FR-051 | source-symbol extraction can create an incorrect test-to-criterion trace claim. |
| FR-058 | upward-trace completeness can report a required relation complete when an obliged target is missing. |
| FR-059 | declared-vocabulary coverage can report a required vocabulary value present when no requirement owns it. |
| FR-060 | vocabulary-reference assertions can accept a document whose declared value differs from its referenced vocabulary. |
| FR-062 | the `implements` relation can map a requirement to the wrong production symbol as if it were implemented. |
| FR-069 | semantic-module contract validation can accept a malformed or inconsistent contract at load. |
| FR-077 | trace search can return an incidental mention as a declared `verifies`/`implements` relation. |
| FR-078 | retained-evidence qualification can award coverage for an unperformed or mismatched verification act. |
| FR-084 | relation coverage can report a requirement's declared relation as verified when the supplied coverage evidence does not. |
| FR-085 | cross-repository resolution can report an edge resolved to the wrong repository target. |
| FR-086 | design-element joins can report a requirement realized by the wrong design element. |

## Assurance activities

| Activity | Acceptance criterion |
|---|---|
| Run the focused acceptance check for FR-002: inspect the focused fixture result and its diagnostics. | FR-002-AC-1 |
| Run the focused acceptance check for FR-026: inspect the focused fixture result and its diagnostics. | FR-026-AC-12 |
| Run the focused acceptance check for FR-032: inspect the focused fixture result and its diagnostics. | FR-032-AC-9 |
| Run the focused acceptance check for FR-033: inspect the focused fixture result and its diagnostics. | FR-033-AC-10 |
| Run the focused acceptance check for FR-034: inspect the focused fixture result and its diagnostics. | FR-034-AC-1 |
| Run the focused acceptance check for FR-036: inspect the focused fixture result and its diagnostics. | FR-036-AC-6 |
| Run the focused acceptance check for FR-038: inspect the focused fixture result and its diagnostics. | FR-038-AC-12 |
| Run the focused acceptance check for FR-039: inspect the focused fixture result and its diagnostics. | FR-039-AC-1 |
| Run the focused acceptance check for FR-040: inspect the focused fixture result and its diagnostics. | FR-040-AC-5 |
| Run the focused acceptance check for FR-042: inspect the focused fixture result and its diagnostics. | FR-042-AC-1 |
| Run the focused acceptance check for FR-047: inspect the focused fixture result and its diagnostics. | FR-047-AC-14 |
| Run the focused acceptance check for FR-049: inspect the focused fixture result and its diagnostics. | FR-049-AC-18 |
| Run the focused acceptance check for FR-050: inspect the focused fixture result and its diagnostics. | FR-050-AC-45 |
| Run the focused acceptance check for FR-051: inspect the focused fixture result and its diagnostics. | FR-051-AC-1 |
| Run the focused acceptance check for FR-058: inspect the focused fixture result and its diagnostics. | FR-058-AC-23 |
| Run the focused acceptance check for FR-059: inspect the focused fixture result and its diagnostics. | FR-059-AC-8 |
| Run the focused acceptance check for FR-060: inspect the focused fixture result and its diagnostics. | FR-060-AC-1 |
| Run the focused acceptance check for FR-062: inspect the focused fixture result and its diagnostics. | FR-062-AC-1 |
| Run the focused acceptance check for FR-069: inspect the focused fixture result and its diagnostics. | FR-069-AC-3 |
| Run the focused acceptance check for FR-077: inspect the focused fixture result and its diagnostics. | FR-077-AC-1 |
| Run the focused acceptance check for FR-078: inspect the focused fixture result and its diagnostics. | FR-078-AC-1 |
| Run the focused acceptance check for FR-084: inspect the focused fixture result and its diagnostics. | FR-084-AC-18 |
| Run the focused acceptance check for FR-085: inspect the focused fixture result and its diagnostics. | FR-085-AC-1 |
| Run the focused acceptance check for FR-086: inspect the focused fixture result and its diagnostics. | FR-086-AC-1 |
