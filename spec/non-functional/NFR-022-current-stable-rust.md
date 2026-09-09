---
id: NFR-022
title: "Current stable Rust qualification"
type: NFR
quality_attribute: maintainability
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-004"
    type: "constrains"
---
# NFR-022: Current stable Rust qualification

## Statement

`quire-rs` SHALL set its minimum and qualification compiler to exact Rust
1.98.1 and advance through the governed transition below when a newer stable
release becomes available.

## Scope

The three Rust configuration fields have distinct meanings and deliberately
carry the same accepted version:

- `Cargo.toml` `rust-version` is the minimum compiler promised to downstream
  consumers. Setting it to 1.98.1 intentionally ends support for compilers
  below 1.98.1; backward compatibility with the inherited 1.75 floor is not a
  goal.
- `rust-toolchain.toml` selects the exact stable compiler used to build and
  qualify this repository.
- `clippy.toml` `msrv` bounds the language/library assumptions Clippy may use
  in suggestions; aligning it to 1.98.1 makes the lint policy match the
  supported compiler floor.

The requirement also applies to every stable-channel CI and build-script
selection, the default crate, all targets/features, Python bindings and
packaging, the WASM target, tests, documentation, audit tools, benchmarks, and
release builds. Separately pinned nightly fuzz and sanitizer lanes are outside
the stable-version declaration and advance only through their own tool
qualification.

Downstream repositories and packages—including `quire-cli`, the Python wheel
lane, the WASM consumer, and any external Rust consumer—must use Rust 1.98.1 or
newer before adopting a `quire-rs` revision governed by this requirement. No
compatibility shim or parallel old-Rust implementation is required.

Existing repository metadata, formatting changes, and repairable Clippy
findings are never evidence of tool incompatibility.

## Rationale

The prior 1.75 declaration propagated from scaffold history without a current
compatibility decision, while the selected toolchain independently moved to
1.94.1. Neither number is justified merely because it exists. Current stable
is the default for this young ecosystem; an older hold is an exceptional,
time-bounded response to a reproduced failure in a required tool.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-022-AC-1 | `Cargo.toml` advertises minimum supported Rust 1.98.1, `rust-toolchain.toml` selects exact 1.98.1, `clippy.toml` permits 1.98.1 language/library assumptions, and every stable CI/build declaration selects exact 1.98.1; no stable 1.94.1 or floating `stable` selection remains. | static-quality |
| NFR-022-AC-2 | The locked default, all-target/all-feature, Python, WASM, test, documentation, advisory, license, unsafe-audit, benchmark-build, and release gates applicable to stable Rust pass on 1.98.1 without weakened flags, exclusions, or thresholds. | integration-testing |
| NFR-022-AC-3 | Within seven calendar days after a newer stable Rust release, the owner records the same compatibility matrix and either advances all stable declarations together or records an AC-4 hold. | inspection |
| NFR-022-AC-4 | A compiler hold names the required failing tool and exact command, retains the reproduced newer-version failure and successful older-version control, links an upstream issue, names an owner and rerun trigger, and expires no later than 30 days after recording. | inspection |
| NFR-022-AC-5 | Review rejects a proposed hold based only on formatting drift, a repairable lint finding, existing repository metadata, an untested concern, or downstream preference for an unsupported older compiler. | inspection |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Stable Rust declarations with the required value and documented role | all | all equal 1.98.1 | static-quality |
| Required stable Rust gate classes passing on 1.98.1 | all | all | integration-testing |
| Days from a newer stable release to a recorded compatibility result | at most 7 | 7 | inspection |
| Compiler hold attribution, reproduction, control, and removal fields present | all | all | inspection |
| Compiler hold lifetime | at most 30 days | 30 days | inspection |
| Formatting-, lint-, inherited-pin-, speculation-, or old-consumer-only holds accepted | 0 | 0 | inspection |

## Verification

- A static conformance test parses the manifest, toolchain, Clippy, workflow,
  and stable build-script declarations, asserts each field's role and exact
  accepted value, and rejects old or floating stable selections.
- The repository's real default, all-target/all-feature, Python, WASM, test,
  documentation, advisory, license, unsafe-audit, benchmark-build, and release
  commands run without weakening flags or thresholds.
- Release-date and hold records are reviewed as governed evidence; an
  executable test is not substituted for calendar timeliness or the judgement
  that a required tool is genuinely incompatible.
- Every new Rust requirement test imports `ix_trace_rs::trace` and uses bare
  `#[trace("TC-...", "NFR-022-AC-...")]` markers.

## Dependencies

- **Upstream**: Rust 1.98.1, released 2026-09-01, is the accepted current stable
  observation for this decision.
- **Downstream**: `agent-ix/quire-cli`, the in-repository Python and WASM lanes,
  `agent-ix/quire-wasm`, and external Rust consumers must adopt the new minimum
  before taking a governed `quire-rs` revision.
- **Independent consumers**: dependency decisions such as ADR-0012 may cite
  this compiler baseline but do not own or gate it.
