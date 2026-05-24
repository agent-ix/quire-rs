# Static Audit Scripts

Collected static checks referenced by FR/NFR ACs. Each is a standalone bash script invoked from CI and locally via `make ci`.

| Script | Audits | Acceptance Criteria |
|---|---|---|
| `check_no_net_deps.sh` | No HTTP/RPC client crates in `Cargo.lock` | FR-013-AC-6, TC-085 |
| `check_no_schemars.sh` | `schemars` is not a dep | FR-003-AC-4, TC-062 |
| `check_no_shellout.sh` | No `std::process::Command` invocations targeting python/node/npm/pip in `src/` | StR-001-AC-2, TC-201 |
| `check_dep_pins.sh` | Load-bearing deps use tilde or equals pins per NFR-009 | NFR-009-AC-1, AC-3 |
| `check_hashmap_audit.sh` | No `std::collections::HashMap` in render/parse code paths | NFR-006-AC-3, TC-058 |
| `verify_cookiecutter_inheritance.sh` | Safety scaffolding files match `rust-lib-cookiecutter` baseline | StR-004-AC-1, TC-203 |

All scripts SHALL exit 0 on success and non-zero on violation, with descriptive stderr output.

The unsafe-comment check (`../check_unsafe_comments.sh`) lives one level up, inherited from the cookiecutter.

## Implementation note

Each script is created lazily as the corresponding code path lands. Initial versions can be no-op stubs (exit 0) when the inputs they audit don't exist yet (e.g. `check_no_schemars.sh` is trivially passing today because `Cargo.toml` has no deps).
