---
id: SR-045
title: "code-review of ADR-0011 Phase 2 Wave A (CR-067..CR-071)"
type: SpecReview
analysis: code-review
scope: "src/corpus/resolve.rs, src/corpus/validate.rs, src/corpus/trace_refs.rs, src/corpus/declared_tables.rs, src/grammar/ears.rs, src/writeback.rs, examples/, spec/"
review_set: subset
---

## Summary

Reviewed the five Wave A changes as one integrated diff against `main` — CR-067 (`ix://` URI
grammar), CR-068 (per-check corpus severity), CR-069 (metamorphic properties + two writeback
fixes), CR-070 (ADR-0010 decisions, spec only) and CR-071 (mutation-scope tooling) — using the
`rust-review` lane. Five findings, one of them high; four are fixed in this branch as **CR-072**,
one is recorded and deliberately not fixed.

## Verdict

**FAIL** — one `high` finding (FND-001). It is **fixed in this branch**; the verdict records the
severity at the time of review, not the state after remediation. Re-review after CR-072 is
**PASS**.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Both link regexes recompiled on every call; `harvest_edges` is the per-document binding surface, so N documents cost N compiles — 148µs compile against 4.8µs of scanning | src/corpus/resolve.rs:308, src/corpus/resolve.rs:269 |
| FND-002 | medium | `make mutants-fr` resolved its traceability module from a hardcoded absolute path, so the new target only ran on one machine | examples/mutants_scope.rs:29 |
| FND-003 | low | The `MAX_PASSES` fixpoint ceilings degrade silently: reaching one returns a non-fixpoint with no signal | src/grammar/ears.rs:352, src/corpus/declared_tables.rs:265 |
| FND-004 | low | `BundleReport::route` panics via `expect` on an invariant no caller can currently violate | src/corpus/validate.rs:143 |
| FND-005 | low | `issue89_edge_sweep` hardcoded the workspace root, so the `[RAN]` harness was not reproducible elsewhere | examples/issue89_edge_sweep.rs:126 |

## Detail

### FND-001 — per-call regex compilation (high, fixed)

`ix_link_regex()` and `md_link_regex()` built a fresh `Regex` on every call.
`resolve()` calls each once per corpus load, which is harmless — but `harvest_edges()` calls
`ix_link_regex()` **once per document**, and it is the public single-document surface the Python
binding exposes as `quire.harvest_edges`. A consumer looping over a corpus paid one full regex
compilation per document.

**Measured**, rather than asserted: **147.8µs to compile against 4.8µs to scan a 20-line
document — 31× the work it enables.** Over the 2,152 FR documents in the `~/dev` corpus that is
roughly 300ms of pure compilation for ~10ms of scanning.

**Pre-existing, not a Wave A regression.** The blacklist CR-067 replaced compiled in 161.3µs,
marginally *slower* than the grammar that replaced it. The review surfaced it because Wave A
touched the function, not because Wave A caused it.

It also contradicts the repo's own idiom: `declared_tables.rs:329` already compiles its regexes
once behind `OnceLock`, and `scripts/audits/check_no_shared_mutable.sh` sanctions that exact
pattern by name. Fixed the same way, with the new site added to the audit's exemption list so the
justification is recorded where the gate reads it.

### FND-002 — machine-specific module path (medium, fixed)

`mutants_scope` read the traceability model from a hardcoded
`/home/peter/dev/spec-artifacts-process/...`. `make mutants-fr` is a documented target, so this
made a shipped entry point unusable off one machine, and it would have failed with a path nobody
else has rather than with an actionable message. Now derived from the sibling checkout, overridable
via `QUIRE_PROCESS_MODULE`, and it names the variable when the module is absent.

This is the shape that has bitten this ecosystem before — a release failed on a test reading a
hardcoded `~/dev` path (quoin v0.15.0).

### FND-003 — silent ceiling on the fixpoint loops (low, fixed)

CR-069 replaced two single-pass normalizers with bounded fixpoint loops. Both stop at
`MAX_PASSES = 16` and return whatever the last pass produced. Termination is *proven* — each pass
strictly shortens the string, or consumes a `..` an expansion cannot re-create — so the ceiling is
unreachable for real input and exists only as a fuzz-surface guard. But if the termination argument
were ever wrong, the functions would silently return a non-fixpoint, which for
`normalize_reference_cell` means a cell the pattern then rejects as a dangling reference.

Fixed with a `debug_assert!` at the ceiling: a broken termination argument now fails in tests
instead of degrading in production, without adding a release-path branch or changing the return
type from `String` to `Result`.

### FND-004 — panic on an internal invariant (low, **not** fixed)

`BundleReport::route` calls `.expect("route() is for pack findings; bridged ones use bridged()")`
on the finding's severity key. `route` is `pub(crate)`, every one of its eight callers passes a
`BundleFinding::in_pack(...)` which always sets the pack, and the bridged path has its own method —
so the panic is unreachable today.

Recorded rather than removed. The alternatives cost more than they buy: routing a pack-less finding
at its default tier would silently accept a programming error, and threading the pack through as a
separate parameter to eight call sites trades a named invariant for six extra arguments. The
`expect` message names the invariant and the violation is a compile-time-adjacent authoring
mistake, not untrusted input — which is the case `rust-review` §6 explicitly allows.

**Revisit if `route` ever becomes `pub`.** At that point the invariant stops being enforceable by
review and the type should carry it.

### FND-005 — hardcoded workspace root in the sweep harness (low, fixed)

`issue89_edge_sweep` is the `[RAN]` harness whose numbers CR-067 and CR-071 cite. Hardcoding
`/home/peter/dev` meant nobody could reproduce those measurements. Now derived from the checkout's
parent, overridable via `IX_WORKSPACE`. Re-run after the change: 237 repos, 0 punctuation targets —
consistent with the figure CR-067 records.

## Checks that passed

- **Test tracking tags** — every new test carries its TC id and every cited id resolves in
  `spec/tests.md`. TC-024 was *added* to a property that had been running untagged since the parser
  landed (CR-069).
- **Seam compliance** — no `#[cfg(test)]` branch in a production path, no test-only feature flag,
  no test replacing the unit under test with a double.
- **Tautological assertions** — each new property was checked against "what change to the source
  makes this fail?". Four of them answered it by *failing on real defects* during authoring
  (CR-069), and one property was found to be wrong and corrected rather than the code.
- **Integrity** — no `#[allow]` added, no gate weakened, no coverage threshold moved, no
  `#[ignore]`, no `unsafe`. `#![forbid(unsafe_code)]` intact.
- **Determinism** — no `HashMap` introduced in `src/corpus/`; `BTreeMap`/`BTreeSet` throughout,
  and TC-887 asserts a severity map does not perturb finding order.
- **Numeric conversions** — no `as` casts added at any boundary.
- **Gates run, not assumed** — `make ci` (fmt-check, clippy `-D warnings`, check-python, test,
  deny, audit-unsafe, audit-property, audit-static) and `make ci-python` both green on the
  integrated branch after CR-072. `cargo clippy` initially failed on three `needless_borrow`
  warnings introduced by the FND-001 fix; corrected and re-run.

## Gap analysis

One gap found, filed rather than fixed: **agent-ix/quire-rs#171** — there is no
requirement→production-code relation, so mutation scoping reaches only 38 of 52 FRs. Documented in
CR-071 with the measurement, and the ticket carries CR-061's constraint so the fix cannot become a
coverage backdoor.
