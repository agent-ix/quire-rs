# Static Audit Scripts

Collected static checks referenced by FR/NFR ACs. Each is a standalone bash script run by `make audit-static` (which globs `scripts/audits/*.sh`) and so by `make ci`.

| Script | Audits | Acceptance Criteria |
|---|---|---|
| `check_no_shared_mutable.sh` | Parallel-walk fan-out in `src/corpus` and `src/python` is lock-free; interior mutability only via named exemptions. Kept as a script because clippy `disallowed-types` has one shared list, so banning Mutex/OnceLock in `src/corpus` would also ban HashMap there, and `thread_local!`, `static mut` and `unsafe impl Sync` are not types. | FR-024-AC-9, TC-502 |
| `check_semantic_boundary.sh` | Semantic-extraction boundary | NFR-021-AC-1/AC-2, FR-070-CON-1/CON-2, FR-071-CON-1, FR-072-CON-2 |
| `check_tree_sitter_boundary.sh` | No `Cargo.toml` other than `crates/quire-code-extraction/Cargo.toml` names `quire-code-parse` or `tree-sitter`/`tree-sitter-<lang>` as a dependency | FR-051-CON-1, PLAT-843, PLAT-851, PLAT-911 |

Enforced by clippy (`make lint`) via `clippy.toml` `disallowed-types` / `disallowed-methods`:

- No `std::collections::HashMap` in `src/parser`, `src/extract`, `src/merge.rs`, `src/validate_document.rs` (NFR-006-AC-3, TC-058).
- No `std::process::Command::new` in the library crate (StR-001-AC-2, TC-201).

All scripts SHALL exit 0 on success and non-zero on violation, with descriptive stderr output.

Unsafe blocks are checked by the workspace Clippy `undocumented_unsafe_blocks` lint.
HTTP/RPC client crates and `schemars` are denied by `deny.toml` through
`make deny` and `make deny-grammars`. The semantic schema's generator markers
are checked by `tests/it/semantic_surface.rs`.
