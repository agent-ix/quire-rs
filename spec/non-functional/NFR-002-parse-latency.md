---
id: NFR-002
title: "Parse Latency: 5 MB Document Under 500 ms"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-002"
    type: "traces_to"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "depends_on"
    cardinality: "1:1"
---

## Statement

`quire_rs::parse_document(markdown)` SHALL parse a 5 MB markdown document in under **500 ms median** on the canonical baseline runner (Apple Silicon M2 Pro, release build).

For smaller documents the target is sublinear in input size up to the headings count — empirically O(n) in lines + O(n) in heading positions, no quadratic component.

### Cross-runner policy

Per-runner baselines stored separately (Apple Silicon M2 Pro canonical; Ubuntu x86_64 CI allowance +50%). Same regression-gate policy as [NFR-001](./NFR-001-render-latency.md).

## Rationale

Bulk extraction across the `spec-objects-business` corpus may parse thousands of documents in a single batch. A 5 MB ceiling captures the largest realistic spec/document size; 500 ms is the upper bound at which interactive paths (editor first-paint, live preview) remain responsive.

## Acceptance Criteria

- **NFR-002-AC-1**: A criterion benchmark `bench_parse_5mb` loads a synthetic 5 MB document (real markdown content with ~5000 headings) and measures `parse_document`; median below 500 ms.
- **NFR-002-AC-2**: A regression test compares against a stored baseline; >10% slowdown fails CI.
- **NFR-002-AC-3**: A correctness test on the same 5 MB document asserts the document round-trips: reconstructing the body from sections + preamble reproduces the input byte-for-byte (verifies [FR-008](../functional/FR-008-byte-exact-slicing.md) at scale).
- **NFR-002-AC-4**: A criterion benchmark `bench_validate_document` validates a typical authored artifact (under 32 KB, FR-sized: frontmatter + required sections + an AC table) against its archetype via `validate_document`; median below **1 ms** on the canonical baseline runner (warm `Registry`). A regression test compares against a stored baseline; >10% slowdown fails CI (same gate policy as NFR-002-AC-2).

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| `parse_document` latency for 5 MB doc (canonical M2 Pro) | < 500 ms median | 500 ms median | Criterion Benchmark |
| Parse latency on Ubuntu x86_64 CI runner | within +50% of canonical | < 750 ms median | Criterion Benchmark |
| `validate_document` latency for typical artifact (< 32 KB, warm Registry) | < 1 ms median | 1 ms median | Criterion Benchmark |
| Same-runner regression vs stored baseline | no slowdown | < 10% slowdown | Criterion Benchmark |

## Verification

- Criterion benches in `benches/parse.rs` execute on every PR with a stored baseline.
