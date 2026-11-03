# Static Audit Scripts

Collected static checks referenced by FR/NFR ACs. Each is a standalone bash script invoked from CI and locally via `make ci`.

| Script | Audits | Acceptance Criteria |
|---|---|---|
| `check_no_shellout.sh` | No `std::process::Command` invocations targeting python/node/npm/pip in `src/` | StR-001-AC-2, TC-201 |
| `check_hashmap_audit.sh` | No `std::collections::HashMap` in render/parse code paths | NFR-006-AC-3, TC-058 |
| `check_tree_sitter_boundary.sh` | No `Cargo.toml` other than `crates/quire-rust-extraction/Cargo.toml` names `quire-code-parse` or `tree-sitter`/`tree-sitter-<lang>` as a dependency | FR-051-CON-1, PLAT-843, PLAT-851, PLAT-911 |

All scripts SHALL exit 0 on success and non-zero on violation, with descriptive stderr output.

Unsafe blocks are checked by the workspace Clippy `undocumented_unsafe_blocks` lint.
HTTP/RPC client crates and `schemars` are denied by `deny.toml` through
`make deny` and `make deny-grammars`. The semantic schema's generator markers
are checked by `tests/semantic_surface.rs`.

## Implementation note

Each script is created lazily as the corresponding code path lands.
