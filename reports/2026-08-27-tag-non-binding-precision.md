# `tag-on-non-binding-symbol` precision calibration

- Frame date: `2026-08-27`
- Candidate population: **1541**
- Deterministic sample: **110** (`agent-ix/quire-rs#355-v1`)
- Engine: `0.30.2` / `9126463`
- Decision: **retain-current-rule**

## Adjudication

| Stratum | Population | Sample | Authored tag | Prose citation | Other | Ambiguous | Unresolved | Precision interval |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `module-scope` | 976 | 30 | 30 | 0 | 0 | 0 | 0 | 100.0%–100.0% |
| `production-symbol` | 565 | 80 | 76 | 4 | 0 | 0 | 0 | 95.0%–95.0% |

Ambiguity is included in the upper precision bound and never excluded from the denominator. Population-weighted precision is **98.2%–98.2%**; the conservative stratified 95% sampling interval is **88.4%–99.3%**.

## Recall and locality

- Recall effect: No matcher change: controlled tag-on-non-binding-symbol recall remains 6/6 at L1, L2, and L3.
- Locality effect: No matcher change: controlled locality is unchanged; emitted lines remain symbol-leading loci and exact-occurrence locality is reported separately.
- Exact emitted-line locality in the sample: 4/110

## Sample rulings

| Candidate | Ruling | Rationale |
|---|---|---|
| `009709b202b57f44ef78` `filament-editor-gateway/filament_editor_gateway/discovery.py:227` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `027e33e514961946923e` `permission-service/permission_service/services/policy_service.py:143` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `04823c6f184014d30e39` `quire-rs-plain/src/loader/mod.rs:1061` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `04e1aeb3cc34b5b954ed` `workflow-worker-pool/workflow_worker_pool/domain/nodes.py:307` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `069a6aaa19bca2035943` `quire-rs/src/corpus/resolve.rs:252` | `prose-citation` | Both stored occurrences are parenthetical citations in explanatory Rustdoc; neither is an authored tag. |
| `083b7aa58edd101030d7` `quire-rs/src/loader/mod.rs:647` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `0940a44a20a5eb57ff9f` `ts-auth-ui/src/DeviceApprovalRoute.tsx:86` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `09bcf0ac5b9156c92af9` `agent-duncan/tests/unit/test_executor.py:3610` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `0c48d8e66dbe822cdabe` `rjsf-ix-theme/src/templates/BaseInputTemplate.tsx:10` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `0e5f9a597543c860473b` `quire-rs/src/loader/mod.rs:647` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `1174a6cdc915f46ba6f2` `agent-duncan/tests/unit/test_rotation.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `139e3399148bc9a989f8` `ix-cli/packages/local/src/commands/auth-create-user.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `1a17a60bc42164ac214b` `user-admin-ui/tests/FR008.test.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `1a3ed6ca17a46ffc2312` `filament-editor-app/services/filament-collab/src/server.ts:41` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `1bf215bd397af5bbb438` `ix-cli-core/src/config/service.ts:156` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `1cf034016e789c582518` `py-permissions/py_permissions/evaluation/hierarchy.py:40` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `1e8625601b41033366d1` `chat-markdown-renderer/src/ChatMarkdown.tsx:26` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `1efcc32119645b2529b6` `agent-duncan/tests/integration/test_it_007_config.py:44` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `21a94c6eb3e77dbc9bb6` `typesetter/src/components/CodeblockRegistry.tsx:27` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `2254d6f267b23f446f60` `ix-cli/apps/ix/src/commands/local/up.ts:49` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `25cd22fd9f36405be136` `ecaz/src/am/ec_distann/routine.rs:336` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `27fe49921bd4d14ae777` `scenario-runner/scenario_runner/services/executor.py:32` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `284790037dadca0d6283` `filament-editor-integration/tests/test_imports.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `28e9aa8635a59e908a08` `review-worker/review_worker/domain/validation_summary.py:213` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `29e74420d2455ff811a9` `quire-rs/src/registry.rs:210` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `2b52de4f05a4c9ff46c0` `gateway-bff-contract/tests/test_device_models.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `2fe1a2fc2ca09492d6d3` `ix-agent-memory-service/ix_agent_memory_service/api/routes.py:33` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `3046a2504b9b72284420` `quire/src/react/QuireProvider.tsx:59` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `31ed163fb1ff3de6c24d` `quire-cli/src/commands/coverage.rs:82` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `37bf38dcda6f6c327f53` `secrets-ref/secrets_ref/models.py:51` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `396d50deceb72c87ee4e` `ticket-runner/tests/cli.test.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `3b8edde64a5e3e036098` `filament-ui/src/sidebar/Sidebar.tsx:75` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `3dfd1fa1272acdcc40ef` `ix-agent-flow/tests/test_skill_emit.py:18` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `3e1444754c40b0623fc3` `filament-editor-gateway/filament_editor_gateway/discovery.py:227` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `3ee9023a71436df4eac2` `quire-rs-plain/src/symbols/trace.rs:536` | `prose-citation` | The only occurrence is inside a Rustdoc code example describing list grammar, not a tag on legacy_ids. |
| `40a42a54c13d08c1255d` `filament-view-ecosystem/src/components/LegendControls.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `45ccc6645502f14aa0f1` `filament-ui-shared/src/hooks/useArchetypeNav.ts:66` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `50c29426f0f193827dc3` `quire-wasm/tests/extract_validate.rs:85` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `53820843ae59930ec6a7` `pytest-results/pytest_results/grouping.py:31` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `54838964136fa3f899d6` `quire-rs-plain/src/loader/mod.rs:920` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `550b62f55b41f6246561` `workflow-worker-pool/workflow_worker_pool/domain/nodes.py:307` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `55e374327d46d50519cc` `quire/src/react/SectionTable.tsx:24` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `5759292edbb0b05116a5` `ix-agent-flow/tests/test_runtime.py:62` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `58e58177fc80225395d0` `user-admin-ui/src/ApprovalQueue.tsx:32` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `58ec412d46b32a6b6aa8` `quire/src/core/frontmatter.ts:62` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `599a52795a05c5c4fe2a` `ix-cli/packages/local/src/rollout.ts:622` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `6159baa2e732fd1c53ac` `filament-ide-rs/crates/filament-backend/src/backend.rs:820` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `61f889c9af34924a609a` `auth-py/auth_py/agent_auth.py:24` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `677a8ae1c57c906a1c85` `filament-view-ecosystem/src/components/EcosystemPane.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `67801daece5062f12793` `quire-rs/src/registry.rs:210` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `6b2b494bf585ec31bdc6` `review-worker/review_worker/domain/validation_summary.py:213` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `6e967b5fd108ca449a5b` `filament-view-review/src/components/ReviewActions.tsx:14` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `6edeae011e81c6985c12` `ix-cli/packages/local/tests/cluster-status.test.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `72879004aef6641563ff` `identity/tests/test_nfr008_change_password_timing.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `73e04d2eaa32dbb8a167` `auth-service/auth_service/services/auth_service.py:217` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `743708a7b2966409296b` `ts-build-chain/src/commands/build-chain.ts:494` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `74d4fc059dc232d5daf5` `quire-rs/src/python/mod.rs:144` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `74e205a42a6334f9c502` `catalog-service/catalog_service/services/repo_service.py:158` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `76a1c1adf9d5f4222bba` `filament-view-object/src/__tests__/client.test.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `7702e03aec18a54dbea0` `quire-rs/src/traceability.rs:829` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `7922bb8a2f3401e8f49d` `auth-service/auth_service/services/auth_service.py:217` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `79935ace12de12a2ed24` `py-state-machine/py_state_machine/engine.py:74` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `7e0eda774b1b427223fe` `code-diff-editor/src/ViewModeSelector.stories.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `88433ac2afc5feec7d35` `quire-rs/src/validate_document.rs:337` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `887381b7dda81adbd0d3` `workspace-worker/workspace_worker/handler.py:32` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `8b35013f505e88ec911e` `quire-rs/src/validate_document.rs:214` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `8dc0b8e7207a17f46b78` `ix-agent-browser-control/ix_agent_browser_control/services/actions.py:130` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `92d3522a9698d56362ee` `catalog-mcp-ui/src/index.tsx:12` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `9416a6490ef527c5972a` `ix-agent-extensions/ix_agent_extensions/delegation/manager.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `94eb83fd52bea704bfae` `ts-auth-ui/src/LoginDialog.tsx:14` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `9674bc4452526a4e8989` `quire-rs-plain/src/loader/mod.rs:920` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `96d6df141b106dd10742` `table-renderer/src/types.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `981e946b937ce420faab` `quire/src/react/QuireProvider.tsx:168` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `9858ef2ce47c659efbc7` `ix-agent-messaging-bridge/ix_agent_messaging_bridge/config.py:8` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `98c44c928305a9b50136` `ts-build-chain/src/utils/stable.ts:312` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `99612c8fbd8d5d575269` `ts-auth-ui/src/ProtectedRoute.tsx:20` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `9a508d2c3fd259d4e2ee` `cloudmanager-local-sync/tests/test_master_runner.py:114` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `a028c619b3d286871a79` `typesetter/tests/MarkdownGrammar.test.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `a186167ac46b2936884d` `qa-corpus/bounds.py:950` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `a43da53147b256e79495` `workflow-execution/workflow_execution/executor/engine.py:520` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `aa0b3283b0e0074e97cd` `ecaz/src/tests/ec_distann_physical_lifecycle.rs:3339` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `ab9d709f8ebbc10ca977` `cloudmanager-local-sync/tests/test_filament_core_client.py:286` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `b22d41fdee46cd53a449` `filament-ide-rs/crates/filament-service/src/facade.rs:535` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `b3373ee0c005d6279d69` `platform-test-kit/tests/integration/test_k8s_engine.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `b57521b4c5c068d7a7b7` `k8s-orchestration/tests/integration/test_k8s_interaction.py:174` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `b70c89b736b770e593f6` `filament-analysis-worker/filament_analysis_worker/domain/ingest.py:437` | `prose-citation` | The line is a wrapped parenthetical list continuation (TC-215/223/227)., not an authored tag. |
| `bbdfc26c6eb9e8832d00` `quoin/corpus/verify.py:633` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `bd92fff53d0bd18a015b` `typesetter/src/components/TypesetterEditor.tsx:144` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `c4af02d9ce8a93aef582` `ecaz/src/tests/ec_distann_basic.rs:2599` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `c99d76468b4d24350f1b` `secrets-injector-webhook/secrets_injector_webhook/parsing.py:13` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `cd7de609716cb2370698` `workflow-worker-pool/workflow_worker_pool/domain/nodes.py:307` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `d04214a82a242b67d04c` `ix-cli-core/src/secrets/service.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `d0dd16f4f366fecd9ac3` `quire-rs/src/loader/mod.rs:647` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `d4c896e54e42c02be312` `quire-rs-plain/src/corpus/resolve.rs:252` | `prose-citation` | Both stored occurrences are parenthetical citations in explanatory Rustdoc; neither is an authored tag. |
| `daa654d0c25c1533e396` `mermaid-renderer/src/MermaidRenderer.tsx:577` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `de0b4a4dc7016ed4d007` `ecaz/src/am/ec_distann/dml.rs:269` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e02fc05e9379c6ef3b34` `ticket-runner/tests/supervise.test.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e2906857599882e3d8ac` `ix-cli/packages/local/src/commands/list.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e563bfa7b6329405195d` `ix-agent-memory-service/ix_agent_memory_service/api/routes.py:52` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e6d0e545a21c82055f0d` `mermaid-renderer/src/MermaidRenderer.stories.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e8cc9648cb6b03f93c26` `filament-view-object/src/ObjectIndexPage.tsx:129` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e93b30faa7fa2ead8cf5` `identity/alembic/versions/a1b2c3d4e5f6_phase0_auth_gates.py:29` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `eb72c880e1008b24afd5` `filament-editor-integration/filament_integration/fixtures/modules.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `ec810676a0856f33515e` `auth-service/auth_service/services/auth_service.py:860` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `f122b2b78f33d50690ed` `identity/identity/api/bootstrap.py:219` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `f33f9691fcdb497d5c4b` `quire-rs-plain/src/coverage.rs:604` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `f8647f69b48ee9828993` `quire-rs-plain/src/extract/dsl.rs:148` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `f8a0aa19b6838d858dcb` `typesetter/src/hooks/useLineSelection.ts:44` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `fd5e2da10251d170bd12` `auth-service/auth_service/api/auth.py:205` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `feb70ecbe618e9e5d096` `ix-agent-flow/tests/test_decorators.py:45` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
