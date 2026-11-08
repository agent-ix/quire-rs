---
id: SR-316
title: "Base review of FR-050 trace-target extraction"
type: SpecReview
analysis: base
scope: "spec/functional/FR-050-declarative-coverage-computation.md"
review_set: subset
---

## Summary

The base checklist still records the corpus revision pin as a high P0, but it is out of PLAT-316 scope and tracked in P-PLAT-104 by PLAT-1307; no `filament-ide-rs` change is part of this ticket. This deferred finding does not block readiness of the AC-58 trace-source change. The earlier duplicate-AC finding is resolved: AC-45 remains the status-column criterion, and AC-58 has planned Test Matrix row TC-1991. `Test (TC-nnn)` is structured declaration input parsed by FR-050/FR-049, so preserve it for this change and resolve the generic checklist drift separately.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | P0: FR-050's implementation notes preserve a pinned corpus revision and treat moving off that pin as a refusal condition. This finding is deferred outside PLAT-316 to P-PLAT-104 and tracked by PLAT-1307. Do not modify `filament-ide-rs` for this ticket; this finding does not block readiness of the trace-source change. | FR-050 CR-103 implementation note (near lines 1116-1117); STD-228 implementation note (near lines 523-549); PLAT-1307 | wrong-requirement |
| FND-002 | low | The generic checklist says Verification cells contain a method only, but FR-050 and FR-049 explicitly define structured `Test (TC-nnn)` Verification input and parse its TC identifier. Preserve that annotation for this change; resolve the checklist drift separately rather than altering the requirements here. | FR-050 AC-1 through AC-58; FR-049 verification-reference contract | wrong-requirement |
