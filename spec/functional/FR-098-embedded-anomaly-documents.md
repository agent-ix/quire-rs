---
id: FR-098
title: "Expose embedded Quire-RS anomaly documents"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-021-consume-quire-rs-anomaly-documents"
    type: "implements"
  - target: "ix://agent-ix/quoin/FR-217"
    type: "requires"
---
# FR-098: Expose embedded Quire-RS anomaly documents

## Description

The Quire-RS library SHALL expose its owned anomaly documents as embedded
resources for downstream validation-run consumers.

## Outputs

- An ordered collection of anomaly documents, each carrying its stable
  repository-relative path and exact UTF-8 source content.

## Behavior

- The library SHALL embed every Markdown anomaly record in `spec/anomalies/` at
  compile time.
- The self-test API SHALL return exactly those Quire-RS-owned records, ordered
  by repository-relative path, with the path preserved and source content
  unchanged.
- The returned collection SHALL contain no records owned by another tool.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-098-AC-1 | Given the embedded anomaly documents, the API returns one item for each `spec/anomalies/AN-*.md` source file, with stable repository-relative paths in ascending order and no other paths. | Test |
| FR-098-AC-2 | For each returned item, the content is byte-for-byte equal to the corresponding source file and its frontmatter ID matches the `AN-NNN` path segment. | Test |

## Dependencies

- **Upstream**: [US-021](../usecase/US-021-consume-quire-rs-anomaly-documents.md) and [Quoin FR-217](ix://agent-ix/quoin/FR-217).
- **Downstream**: Quire CLI embeds these resources in validation-run evidence.

## Change Records

### CR-001 — PLAT-1884 embedded anomaly document API

FR-217-AC-8 requires a Quire CLI validation run to include Quire-RS-owned
anomaly documents while excluding Quoin records. The existing Quire-RS
self-test API exposes case resources only. This requirement adds the
Quire-RS-owned resource contract; the Quire CLI remains responsible for
combining these documents with its own records in the retained run record.

| Intent | Existing requirement | Acceptance criterion | Test |
|--------|----------------------|----------------------|------|
| Let downstream self-test consumers retain the anomaly documents shipped with Quire-RS | [Quoin FR-217](ix://agent-ix/quoin/FR-217) | FR-098-AC-1 | A tagged unit test checks the exact sorted path set and ownership boundary. |
| Preserve document identity and source content for downstream retention | [Quoin FR-217](ix://agent-ix/quoin/FR-217) | FR-098-AC-2 | A tagged unit test checks each path ID against frontmatter and compares raw content with the embedded source. |
