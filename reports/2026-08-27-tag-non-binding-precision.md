# `tag-on-non-binding-symbol` precision calibration

- Frame date: `2026-08-27`
- Candidate population: **1541**
- Deterministic sample: **110** (`agent-ix/quire-rs#355-v1`)
- Engine: `0.30.2` / `9126463`
- Module revision: `61a20e010d5e758f52864ad3152ccdb304a39d27`
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
| `0868c42702e363abcda3` `ix-agent-messaging-bridge/ix_agent_messaging_bridge/config.py:8` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `08d1cd5d8fc3ca606dd1` `quire-rs/src/validate_document.rs:337` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `09d0de101952c459849f` `ix-cli/packages/local/src/commands/auth-create-user.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `0a540180c8ea11f65d97` `quire-rs/src/corpus/resolve.rs:252` | `prose-citation` | Both stored occurrences are parenthetical citations in explanatory Rustdoc; neither is an authored tag. |
| `0ae3a351a6df0a2333cd` `chat-markdown-renderer/src/ChatMarkdown.tsx:26` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `0c0b136d1d6ed48be6a1` `ticket-runner/tests/supervise.test.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `0c59aa1a25534aa8f2ab` `identity/tests/test_nfr008_change_password_timing.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `0ca4ad8558cc3145f9a1` `auth-service/auth_service/services/auth_service.py:860` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `10a6794450ff505ad2f7` `auth-service/auth_service/services/auth_service.py:217` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `124caef55285e03054c2` `quire-rs/src/registry.rs:210` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `1aa084ab6f9321ec6c06` `auth-service/auth_service/api/auth.py:205` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `1e088ac91b4a34855f8d` `ix-agent-extensions/ix_agent_extensions/delegation/manager.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `20a1829d4912bcd870e9` `agent-duncan/tests/unit/test_executor.py:3610` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `21d5751e5ed8600a0e2c` `ts-auth-ui/src/LoginDialog.tsx:14` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `25506b29025dbbb7672d` `quire-wasm/tests/extract_validate.rs:85` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `2ed88afa8641aaf265c8` `filament-ui-shared/src/hooks/useArchetypeNav.ts:66` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `2faedc73f9b3f1cc303e` `typesetter/src/components/TypesetterEditor.tsx:144` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `30c210cd610e3c0ebb19` `quoin/corpus/verify.py:633` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `35ef60a490af7b5fcec7` `workflow-worker-pool/workflow_worker_pool/domain/nodes.py:307` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `394e4ed39758b2643189` `workflow-worker-pool/workflow_worker_pool/domain/nodes.py:307` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `3b1d2ba56c32e78054aa` `catalog-mcp-ui/src/index.tsx:12` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `3d298aed2b5fb27f3b38` `quire-rs-plain/src/loader/mod.rs:920` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `3e42473ef5dedbe35b1c` `filament-analysis-worker/filament_analysis_worker/domain/ingest.py:437` | `prose-citation` | The line is a wrapped parenthetical list continuation (TC-215/223/227)., not an authored tag. |
| `3e86b01e56243c7a6b29` `identity/identity/api/bootstrap.py:219` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `42f725be593ca7b78e8a` `ix-cli-core/src/config/service.ts:156` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `458152cbd25de597ae97` `permission-service/permission_service/services/policy_service.py:143` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `4a244b46e16c5460b2a4` `gateway-bff-contract/tests/test_device_models.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `4a9017f5f9bb7a6a9704` `py-permissions/py_permissions/evaluation/hierarchy.py:40` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `4d6ab7ca453f9ff73aeb` `ix-cli/packages/local/src/commands/list.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `4d91dcfe5809b43a710e` `k8s-orchestration/tests/integration/test_k8s_interaction.py:174` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `51d65b1095636008f947` `ix-agent-flow/tests/test_runtime.py:62` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `536746915a55d4239144` `table-renderer/src/types.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `5392e129185c2d564a39` `review-worker/review_worker/domain/validation_summary.py:213` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `557e3e3630aed679e4cc` `quire-cli/src/commands/coverage.rs:82` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `58eb173e6c86d75c7418` `ts-build-chain/src/utils/stable.ts:312` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `59c103e63524b83acdfd` `quire-rs/src/loader/mod.rs:647` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `5cb5b523af4f2ecb30a5` `agent-duncan/tests/integration/test_it_007_config.py:44` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `5d5a9c3c11792634ab0f` `filament-view-object/src/__tests__/client.test.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `60d19529fd4d238e7ec2` `ix-cli-core/src/secrets/service.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `61b0dff0a041d1cc73c5` `typesetter/src/components/CodeblockRegistry.tsx:27` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `639fe0ac2e31e8601be1` `user-admin-ui/tests/FR008.test.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `683bc7d691f098e8c2c7` `filament-view-review/src/components/ReviewActions.tsx:14` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `6f16b3137c2c92cf39ea` `workflow-worker-pool/workflow_worker_pool/domain/nodes.py:307` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `72343f7b8d4e5943e021` `user-admin-ui/src/ApprovalQueue.tsx:32` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `73152eb8faf94b8c57f1` `ix-agent-browser-control/ix_agent_browser_control/services/actions.py:130` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `7347cd15b791e1a18ed0` `filament-ide-rs/crates/filament-service/src/facade.rs:535` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `73b4bd39450c03eb22d0` `workspace-worker/workspace_worker/handler.py:32` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `7421b458e55e99f6d576` `quire-rs/src/loader/mod.rs:647` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `751f9e60f304668cc08b` `identity/alembic/versions/a1b2c3d4e5f6_phase0_auth_gates.py:29` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `78c1fbae05c16ae9f17e` `filament-view-ecosystem/src/components/EcosystemPane.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `79a755c8731b534d943a` `code-diff-editor/src/ViewModeSelector.stories.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `7cda78f8f8e32dac9c2a` `filament-editor-gateway/filament_editor_gateway/discovery.py:227` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `80a437867e0702f51ee9` `rjsf-ix-theme/src/templates/BaseInputTemplate.tsx:10` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `8252cb8e459f2b0a683f` `ts-auth-ui/src/ProtectedRoute.tsx:20` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `83a145f9718859904fb9` `catalog-service/catalog_service/services/repo_service.py:158` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `83ee08eda4b05a5e9f71` `ts-build-chain/src/commands/build-chain.ts:494` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `85692afc49c09b759c0d` `ix-agent-flow/tests/test_skill_emit.py:18` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `861d120de68fa66f7dcd` `filament-ui/src/sidebar/Sidebar.tsx:75` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `862d03be22f5fe176c43` `cloudmanager-local-sync/tests/test_master_runner.py:114` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `8b100908382980a7b98e` `filament-editor-integration/tests/test_imports.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `8d09388a34696782173d` `ecaz/src/tests/ec_distann_basic.rs:2599` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `8f501fad8004cdc1e5ea` `quire/src/react/SectionTable.tsx:24` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `91df5fcba3fbf2d10f9e` `quire-rs-plain/src/symbols/trace.rs:536` | `prose-citation` | The only occurrence is inside a Rustdoc code example describing list grammar, not a tag on legacy_ids. |
| `93f17fc00cfca6f2df68` `filament-view-ecosystem/src/components/LegendControls.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `943c78b3007db5d55384` `ts-auth-ui/src/DeviceApprovalRoute.tsx:86` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `94e68805cb15a774ae9f` `quire-rs-plain/src/corpus/resolve.rs:252` | `prose-citation` | Both stored occurrences are parenthetical citations in explanatory Rustdoc; neither is an authored tag. |
| `9811132da77dd8d28ae0` `quire-rs/src/traceability.rs:829` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `9ad6d40b711693c3a949` `pytest-results/pytest_results/grouping.py:31` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `9bef65e5e880d3e4e6d7` `quire-rs/src/registry.rs:210` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `9ece23053f962bfb5292` `quire/src/core/frontmatter.ts:62` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `a1b919381c656cd2f6a1` `quire-rs-plain/src/coverage.rs:604` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `a4e216e58af538028123` `quire-rs-plain/src/loader/mod.rs:1061` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `a88d733a939d884cf4f4` `ecaz/src/tests/ec_distann_physical_lifecycle.rs:3339` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `a9f373fa07bb5993ddef` `ticket-runner/tests/cli.test.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `abf70838ba7ca5809a97` `filament-editor-app/services/filament-collab/src/server.ts:41` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `b094bff7e65fb8f326b0` `secrets-injector-webhook/secrets_injector_webhook/parsing.py:13` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `b0f2410b14f68ec00ac3` `quire/src/react/QuireProvider.tsx:168` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `b38b82213f95de46f9e8` `typesetter/tests/MarkdownGrammar.test.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `b4e5bb044e6dd44e597d` `quire-rs/src/python/mod.rs:144` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `b4f98415c8504332ab04` `quire-rs-plain/src/loader/mod.rs:920` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `b9315725e297a0fe5613` `ix-agent-memory-service/ix_agent_memory_service/api/routes.py:33` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `ba5c310648c90b4db2c8` `quire/src/react/QuireProvider.tsx:59` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `bab19a854d88326d7748` `qa-corpus/bounds.py:950` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `bda1b017f6a915a89ab8` `mermaid-renderer/src/MermaidRenderer.tsx:577` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `be071c21a0aefdae858a` `secrets-ref/secrets_ref/models.py:51` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `c0fd8d5c2e5bcdd67cbd` `agent-duncan/tests/unit/test_rotation.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `c6520011c92b02135d55` `filament-editor-integration/filament_integration/fixtures/modules.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `cbb52aa285d104752650` `ix-cli/apps/ix/src/commands/local/up.ts:49` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `d12650748cf7bab87102` `filament-editor-gateway/filament_editor_gateway/discovery.py:227` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `d2d4c250fab99d73edc4` `auth-py/auth_py/agent_auth.py:24` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `d596327e880024b4eac3` `ecaz/src/am/ec_distann/dml.rs:269` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `d5f0fef838288b11aa50` `typesetter/src/hooks/useLineSelection.ts:44` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `da33755ea6831e85930b` `ix-cli/packages/local/src/rollout.ts:622` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `da84c461c7bc1a32c113` `workflow-execution/workflow_execution/executor/engine.py:520` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `defff72bb7c61bccc27b` `quire-rs/src/loader/mod.rs:647` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e0901138f9b27e35ad6f` `ix-agent-memory-service/ix_agent_memory_service/api/routes.py:52` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e124605eb49caf6c7a92` `review-worker/review_worker/domain/validation_summary.py:213` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e386c6324b13a69d7475` `filament-view-object/src/ObjectIndexPage.tsx:129` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e435e32e874f9a7d7b42` `auth-service/auth_service/services/auth_service.py:217` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e5085d6167b85a27e1d3` `mermaid-renderer/src/MermaidRenderer.stories.tsx:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e9561ea47c9c696a30a9` `platform-test-kit/tests/integration/test_k8s_engine.py:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `e9e2862c60578ec7eeba` `ecaz/src/am/ec_distann/routine.rs:336` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `eb14c667d104a4413609` `quire-rs-plain/src/extract/dsl.rs:148` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `ec5ca3e7c9b0bedd1efc` `cloudmanager-local-sync/tests/test_filament_core_client.py:286` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `f7fdd8c4df399391125b` `py-state-machine/py_state_machine/engine.py:74` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `f99542988fbc595dbadd` `ix-cli/packages/local/tests/cluster-status.test.ts:1` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `f9cab11b857ec7511eef` `filament-ide-rs/crates/filament-backend/src/backend.rs:820` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `fe4edd563a5004ddcf98` `scenario-runner/scenario_runner/services/executor.py:32` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `feb4de0b535c931905cd` `quire-rs/src/validate_document.rs:214` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
| `ff01053ee9144d8cb0dc` `ix-agent-flow/tests/test_decorators.py:45` | `authored-tag` | The stored occurrence uses the id as an explicit requirement or trace label on code or a non-binding test/container surface. |
