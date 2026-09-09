---
id: ADR-0012
title: "YAML engine maintenance and parity"
type: ADR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-009"
    type: "relates_to"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-006"
    type: "relates_to"
---

# ADR 0012: YAML engine maintenance and parity

**Status**: Proposed
**Date**: 2026-09-07
**Decision authority**: kreneskyp (acceptance pending)

## Context

`quire-rs` resolves `serde_yaml` to `0.9.34+deprecated`. Its upstream repository
was archived on 2024-03-25 and states that no further releases are planned. The
crate is used in production paths for document frontmatter, module manifests,
clause sets, and extraction DSL data. Of those, frontmatter is a compatibility
surface: [FR-006](../../functional/FR-006-frontmatter-with-fallback.md) ports the
TypeScript and Python parser behavior, and its parsed map carries document
identity and relationships.

[NFR-009](../../non-functional/NFR-009-dependency-pinning.md) already treats the
YAML engine as load-bearing, requires a tilde or exact pin, and says to replace
`serde_yaml` if its upstream becomes inactive. The manifest currently declares
`serde_yaml = "^0.9"`, while `scripts/audits/check_dep_pins.sh` rejects only
wildcards. The written requirement, manifest, and executable gate therefore do
not agree.

The production call surface is wider than frontmatter and module discovery.
Clause sets, extraction DSL data, traceability models, and lint rules also pass
through the same dependency, including serialization in the traceability path.
A repository-wide import alias changes all of them at once, so a frontmatter-only
differential cannot establish migration parity.

A replacement cannot be selected from API similarity alone. Duplicate keys,
merge keys, aliases, implicit scalars, timestamp interpretation, and non-string
mapping keys can change the `serde_json::Value` returned for the same document,
which would silently change behavior across languages and repositories.

## Decision drivers

1. Preserve the current parsed `serde_json::Value` and malformed/absent outcome
   for every existing frontmatter block.
2. Use a maintained crate with reviewable provenance and no active advisory
   against the selected version.
3. Preserve the existing `serde_yaml::from_str`, `from_slice`, and `to_string`
   call surface so this change does not mix dependency migration with parser
   redesign.
4. Make the selected package and pin mechanically visible in the repository.

## Options considered

| Option | Maintenance and compatibility | Disposition |
| --- | --- | --- |
| Hold `serde_yaml = 0.9.34` | Exact current behavior, but the upstream is archived and explicitly deprecated. A hold would contradict NFR-009's inactive-upstream direction and would need an explicit expiry. | Rejected. |
| `serde_yml` | A historical near-drop-in fork. Versions through 0.0.12 have the unpatched `RUSTSEC-2025-0068` unsoundness advisory and the project was archived. | Rejected. |
| `serde_yaml_ng = 0.10.0` | Drop-in fork, MIT, Rust 1.64 MSRV, and compatible in principle. It is maintained by an individual project rather than the YAML organization. | Viable fallback, not selected. |
| `serde_norway = 0.9.42` | Drop-in fork, MIT/Apache-2.0, Rust 1.71.1 MSRV. | Viable fallback, not selected. |
| `yaml_serde = 0.10.2` | The active fork under the YAML Organization and API-compatible, but selected only because the inherited Rust 1.75 declaration prevented Cargo from choosing the current line. It retains the deprecated implementation's `unsafe-libyaml` backend. | Rejected: an old repository declaration is not an independent reason to freeze new work. |
| `yaml_serde = 0.10.7` | Current crates.io release as observed 2026-09-08; maintained by the YAML Organization, MIT/Apache-2.0, Rust 1.82 minimum, and backed by `libyaml-rs 0.3.0`. The backend shares the former backend's C2Rust-transpiled libyaml lineage, so this is a maintenance choice rather than a comparative-safety claim. Rust 1.98.1 satisfies its tool requirement. | Selected, subject to the zero-difference and toolchain gates below. |
| A pure-Rust parser or custom adapter | Could remove the inherited backend, but is not a drop-in Serde surface and widens the semantic and implementation change. | Deferred to a separate ADR if the backend itself must change. |

Primary maintenance sources:

- [`serde_yaml` 0.9.34 archive notice](https://github.com/dtolnay/serde-yaml/releases/tag/0.9.34)
- [`yaml_serde` repository and migration guidance](https://github.com/yaml/yaml-serde)
- [`serde_yml` advisory RUSTSEC-2025-0068](https://rustsec.org/advisories/RUSTSEC-2025-0068.html)

## Differential decision evidence

On 2026-09-07, an exploratory read-only spike parsed complete frontmatter blocks with both
`serde_yaml = 0.9.34` and `yaml_serde = 0.10.2` into `serde_json::Value` and
compared success/failure and the complete value. The inputs were the
`quire-rs` specification at `8b8020e`, its checked-out controlled corpus, and
the then-current TypeScript `quire` worktree, excluding `.git`, `.worktrees`,
`node_modules`, `target`, and `dist`.

| Measure | Result |
| --- | ---: |
| Markdown documents enumerated | 589 |
| Complete frontmatter blocks compared | 575 |
| Corpus outcome or value differences | 0 |
| Focused semantic-case differences | 0 of 5 |

The focused cases pin the decision-sensitive behavior observed in the spike:

- duplicate keys: both engines retain the final value;
- merge key: both preserve `<<` as a mapping key rather than applying a merge;
- `y`, `no`, `on`, and `2026-09-07`: both return strings;
- aliases: both materialize the same value;
- a sequence used as a mapping key: both fail conversion to
  `serde_json::Value`.

This spike demonstrates that the comparison method can expose the required
semantic surface; it does not establish the selected 0.10.7 engine and is not
implementation evidence. Its exact
TypeScript revision, complete input manifest, comparator, module identities,
and raw payload were not retained, so the zero is not independently
reproducible. The committed migration must repeat the differential at its
reviewed revision and retain the complete AP-201 provenance record rather than
copy this exploratory count into the delivery pull request.

## Decision

After this ADR and the corresponding NFR-009 amendment are accepted, replace
the package while retaining the existing Rust import name:

```toml
serde_yaml = { package = "yaml_serde", version = "=0.10.7" }
```

The exact pin is deliberate because YAML behavior is identity-bearing. It is
the current `yaml_serde` release observed on 2026-09-08 and uses the maintained
`libyaml-rs 0.3.0` backend rather than the former `unsafe-libyaml` package.
Both packages descend from a C2Rust translation of libyaml; replacing the
package identity establishes active maintenance ownership, not a reduction in
transitive unsafe code. Issue #417 independently governs the repository's
exact Rust 1.98.1 support and qualification policy. This ADR consumes 1.98.1 as
the implementation-run compiler and does not own the compiler lifecycle. The
implementation order is nevertheless deliberate: `yaml_serde 0.10.7` itself
requires only Rust 1.82, but this migration waits for #417 to land the separately
accepted 1.98.1 repository baseline rather than introducing and qualifying an
interim 1.82 baseline that the ecosystem would immediately retire. NFR-022 owns
the choice and compatibility consequences of 1.98.1; this ADR owns only the
decision to begin the YAML implementation after that baseline is available.

The import alias is repository-wide. Production compatibility evidence covers
frontmatter, module manifests, clause sets, extraction DSL data, traceability
models, and lint rules. Test and corpus loaders may be adapted to the aliased
package, but their successful compilation or parsing is not a substitute for
those production-path assertions and must not narrow the governed corpus.

Implementation must also strengthen `scripts/audits/check_dep_pins.sh` so it
verifies the selected package and exact version, rejects the deprecated
`serde_yaml` package, and fails if the YAML dependency returns to a caret or
wildcard range. `Cargo.toml` must link this ADR beside the dependency.

No production YAML behavior change is accepted as part of the migration. Any
corpus difference, TypeScript/Python parity failure, typed manifest, clause,
DSL, traceability, or lint failure, Rust 1.98.1 build failure, license denial, or
dependency advisory blocks the change and reopens this decision.

## Required implementation evidence

Before the dependency change may leave draft:

1. Repeat the two-engine differential over the same corpus classes at the
   implementation revision and retain exact source/corpus/module revisions,
   input paths and digests, comparator and producer versions, raw result,
   population counts, exclusions, and every difference; the accepted difference
   count is zero.
2. Run the existing Rust/TypeScript/Python frontmatter parity suite unchanged.
3. Run typed module-manifest, clause-set, extraction-DSL, traceability-model,
   and lint-rule tests unchanged.
4. After #417 lands the independently governed compiler baseline, compile the
   default crate and all affected targets with Rust 1.98.1 and run the
   repository's full CI, license, advisory, first-party unsafe-surface, and
   static-audit gates. This deliberate sequencing is not a claim that the YAML
   engine requires more than Rust 1.82. Refresh the
   advisory database at the implementation revision and retain its observed
   index revision or timestamp with the result.
5. Prove from `Cargo.lock`/`cargo tree` that `serde_yaml 0.9.34+deprecated` and
   its former `unsafe-libyaml` backend are no longer present, while
   `yaml_serde 0.10.7` and `libyaml-rs 0.3.0` are selected. This is package
   resolution and maintenance-line evidence, not comparative unsafe evidence.
6. Run NFR-002's existing 5 MB parse, typical-artifact validation, and
   same-runner regression gates without changing their baselines or thresholds.
7. Retain a repository-wide import/call-site census that classifies every YAML
   use as a production consumer or test/corpus loader and proves every production
   class is represented in the differential or typed-consumer suites.
8. Bind every new Rust test to its TC and NFR-009 criterion with
   `use ix_trace_rs::trace;` and a bare `#[trace("TC-...", "NFR-009-AC-...")]`
   attribute, then run Quire reconciliation and the static macro-form audit.

## Dependency boundary

- Upstream maintenance and future security-fix availability are assumptions
  monitored by the revisit triggers.
- The selected package bytes/version, resolved backend, license/advisory
  posture, Rust 1.98.1 compatibility, and observed parser equivalence are
  guarantees owned and verified by `quire-rs`.
- TypeScript and Python are reference implementations only for the governed
  frontmatter compatibility surface; Rust-only typed YAML consumers are
  verified by their own suites.
- The governed corpora are versioned test inputs. Population completeness and
  exclusions are part of the retained result, not ambient workspace state.

## Consequences

- The maintained wrapper has organizational ownership and an active release
  line while the observable YAML behavior and call surface stay fixed.
- The selected version replaces the inherited `unsafe-libyaml` package with
  `libyaml-rs` while retaining the same C2Rust-transpiled libyaml lineage. The
  package change improves maintenance ownership, not demonstrated transitive
  safety. It expands the differential obligation; API compatibility alone is
  not evidence of semantic compatibility.
- A newer `yaml_serde` fix is not absorbed implicitly: the exact pin moves only
  after the same differential, toolchain, advisory, and performance gates pass.
  A security fix unavailable on the selected line reopens this ADR immediately.
- While this ADR is reopened by an advisory against the selected package or
  backend, releases containing the affected parser remain blocked. Recovery
  requires a newly reviewed zero-difference selection: move to the fixed
  upstream line, select another maintained engine, or remove the affected
  release surface. A time-bounded security waiver
  is a separate owner decision with an expiry; reopening alone is not a waiver.
- NFR-009 and its audit must be amended before implementation because their
  current crate names and executable policy do not describe this decision.

## Revisit triggers

Reopen this decision when any of the following occurs:

- an advisory affects `yaml_serde 0.10.7` or `libyaml-rs 0.3.0`;
- the selected release line is archived or has no viable security-fix path;
- a maintained pure-Rust Serde engine demonstrates zero differential over the
  same corpus and typed inputs;
- TypeScript or Python reference semantics intentionally change.
