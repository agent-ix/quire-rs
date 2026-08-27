# Gap disposition census

- Date: `2026-08-27`
- CLI: `0.30.2`
- Engine: `9126463`
- Capabilities: `binding_census, binding_census.tagged, metrics_envelope, minted_targets, reference_only_targets, unmatched_tags, suspicions, specific_shaped`
- Module commit: `995288d609a47ab5a25f300ac0fa600d390b348c`
- Repositories: 241 scanned / 241 enumerated
- Exclusions: none

## Populations

| Population | Count |
|---|---:|
| `P1_evidence_symbols` | 25715 |
| `P2_tagged_symbols` | 8327 |
| `P3_authored_rows` | 26736 |
| `P4_minted_rows` | 23222 |

## Partition

| Outcome | Rows | Owner |
|---|---:|---|
| `backed` | 5566 | nobody |
| `instrument-unread` | 764 | engine |
| `declaration-unreached` | 231 | declaration-or-repository |
| `marker-form-mismatch` | 1011 | module-declaration |
| `id-class-unminted` | 3283 | module-declaration |
| `method-exempt` | 983 | nobody |
| `authoring-absent` | 14898 | repository |

Invariant: **PASS** — 26736 classified = 26736 authored rows.
Status-lie overlay: 1550 (not part of the sum).
Readable zero-tag repositories: 90.

## Actionable examples

| Outcome | Where | Why | Owner / next action |
|---|---|---|---|
| `instrument-unread` | `agent-cli-daemon/spec/functional/FR-001-run-command.md:19` `FR-001-AC-1`; source `typescript:tests/commands/run.test.ts:41` | the row has a tag on a language surface below the binding floor | engine — repair the named language binding surface, then rerun |
| `declaration-unreached` | `agent-duncan/spec/tests.md:122` `FR-001-CON-1` | the id class has an active target but this authored row was not minted | declaration-or-repository — repair the target archetype, section, or id-column declaration |
| `marker-form-mismatch` | `chat-markdown-renderer/spec/functional/FR-001-markdown-rendering.md:52` `FR-001-AC-1`; source `typescript:tests/ChatMarkdown.test.tsx:18` | the minted unbacked row has an authored id token no declared form bound | module-declaration — add or repair the declared tag form without widening unrelated forms |
| `id-class-unminted` | `agent-cli-daemon/spec/functional/FR-015-credential-storage.md:41` `FR-015-INV-1` | no active trace target mints this authored id class | module-declaration — decide and declare whether this id class is a trace target |
| `method-exempt` | `agent-config-loader/spec/functional/FR-001-loader-interface.md:49` `FR-001-CON-1` | the declared verification method mints no source symbol | nobody — none; the declared method does not mint a source symbol |
| `authoring-absent` | `agent-cli-daemon/spec/functional/FR-003-status-command.md:22` `FR-003-AC-4` | the row is minted and unbacked, with no authored source tag found | repository — add an applicable trace tag and its controlled corpus case |

## Highest owned backlogs

Rows owned by the engine, a module declaration, or a repository; correctly exempt rows are excluded. The JSON report carries the same routing for every repository.

| Repository | Owned rows | Dominant disposition | First repair locus |
|---|---:|---|---|
| `filament-ide-rs` | 1247 | `marker-form-mismatch` | `spec/backend/functional/FR-096-backend-contract.md:80` `FR-096-AC-8`; source `rust:crates/filament-backend-api/tests/envelope.rs:347` |
| `identity` | 797 | `authoring-absent` | `spec/functional/FR-001-user-management.md:85` `FR-001-AC-1` |
| `ecaz` | 658 | `authoring-absent` | `spec/functional/common/FR-001-tqvector-data-type-registration.md:69` `FR-001-AC-1` |
| `auth-service` | 648 | `authoring-absent` | `spec/functional/FR-001-login.md:119` `FR-001-AC-1` |
| `agent-duncan` | 619 | `authoring-absent` | `spec/functional/FR-001-rest-api-invocation.md:65` `FR-001-AC-2` |
| `auth` | 556 | `authoring-absent` | `spec/functional/FR-001-auth-domain.md:129` `FR-001-AC-1` |
| `filament-core-service` | 511 | `authoring-absent` | `spec/functional/FR-001-filament-core-domain.md:172` `FR-001-AC-1` |
| `filament-parser-lib` | 424 | `authoring-absent` | `spec/functional/FR-002-ast-validation-engine.md:74` `FR-002-AC-1` |
| `ix-workflow-runner` | 423 | `authoring-absent` | `spec/functional/FR-001-run-workflow-command.md:76` `FR-001-AC-1` |
| `quire-rs` | 422 | `authoring-absent` | `spec/functional/FR-001-render-dispatch.md:56` `FR-001-AC-3` |
| `ix-cli` | 418 | `authoring-absent` | `spec/functional/core/FR-020-core-plugin-schema.md:99` `FR-020-AC-1` |
| `quire-rs-plain` | 388 | `authoring-absent` | `spec/functional/FR-001-render-dispatch.md:54` `FR-001-AC-1` |
| `user-admin-ui` | 388 | `authoring-absent` | `spec/functional/FR-001.md:60` `FR-001-CON-2` |
| `usul-code` | 371 | `authoring-absent` | `spec/functional/FR-001-project-ingestion.md:53` `FR-001-AC-1` |
| `quoin` | 362 | `authoring-absent` | `spec/functional/FR-003-print-usage-and-help.md:47` `FR-003-AC-1` |
| `filament-editor-app` | 334 | `authoring-absent` | `spec/functional/FR-001-filament-platform-domain.md:252` `FR-001-AC-1` |
| `permission-service` | 295 | `authoring-absent` | `spec/functional/FR-001-role-management.md:56` `FR-001-AC-1` |
| `workflow-execution` | 277 | `authoring-absent` | `spec/functional/FR-001-workflow-definition.md:34` `FR-001-CON-1` |
| `platform-test-kit` | 262 | `authoring-absent` | `spec/functional/FR-001_stack_management.md:47` `FR-001-AC-1` |
| `ticket-runner` | 261 | `marker-form-mismatch` | `spec/functional/FR-001-dispatch-rule-engine.md:84` `FR-001-AC-1`; source `typescript:tests/cli.test.ts:215` |
