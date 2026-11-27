# Baseline artifact corpus provenance

This fixture is a data-only snapshot of the artifact schemas and manifests used by the Quoin installation on 2026-11-27. It intentionally excludes Python modules, skeletons, examples, and other executable or authoring content.

| Module | Quoin source tag | Resolved source commit | Artifact types |
|---|---|---|---:|
| spec-artifacts-iso | v0.28.0 | 88b2a8e041db3b889bd6ecf7170d8447a0b83a68 | 9 |
| spec-artifacts-app | v0.10.1 | cc6be452523f141d4c2539a0502f21126260224a | 1 |
| spec-artifacts-process | v0.38.0 | caa02e57eefd3b8e86de77ca4ff35962a4c1f3c2 | 18 |

The files under each module directory are the exact `manifest.yaml` and `schemas/` files from the pinned source subdirectory. The snapshot contains 28 artifact types (9 + 1 + 18). This replaces the historical 17-artifact count: the full current installed set has grown since that baseline was recorded.
