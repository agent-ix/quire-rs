---
id: US-010
title: "LLM Extracts Structured Data for Retrieval / Grounding"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-016"
    type: "exercises"
---

## Story

As an **LLM agent** grounding an answer against a corpus (RAG over a spec, an architecture pack, or a process repo), I want to call `extract(doc, dsl)` with a body-extraction DSL declared in the archetype's manifest, so that I get back a compact `Vec<JsonMap>` of just the fields I asked for (e.g. "every endpoint", "every acceptance criterion", "every diagram tag") — not the whole markdown body — and can feed those records into the LLM's context window instead of the raw doc.

## Context

`extract()` is the lossy projection from `QuireDocument` to `Vec<Record>` where each record is a flat JSON map. The DSL lives in the archetype's manifest (`body_extraction:` field) and is shipped with the schema/template pair — so the *projection shape* travels with the archetype, not the consumer.

Two yield shapes (FR-011):

- **Single-yield (`match`)**: one record per document. Used for "give me this artifact's summary".
- **Multi-yield (`iterate_over`)**: one record per iteration unit (sub-heading, list item, table row). Used for "give me every X in this doc".

Fallback locator chains (FR-016) let the DSL survive legacy heading variants across the corpus.

The LLM never sees the DSL — the consumer (RAG pipeline, indexer, orchestrator) does. The LLM sees the records the DSL produced.

## Acceptance

- **US-010-AC-1**: A DSL with `iterate_over` against a parsed FR document emits one record per Acceptance Criterion under the AC section.
- **US-010-AC-2**: Each record contains exactly the fields named in `per_match`, no extras, no missing keys (unless `required: false`).
- **US-010-AC-3**: `extract()` is pure: identical `(doc, dsl)` inputs produce identical record sequences across runs and threads.
- **US-010-AC-4**: A document missing the iterate_over root section produces `records: []` + a `Diagnostic::IterateRootMissing` — never an error.
- **US-010-AC-5**: Fallback locators (FR-016) resolve via the second candidate when the first is absent, emitting `Diagnostic::FallbackLocatorUsed`.

## Efficiency Analysis

**Round trips per document:** 1 parse + 1 extract = 1 logical call (callers usually compose).

**LLM context cost** (input tokens):
- *Without extraction*: paste whole document body — typically 5–30 KB of markdown per artifact, much of it not relevant to the question.
- *With extraction*: 50–500 bytes per record × N records — dramatically smaller per doc, and the LLM can be given record samples instead of full body.
- A typical 100-artifact corpus: ~50 KB of extracted records vs ~2 MB of raw markdown. **~40× context savings** for the same grounding fidelity, before any vector-index reranking.

**Server-side cost** per extract call:
- Parse: O(doc size), one-shot, no I/O (FR-005..009).
- DSL evaluation: O(matches × locators) — bounded by the document's heading/list/table cardinality, not by doc size.
- Pre-validated at load time (FR-011-AC-6/7): malformed DSLs surface at `Registry::load_from`, not on every call.

**Comparison to "stuff whole doc into LLM":**
- Extraction is a single-pass tree walk vs the LLM doing its own implicit parse over tokens. Cheaper to do once at index time than at every query.
- Extraction is deterministic (NFR-006); LLM "implicit parsing" is not — small wording differences flip what the LLM thinks is "the AC section".
- Extraction loses fidelity by design — the consumer chose which fields to project. The trade-off is: cheaper context, narrower answers.

**Comparison to vector embeddings:**
- Extraction is *structured* projection. Embeddings are *semantic* projection.
- Extraction gives you "every AC verbatim"; embeddings give you "passages similar to the question".
- They compose: extract structured records, then embed each record. RAG pipelines that do this get the best of both — structural completeness + semantic search.

**When to use US-010:**
- Building a structured index (one record per AC, per endpoint, per diagram) for downstream LLM grounding.
- Producing tool-call inputs that need declarative provenance ("the AC at this block_id said X").
- Avoiding the "huge document in context" pattern.

**When NOT to use US-010:**
- Free-form Q&A where the LLM needs the whole document's narrative. Use parse + section() or just paste the markdown.
- Cases where the projection shape isn't known up-front. The DSL must be authored; it doesn't infer fields.

**Failure cost:** zero LLM cost on `IterateRootMissing` — the diagnostic is non-fatal. The consumer skips that doc or falls back to whole-body grounding.

## Performance Criteria

- **US-010-PC-1**: `parse_document` + `extract` on a 10 KB document with a multi-yield DSL emitting ~10 records completes in p50 < 2 ms (parse <1 ms per NFR-002 envelope at this size + extract <1 ms). Bench: **TC-453**.
- **US-010-PC-2**: DSL evaluation cost is O(matches × locators), bounded by document heading/list/table cardinality. Per-locator cost is one tree walk; no schema-validation overhead per record (records are typed as raw `serde_json::Map`, not validated).
- **US-010-PC-3**: For corpus-scale extraction (100 documents, ~10 records each = ~1,000 records), the parallel sweep completes in p50 < 200 ms on a single thread, < 50 ms on 8 threads (`Send + Sync` confirmed by NFR-006-AC-3). Bench: **TC-454**.
- **US-010-PC-4**: Memory per extract call: bounded by output `records.len() × record_size`. No retained intermediates from the parse tree (extract returns owned `Vec<JsonMap>`; the `QuireDocument` can be dropped after extract).
- **US-010-PC-5**: Determinism: identical (doc, dsl) → identical record sequence across runs and threads (NFR-006-AC-1, verified by TC-056 / TC-057 for the parse leg + dedicated proptest on the extract leg).
