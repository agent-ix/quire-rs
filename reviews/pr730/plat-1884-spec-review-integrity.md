---
id: SR-202611251800000003
title: "integrity review of PLAT-1884 / quire-rs PR 730"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; PR 730 diff at 18b534f235c0b760a455bcc8d3010a41d460b79b; FR-098, US-021, src/selftest.rs"
review_set: subset
---

## Summary

Integrity review of FR-098, US-021, and the implementation/test evidence. The FR-098 outputs and behavior agree with the API shape. The exact two-source assertion checks the present list but does not establish the invariant for later source-set changes. The ID/content loop is consistent with AC-2. No contradictory identifiers or owner leakage were found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The fixed path assertion can stay green after a new `spec/anomalies/AN-*.md` source is added without adding it to `ANOMALY_DOCUMENTS`, so it does not verify the “one item for each source file” contract over future repository state. The embedded resource list then silently becomes incomplete. Consider deriving the compile-time embedded list from the owned anomaly source set and keeping tests free of directory scans, as the spec-review checklist prohibits tests that scan/count repository files. | `spec/functional/FR-098-embedded-anomaly-documents.md:27-29`; `src/selftest.rs:93-110,115-131` |

## Workflow evidence

AP-201 (`spec/assurance/AP-201-detection-minting.md`) applies: this change adds trace-tagged acceptance tests, changing computed coverage, and AP-201 requires `spec-review`. Enforced review run: `75562612-bd85-4954-8935-d0e43a78567a`, definition `review` from quoin 0.31.1.

Reproduction on machine `terra`, repository `agent-ix/quire-rs`, PR #730, head `18b534f235c0b760a455bcc8d3010a41d460b79b`:

- `ix-flow --config-root /home/peter/.ix --state-dir /tmp/claude-1000/-home-peter-dev/85a971df-1008-451e-847e-425c442569c0/scratchpad/pr730-review/flows advance 75562612-bd85-4954-8935-d0e43a78567a analyses_run --json` — expected: invariant provider evaluates interview and advances to `analyses_run`; actual exit 0 (ix-flow JSON `ok:false`) / provider status 3, `invariant_provider_exit_nonzero`.
- `printf '{}' | quoin workflow-invariants --config-root /home/peter/.ix` — expected: protocol error JSON for malformed request; actual exit 1, stderr: `Error: an unknown command reached dispatch`.
- `quoin workflow-invariants` with a valid `quoin.workflow-invariants/v1` JSON request on stdin and `--config-root /home/peter/.ix` — expected: `quoin.workflow-invariants-result/v1` JSON; actual exit 1, stderr: `Error: an unknown command reached dispatch`.
- `quoin review --config-root /home/peter/.ix --path /home/peter/.codex/plugins/cache/agent-ix/quoin/0.31.1/skills/spec-review --target spec --id 75562612-bd85-4954-8935-d0e43a78567a --json` — expected: start/use enforced workflow; actual exit 1, stderr: `Error: EROFS: read-only file system, open '/home/peter/.ix/flows/instances/75562612-bd85-4954-8935-d0e43a78567a.json.tmp.28.1795624279042'`. The alternate ix-flow state dir is writable, so the first provider failure is the current blocker.

Resolved executables: `quoin` = `/home/peter/.npm-global/lib/node_modules/@agent-ix/quoin/bin/launcher.js`, version `0.36.8`; `ix-flow` = `/home/peter/.npm-global/lib/node_modules/@agent-ix/ix-flow/bin/ix-flow.js`, version `0.3.3`; `quire` = `/home/peter/.npm-global/lib/node_modules/@agent-ix/quire-cli/bin/launcher.js`, version `0.52.0 (engine 0.62.2)`. The machine release notice expects quoin 0.30.0, so the resolved Quoin differs from the announced chain.

The enforced workflow's final transition is `validated -> accepted` with a HITL gate. It requires an explicit human acknowledgment after all selected review artifacts validate. No gate is currently open and no acknowledgment token exists because the workflow cannot pass `intake -> analyses_run`; do not acknowledge.



## Dispositions

Disposition pass 1 — rechecked PR #730 at head `4c3be945c3433821cf2d45efbc5bd6e2b902e362`.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The generated list now follows the owned anomaly source set through the shared selector; the previous duplicated two-path list is gone. The focused source-selection and embedded-content/ID tests pass 3/3; matrix AC-1/2 remain tagged. |

`after_excerpt` (fixed implementation evidence, `build.rs`):

```rust
let sources = anomaly_sources::anomaly_sources(&anomaly_dir)
    .unwrap_or_else(|error| panic!("failed to discover anomaly documents: {error}"));
```

Focused evidence: `cargo test -p quire-rs --lib selftest::tests` passed 3/3; `cargo clippy -p quire-rs --lib --tests -- -D warnings` passed; `quire matrix --scope . --format json` exited 0 and reports FR-098-AC-1/2 tagged. `quire validate --scope <repo> spec/functional/FR-098-embedded-anomaly-documents.md spec/usecase/US-021-consume-quire-rs-anomaly-documents.md 'reviews/pr730/*.md'` exits 0 after review-doc frontmatter is present. Optional architecture fixture package fetch warning recurred; dependent tests skip as documented.

AP-201 enforced review remains blocked at intake by the Quoin workflow invariant provider; final `validated -> accepted` still requires explicit human acknowledgment after artifact validation. No gate was acknowledged.
