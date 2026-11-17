---
id: SR-20261117110651
title: "Code review — PLAT-1769 quire-rs local schema refs (PR #700)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@e0024eca; src/loader/compile.rs, src/loader/flow_rules.rs, src/loader/mod.rs, src/semantic/resolver.rs, src/python/mod.rs, src/validate.rs, src/validate_document.rs, tests/it/flow_rules.rs, FR-096-AC-17/26/27/28"
review_set: subset
---

## Summary

Reviewed frozen PR #700 at `e0024eca574066d735da49f6c26e49a6ddc3ef8c`. The module-local resolver follows refs offline and the flow-rule loader uses referenced enums to validate `any_of`. The new tests cover scalar local refs, array-item refs, missing refs, and cycles, with trace tags. Two issues remain: flow report vocabularies still use the inline-only lookup, and the Rust clippy gate fails on the expanded `finish_compiled` signature.

## Verdict

**CONDITIONAL** — address the referenced-enum report gap and restore the clippy gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Flow reports omit enum vocabularies reached through local refs | src/corpus/flow_rules.rs:332 |
| FND-002 | medium | `finish_compiled` exceeds the clippy argument limit and fails `-D warnings` | src/loader/mod.rs:1228 |

## Coverage

- Rust review lane applied to resolver, loader, flow-rule validation, and test changes. Reviewed repository conventions in `AGENTS.md` and `CLAUDE.md`; no CI workflow changed.
- Resolver inspection: `frontmatter_schema_documents` uses `read_module_file` and the existing schema-document walker; external refs are refused without a network fetch. Loader stores resolved documents and schema path, and flow-rule validation reports rule/field context for reference failures.
- Test review: local scalar and array-item refs have tagged tests; missing and cyclic refs have a tagged test. See FND-001 and the gap-analysis artifact for uncovered assertions and report behavior.
- Supplied focused runs pass: `flow_rule_` 3/3, `flow_rules::` 17/17, and formatting check passes. Supplied confined `cargo clippy --test it -- -D warnings` fails on `finish_compiled` with `too_many_arguments (11/8)`.
- No tests were run during this review.

## Findings Detail

### FND-001 — Flow reports omit referenced enum vocabularies

`src/corpus/flow_rules.rs` still builds `FlowReport.vocabularies` with `vocabulary_coverage::enum_at(schema, field)`. That helper reads only inline `enum` properties and inline array-item enums; it does not follow `$ref`. Thus a rule can now be checked against the referenced enum during load, while the flow report omits that same vocabulary, contrary to FR-096-AC-17. Reuse the compiled archetype's resolved schema documents for report vocabulary extraction and add a report assertion for referenced enum members and order.

### FND-002 — Confined clippy gate fails

The new schema context parameters expand `finish_compiled` to 11 arguments. The supplied clippy log reports `clippy::too_many_arguments` at `src/loader/mod.rs:1228` and exits 101 under `-D warnings`. Bundle the related frontmatter schema fields or otherwise bring the function within the enforced lint, then rerun the required clippy lane.
