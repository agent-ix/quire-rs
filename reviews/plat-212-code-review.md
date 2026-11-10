---
id: SR-20267
title: "code-review of PLAT-212 import_types and import diagnostics (quire-rs PR 638)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@7f2eaf60; Cargo.lock; spec/tests.md; src/filament.rs; src/loader/mod.rs; src/semantic/contract.rs; src/semantic/python_entry.rs; src/semantic/resolver.rs; src/semantic/units.rs; tests/it/semantic_contract.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-069
    type: references
---

# Code review of PLAT-212 import_types and import diagnostics (quire-rs PR 638)

## Summary

Ticket: PLAT-212. One code-review pass, with the rust-review lane folded in,
over `git diff origin/main...7f2eaf60` (9 files, +890/-100). Gates were not
rerun; the coder reports `make ci` green on this head.

Examined, clean:

- `check_import_types` refuses each malformed shape (non-object, key absent
  from `imports`, non-array, empty list, non-string, empty name, duplicate)
  with `semantic.invalid-value` at the unescaped
  `semantic.import_types.<org>/<repo>` locus, and the schema-error mapping
  skips `/import_types` paths so no second pointer-form error appears.
- An undeclared package's `$ref` still reaches the `else` branch of `walk` and
  fails `semantic.schema-ref-unshipped`; `without_import_refs` only strips a
  `$ref` whose value `import_package` matches, and `import_package` requires
  `<base><package>/`, so `agent-ix/prov` does not admit `agent-ix/prov-x`.
- Opaque refs are recorded, not followed or fetched; the returned `schema` and
  `documents` stay unstripped, and downstream `RecordSchema` treats an
  unresolvable `$ref` as unreadable rather than panicking.
- `semantic.unknown-import-type` from `import_types` and from `$ref` type
  names fires only against a provider found unrefused at the declared version,
  at the specified loci, deduplicated per object type and name.
- `semantic.unused-import` warns at `semantic.imports.<package>` when no
  recorded import ref names the package.
- Every package cycle is still refused; the type graph only renames a cycle
  when its own edges cycle and the package is on a package cycle. `find_cycles`
  indexes are total (every type edge target is a node; every provider index is
  in `order`).
- Cargo.lock bump of `agent-ix-semantic-schema` to ce72bc51 is required: the
  old revision's `module-semantic-block.schema.json` has no `import_types`, so
  block validation would refuse the key as an unknown root key (not skipped by
  the `/import_types` filter). The upstream range 7a416a9..ce72bc51 is the
  PLAT-212 schema change plus semantic-core/schema release and docs commits; a
  normal `cargo update -p` on a `branch = "main"` git dependency, not a pin.
- Public API: `resolve_reference` gains `imports: &[String]`; `SemanticModule`
  gains `import_types` (`serde(default, skip_serializing_if = "is_empty")`, so
  existing JSON output is unchanged). No out-of-repo caller of
  `resolve_reference` or literal constructor of `quire_rs` `SemanticModule`
  was found under `~/dev` (quoin's `SemanticModule` is its own type).
- Rust lane: the `Walk` context struct removes the
  `too_many_arguments` allow; no new `unwrap` on fallible input; `find_cycles`
  replaces the nested `visit` with an enum mark instead of `u8` colours.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | [code-bug] `walk` tests `import_package(url, imports)` before `url.strip_prefix(module_base)`, so a module whose `semantic.imports` names its own package treats every sibling `$ref` of its own schema as an opaque declared-import ref: the sibling file is never read and the `$ref` is stripped to accept-any. At a matching version the self-edge is refused as `semantic.import-cycle`, but at any other version the provider is not found, only `semantic.import-unresolved` warns, the module loads, and its own type schemas silently stop validating the referenced subschemas. Nothing refuses a self-import. Fix: check the module's own base first (or exclude the module's own package from `imports` passed to `resolve_reference`) and add a test that a self-import at another version still validates a sibling `$ref`. | src/semantic/resolver.rs:411; src/semantic/resolver.rs:442; src/semantic/resolver.rs:70-76; src/loader/mod.rs:1048-1056 | implementation-bug-despite-evidence |

## Verdict

Not mergeable until FND-001 is fixed: it is the one path where opaque
accept-any stripping weakens validation of the module's own schemas. The
rest of the change matches FR-069 AC-16..20 and the settled design.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 42e944bf: `resolve_reference` drops the module's own package (taken from the root `$id` base) from `imports` before walk, compile and strip; `self_import_does_not_opaque_sibling_ref` asserts a self-import keeps a sibling `$ref` followed and validating (fails on 7f2eaf60). |
