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

A replacement cannot be selected from API similarity alone. Duplicate keys,
merge keys, aliases, implicit scalars, timestamp interpretation, and non-string
mapping keys can change the `serde_json::Value` returned for the same document,
which would silently change behavior across languages and repositories.

## Decision drivers

1. Preserve the current parsed `serde_json::Value` and malformed/absent outcome
   for every existing frontmatter block.
2. Use a maintained crate with reviewable provenance and no active advisory
   against the selected version.
3. Preserve the crate's declared Rust 1.75 minimum supported version (MSRV).
4. Preserve the existing `serde_yaml::from_str`, `from_slice`, and `to_string`
   call surface so this change does not mix dependency migration with parser
   redesign.
5. Make the selected package and pin mechanically visible in the repository.

## Options considered

| Option | Maintenance and compatibility | Disposition |
| --- | --- | --- |
| Hold `serde_yaml = 0.9.34` | Exact current behavior, but the upstream is archived and explicitly deprecated. A hold would contradict NFR-009's inactive-upstream direction and would need an explicit expiry. | Rejected. |
| `serde_yml` | A historical near-drop-in fork. Versions through 0.0.12 have the unpatched `RUSTSEC-2025-0068` unsoundness advisory and the project was archived. | Rejected. |
| `serde_yaml_ng = 0.10.0` | Drop-in fork, MIT, Rust 1.64 MSRV, and compatible in principle. It is maintained by an individual project rather than the YAML organization. | Viable fallback, not selected. |
| `serde_norway = 0.9.42` | Drop-in fork, MIT/Apache-2.0, Rust 1.71.1 MSRV. | Viable fallback, not selected. |
| `yaml_serde = 0.10.2` | The active fork under the YAML Organization; MIT/Apache-2.0; Rust 1.64 MSRV; it retains the original API and `unsafe-libyaml 0.2.11` backend. | Selected. |
| Current `yaml_serde` (0.10.3 or newer) | Actively maintained; 0.10.7 uses `libyaml-rs`. Releases from 0.10.3 require Rust 1.82. | Rejected for this change because raising quire-rs's MSRV is out of scope. |
| A pure-Rust parser or custom adapter | Could remove the inherited backend, but is not a drop-in Serde surface and widens the semantic and implementation change. | Deferred to a separate ADR if the backend itself must change. |

Primary maintenance sources:

- [`serde_yaml` 0.9.34 archive notice](https://github.com/dtolnay/serde-yaml/releases/tag/0.9.34)
- [`yaml_serde` repository and migration guidance](https://github.com/yaml/yaml-serde)
- [`serde_yml` advisory RUSTSEC-2025-0068](https://rustsec.org/advisories/RUSTSEC-2025-0068.html)

## Differential decision evidence

On 2026-09-07, a read-only spike parsed complete frontmatter blocks with both
`serde_yaml = 0.9.34` and `yaml_serde = 0.10.2` into `serde_json::Value` and
compared success/failure and the complete value. The inputs were the
`quire-rs` specification at `8b8020e`, its checked-out controlled corpus, and
the current TypeScript `quire` repository, excluding `.git`, `.worktrees`,
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

This spike supports the proposal; it is not implementation evidence. The
committed migration must repeat the differential at its reviewed revision and
record the result on the delivery pull request.

## Decision

After this ADR and the corresponding NFR-009 amendment are accepted, replace
the package while retaining the existing Rust import name:

```toml
serde_yaml = { package = "yaml_serde", version = "=0.10.2" }
```

The exact pin is deliberate. It is the newest `yaml_serde` release compatible
with Rust 1.75, satisfies NFR-009's load-bearing dependency policy, and prevents
a resolver running on a newer compiler from silently selecting a release that
raises the crate's MSRV.

Implementation must also strengthen `scripts/audits/check_dep_pins.sh` so it
verifies the selected package and exact version, rejects the deprecated
`serde_yaml` package, and fails if the YAML dependency returns to a caret or
wildcard range. `Cargo.toml` must link this ADR beside the dependency.

No frontmatter behavior change is accepted as part of the migration. Any
corpus difference, TypeScript/Python parity failure, typed manifest/clause/DSL
failure, Rust 1.75 build failure, license denial, or dependency advisory blocks
the change and reopens this decision.

## Required implementation evidence

Before the dependency change may leave draft:

1. Repeat the two-engine differential over the same corpus classes at the
   implementation revision and report input revisions, counts, and every
   difference; the accepted difference count is zero.
2. Run the existing Rust/TypeScript/Python frontmatter parity suite unchanged.
3. Run typed module-manifest, clause-set, and extraction-DSL tests unchanged.
4. Compile the default crate with Rust 1.75 and run the repository's full CI,
   license, advisory, unsafe-surface, and static-audit gates.
5. Prove from `Cargo.lock`/`cargo tree` that `serde_yaml 0.9.34+deprecated` is no
   longer present and `yaml_serde 0.10.2` is the selected package.

## Consequences

- The maintained wrapper has organizational ownership and an active release
  line while the observable YAML behavior and call surface stay fixed.
- The selected version still uses `unsafe-libyaml 0.2.11`, the same backend as
  the current crate. This ADR removes the deprecated wrapper; it does not claim
  to eliminate the backend or its unsafe surface.
- Newer `yaml_serde` fixes cannot be absorbed until quire-rs raises its MSRV or
  the upstream publishes a compatible maintenance release. A security fix that
  is unavailable on the selected line reopens this ADR immediately.
- NFR-009 and its audit must be amended before implementation because their
  current crate names and executable policy do not describe this decision.

## Revisit triggers

Reopen this decision when any of the following occurs:

- an advisory affects `yaml_serde 0.10.2` or `unsafe-libyaml 0.2.11`;
- the selected release line is archived or has no viable security-fix path;
- quire-rs raises its MSRV to at least Rust 1.82;
- a maintained pure-Rust Serde engine demonstrates zero differential over the
  same corpus and typed inputs;
- TypeScript or Python reference semantics intentionally change.
