# Code review — PLAT-1884, quire-rs PR #730

Scope: `git diff origin/main...HEAD`, repository `agent-ix/quire-rs`, branch `peter/plat-1884-embedded-anomaly-records`, head `18b534f235c0b760a455bcc8d3010a41d460b79b`. Changes are limited to FR-098, US-021, and `src/selftest.rs`.

## Summary

The public API returns a static slice of two `include_str!`-embedded Quire-RS anomaly records with stable repository-relative paths and source content. Ownership boundary is clear, docs describe the CLI follow-up, and Rust types follow the neighboring embedded-case API. No unsafe code, mutable state, or runtime file I/O is introduced. One medium test-oracle gap remains: the test cannot detect future anomaly-source additions omitted from the static array.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The fixed expected path list can remain green when a new `spec/anomalies/AN-*.md` source is added but not embedded. This leaves the “every source file” contract unverified for source-set growth. Consider generating the compile-time include list from owned anomaly files; keep tests from scanning/counting files per the spec-review checklist. | `src/selftest.rs:93-110,115-131`; `spec/functional/FR-098-embedded-anomaly-documents.md:27-29` |

## Checks

Coder-reported evidence: targeted rustfmt passed; `cargo check -p quire-rs --lib` passed; `cargo clippy -p quire-rs --lib --tests -- -D warnings` passed; `cargo test -p quire-rs --lib selftest::tests` passed 2/2. I inspected but did not rerun these focused checks. The computed `quire matrix --scope . --format json` exited 0 and reports FR-098-AC-1/2 tagged. It emits longstanding unrelated warnings about markers not read by the declared trace grammar. The optional `spec-objects-architecture` package-fetch warning was reported as unrelated by the coder; tests requiring the absent fixture package are skipped.

## Verdict

Request changes for FND-001.

## Workflow evidence

AP-201 (`spec/assurance/AP-201-detection-minting.md`) applies: this change adds trace-tagged acceptance tests, changing computed coverage, and AP-201 requires `spec-review`. Enforced review run: `75562612-bd85-4954-8935-d0e43a78567a`, definition `review` from quoin 0.31.1.

Reproduction on machine `terra`, repository `agent-ix/quire-rs`, PR #730, head `18b534f235c0b760a455bcc8d3010a41d460b79b`:

- `ix-flow --config-root /home/peter/.ix --state-dir /tmp/claude-1000/-home-peter-dev/85a971df-1008-451e-847e-425c442569c0/scratchpad/pr730-review/flows advance 75562612-bd85-4954-8935-d0e43a78567a analyses_run --json` — expected: invariant provider evaluates interview and advances to `analyses_run`; actual exit 0 (ix-flow JSON `ok:false`) / provider status 3, `invariant_provider_exit_nonzero`.
- `printf '{}' | quoin workflow-invariants --config-root /home/peter/.ix` — expected: protocol error JSON for malformed request; actual exit 1, stderr: `Error: an unknown command reached dispatch`.
- `quoin workflow-invariants` with a valid `quoin.workflow-invariants/v1` JSON request on stdin and `--config-root /home/peter/.ix` — expected: `quoin.workflow-invariants-result/v1` JSON; actual exit 1, stderr: `Error: an unknown command reached dispatch`.
- `quoin review --config-root /home/peter/.ix --path /home/peter/.codex/plugins/cache/agent-ix/quoin/0.31.1/skills/spec-review --target spec --id 75562612-bd85-4954-8935-d0e43a78567a --json` — expected: start/use enforced workflow; actual exit 1, stderr: `Error: EROFS: read-only file system, open '/home/peter/.ix/flows/instances/75562612-bd85-4954-8935-d0e43a78567a.json.tmp.28.1795624279042'`. The alternate ix-flow state dir is writable, so the first provider failure is the current blocker.

Resolved executables: `quoin` = `/home/peter/.npm-global/lib/node_modules/@agent-ix/quoin/bin/launcher.js`, version `0.36.8`; `ix-flow` = `/home/peter/.npm-global/lib/node_modules/@agent-ix/ix-flow/bin/ix-flow.js`, version `0.3.3`; `quire` = `/home/peter/.npm-global/lib/node_modules/@agent-ix/quire-cli/bin/launcher.js`, version `0.52.0 (engine 0.62.2)`. The machine release notice expects quoin 0.30.0, so the resolved Quoin differs from the announced chain.

The enforced workflow's final transition is `validated -> accepted` with a HITL gate. It requires an explicit human acknowledgment after all selected review artifacts validate. No gate is currently open and no acknowledgment token exists because the workflow cannot pass `intake -> analyses_run`; do not acknowledge.
