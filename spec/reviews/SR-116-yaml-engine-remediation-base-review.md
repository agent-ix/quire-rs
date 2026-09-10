---
id: SR-116
title: "Base review of the YAML dependency-gate remediation"
type: SpecReview
analysis: base
scope: "NFR-009; ADR-0012; TC-330; TC-1820; SR-115 FND-001 remediation"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-009"
    type: reviews
---

## Summary

The base checklist remains satisfied without changing the accepted YAML-engine
decision. NFR-009 and ADR-0012 already require the selected package pair and
forbid the deprecated pair across the root and fuzz dependency surfaces; the
remediation adds the missing fuzz-graph evidence and no new requirement.

Every NFR-009 acceptance criterion retains a Test Matrix entry. The affected
static obligations remain TC-330 and TC-1820, and the new negative cases test
the existing forbidden-package boundary rather than introducing another ID or
coverage claim.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | Closed: the fuzz manifest's alias was verified while its resolved packages were not; full Cargo metadata now requires `yaml_serde 0.10.7` and `libyaml-rs 0.3.0` and rejects resolved `serde_yaml` or `unsafe-libyaml`, with two direct mutants. | `scripts/audits/check_dep_pins.sh:35`; `scripts/audits/check_dep_pins.sh:63`; `scripts/tests/test_dep_pins.py:112`; NFR-009-AC-3; NFR-009-AC-5 | correct-requirement-no-evidence |
