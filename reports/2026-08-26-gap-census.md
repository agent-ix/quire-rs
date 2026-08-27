# Gap disposition census

- Date: `2026-08-26`
- CLI: `0.30.2`
- Engine: `7aaf19a`
- Capabilities: `binding_census, binding_census.tagged, metrics_envelope, minted_targets, reference_only_targets, unmatched_tags, suspicions, specific_shaped`
- Module commit: `737987b7131938203c2bda0f153f4bf15e8818bd`
- Repositories: 241 scanned / 241 enumerated
- Exclusions: none

## Populations

| Population | Count |
|---|---:|
| `P1_evidence_symbols` | 25707 |
| `P2_tagged_symbols` | 8320 |
| `P3_authored_rows` | 26726 |
| `P4_minted_rows` | 20097 |

## Partition

| Outcome | Rows | Owner |
|---|---:|---|
| `backed` | 5395 | nobody |
| `instrument-unread` | 733 | engine |
| `declaration-unreached` | 229 | declaration-or-repository |
| `marker-form-mismatch` | 883 | module-declaration |
| `id-class-unminted` | 6400 | module-declaration |
| `method-exempt` | 655 | nobody |
| `authoring-absent` | 12431 | repository |

Invariant: **PASS** — 26726 classified = 26726 authored rows.
Status-lie overlay: 1466 (not part of the sum).
Readable zero-tag repositories: 90.

## Actionable examples

| Outcome | Where | Why | Owner / next action |
|---|---|---|---|
| `instrument-unread` | `agent-cli-daemon/spec/functional/FR-001-run-command.md:19` `FR-001-AC-1`; source `typescript:tests/commands/run.test.ts:41` | the row has a tag on a language surface below the binding floor | engine — repair the named language binding surface, then rerun |
| `declaration-unreached` | `agent-duncan/spec/tests.md:632` `FR-006-AC-14` | the id class has an active target but this authored row was not minted | declaration-or-repository — repair the target archetype, section, or id-column declaration |
| `marker-form-mismatch` | `chat-markdown-renderer/spec/functional/FR-001-markdown-rendering.md:52` `FR-001-AC-1`; source `typescript:tests/ChatMarkdown.test.tsx:18` | the minted unbacked row has an authored id token no declared form bound | module-declaration — add or repair the declared tag form without widening unrelated forms |
| `id-class-unminted` | `agent-cli-daemon/spec/functional/FR-012-login-command.md:68` `FR-012-CON-1` | no active trace target mints this authored id class | module-declaration — decide and declare whether this id class is a trace target |
| `method-exempt` | `agent-config-models/spec/functional/FR-002-agent-config-model.md:40` `FR-002-AC-2` | the declared verification method mints no source symbol | nobody — none; the declared method does not mint a source symbol |
| `authoring-absent` | `agent-cli-daemon/spec/functional/FR-003-status-command.md:22` `FR-003-AC-4` | the row is minted and unbacked, with no authored source tag found | repository — add an applicable trace tag and its controlled corpus case |

## Highest owned backlogs

Rows owned by the engine, a module declaration, or a repository; correctly exempt rows are excluded. The JSON report carries the same routing for every repository.

| Repository | Owned rows | Dominant disposition | First repair locus |
|---|---:|---|---|
| `filament-ide-rs` | 1286 | `authoring-absent` | `spec/backend/functional/FR-096-backend-contract.md:73` `FR-096-AC-1` |
| `identity` | 840 | `authoring-absent` | `spec/functional/FR-001-user-management.md:85` `FR-001-AC-1` |
| `auth-service` | 696 | `id-class-unminted` | `spec/analysis/evidence-issue-21.md:23` `FND-001` |
| `ecaz` | 662 | `authoring-absent` | `spec/functional/common/FR-001-tqvector-data-type-registration.md:69` `FR-001-AC-1` |
| `agent-duncan` | 625 | `id-class-unminted` | `spec/functional/FR-001-rest-api-invocation.md:43` `FR-001-CON-1` |
| `auth` | 556 | `authoring-absent` | `spec/functional/FR-001-auth-domain.md:129` `FR-001-AC-1` |
| `filament-core-service` | 523 | `id-class-unminted` | `spec/functional/FR-001-filament-core-domain.md:152` `FR-001-CON-1` |
| `quire-rs` | 461 | `authoring-absent` | `spec/functional/FR-001-render-dispatch.md:56` `FR-001-AC-3` |
| `ix-cli` | 438 | `authoring-absent` | `spec/functional/core/FR-020-core-plugin-schema.md:99` `FR-020-AC-1` |
| `filament-parser-lib` | 437 | `authoring-absent` | `spec/functional/FR-002-ast-validation-engine.md:74` `FR-002-AC-1` |
| `quire-rs-plain` | 426 | `authoring-absent` | `spec/functional/FR-001-render-dispatch.md:54` `FR-001-AC-1` |
| `ix-workflow-runner` | 423 | `authoring-absent` | `spec/functional/FR-001-run-workflow-command.md:76` `FR-001-AC-1` |
| `user-admin-ui` | 395 | `authoring-absent` | `spec/functional/FR-002.md:59` `FR-002-AC-1` |
| `quoin` | 387 | `authoring-absent` | `spec/functional/FR-003-print-usage-and-help.md:47` `FR-003-AC-1` |
| `usul-code` | 371 | `authoring-absent` | `spec/functional/FR-001-project-ingestion.md:53` `FR-001-AC-1` |
| `filament-editor-app` | 334 | `id-class-unminted` | `spec/functional/FR-001-filament-platform-domain.md:232` `FR-001-CON-1` |
| `permission-service` | 295 | `authoring-absent` | `spec/functional/FR-001-role-management.md:56` `FR-001-AC-1` |
| `ticket-runner` | 286 | `marker-form-mismatch` | `spec/functional/FR-001-dispatch-rule-engine.md:84` `FR-001-AC-1`; source `typescript:tests/cli.test.ts:215` |
| `workflow-execution` | 278 | `authoring-absent` | `spec/functional/FR-003-parallel-execution.md:85` `FR-003-AC-1` |
| `platform-test-kit` | 270 | `id-class-unminted` | `spec/analysis_evaluation_feb_2026.md:307` `D-001` |
