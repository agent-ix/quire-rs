---
id: SR-110
title: "Independent code review — exact Rust 1.98.1 qualification at 05e0e4d"
type: SpecReview
analysis: code-review
scope: "PR #422 at 05e0e4d; NFR-022; scripts/audits/check_tool_drift.sh; scripts/tests/test_tool_drift.py; .github/workflows/; src/ Clippy and rustdoc repairs; Cargo.lock; TC-1832..TC-1835"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-022"
    type: reviews
---

## Summary

This is the strongest of the four changes under review and it reproduces
completely: every declaration agrees on exact 1.98.1, the full locked suite is
1,031 passing with 0 failed and 0 ignored, strict rustdoc is clean, and
`cargo audit` reports zero advisories across 219 dependencies. The
source diff is 20 files of `Option::map_or(true, f)` → `Option::is_none_or(f)`
and rustdoc-link repairs — every one behaviour-identical, with no `#[allow]`
added anywhere. The strengthened drift audit is real: I killed it with seven
independent mutants. Two holes remain in the surface it scans, and the audit
this PR strengthens is one of four `scripts/audits/*.sh` the hosted job does
not run.

## Verdict

**CONDITIONAL** — no high findings. Two mediums, both about the reach of a gate
rather than the correctness of the change.

## Gates run at `05e0e4d`

Exact Rust 1.98.1, `-j 2`, isolated target directory, `corpus/` submodule
initialised.

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | pass |
| `cargo test --locked` | pass — **1,031 passed, 0 failed, 0 ignored**, of which 609 are library tests |
| `RUSTDOCFLAGS=-D warnings cargo doc --all-features --no-deps --locked` | pass |
| `cargo deny check` | advisories, bans, licenses, sources ok |
| `cargo audit` | pass — 219 crate dependencies, **zero advisories** |
| `cargo check --locked --features python` | pass |
| `make audit-static` | pass — all 11 scripts, including `check_tool_drift.sh` |
| `bash scripts/audits/check_tool_drift.sh` | pass |

## Independently verified claims

| Claim | Check | Result |
| --- | --- | --- |
| Every declaration is exact 1.98.1 | `Cargo.toml` `rust-version`, `rust-toolchain.toml` `channel`, `clippy.toml` `msrv`, and all seven stable `toolchain:` keys across six workflows | **all 1.98.1**; `sanitize.yml` and `fuzz.yml` correctly retain exact-dated `nightly-2026-08-27` |
| RUSTSEC-2026-0190 is real and 1.0.103 patches it | read the advisory in the local RustSec DB | confirmed — `informational = "unsound"`, `anyhow::Error::downcast_mut` affected `< 1.0.103`, `patched = [">= 1.0.103"]` |
| `anyhow` is not a library dependency | `cargo tree -i anyhow` | transitive through `jsonschema` only; the rust-review rule against `anyhow` on a crate boundary does not apply |
| No `#[allow]` was added to make the change pass | `git diff origin/main..HEAD \| grep -c '^+.*#\[allow'` | **0** |
| The `is_none_or` rewrites are behaviour-identical | read all 12 sites | `map_or(true, f)` and `is_none_or(f)` are the same function on `Option`; the two hand-written `match … None => true` forms in `src/symbols/rust.rs` and `tests/corpus_case/grading.rs` collapse identically, and both MSRV comments were correctly deleted with them |
| The rustdoc changes do not hide a broken link | strict-docs gate above | pass; the de-linked targets (`has_glossary_heading`, `BINDINGS`, `mask_code_spans`, `body_offset_in`, `run_grammar`, `BUILTIN_AMBIGUOUS`) are all private items rustdoc cannot link to from public docs, so demoting them to code spans is the correct repair rather than a suppression |

## Mutation evidence

Each mutant applied to `05e0e4d` and reverted; the tree was clean before and
after. The PR's central claim is "old numeric pins and floating stable fail the
strengthened drift audit", and on every surface the audit scans, it does.

| # | Mutant | `check_tool_drift.sh` |
| --- | --- | --- |
| M1 | `rust-toolchain.toml` channel → `1.97.1` | **killed** — "must select exact Rust 1.98.1" |
| M2 | `rust-toolchain.toml` channel → `stable` | **killed** — same |
| M3 | `clippy.toml` msrv → `1.97.1` | **killed** — "must declare Clippy MSRV 1.98.1" |
| M4 | `Cargo.toml` rust-version → `1.97.1` | **killed** — "must declare minimum supported Rust 1.98.1" |
| M5 | `loom.yml` `toolchain: stable` | **killed** — "stable Rust 1.98.1 is required" |
| M6 | `loom.yml` `toolchain: 1.94.1` | **killed** — same |
| M7 | `Makefile` recipe `cargo +stable build --locked` | **killed** — "stable Rust build selection must be 1.98.1" |
| M10 | `Makefile` recipe `cargo +1.94.1 build --locked` | **killed** |
| M11 | new `scripts/zz.sh` with `cargo +stable build` | **killed** |
| M12 | new `scripts/audits/zz.sh` with `RUSTUP_TOOLCHAIN=stable` | **killed** |
| M8 | workflow step `run: cargo +stable test --locked` | **SURVIVED** |
| M9 | `Makefile` recipe `rustup default nightly` | **SURVIVED** |

Ten of twelve killed, including every axis the PR body names. M8 and M9 are
FND-001 and FND-003.

Worth recording separately: `scripts/tests/test_tool_drift.py` already carries
its own parametrized mutation controls for seven of these, so the gate's
discrimination is tested in-repo and not only by this review. That is the right
pattern and it is why ten of my twelve mutants died on the first try.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The drift audit never scans workflow `run:` steps, so a floating `cargo +stable` inside a CI step passes | scripts/audits/check_tool_drift.sh:182 | correct-requirement-no-evidence |
| FND-002 | medium | The hosted `Static Audits` job enumerates 7 of the 11 `scripts/audits/*.sh` and omits `check_tool_drift.sh` — the audit this PR strengthens | .github/workflows/ci.yml:246 | missing-requirement |
| FND-003 | low | A bare `rustup default nightly` in the Makefile or a script is not recognised as a toolchain selection at all | scripts/audits/check_tool_drift.sh:182 | correct-requirement-no-evidence |
| FND-004 | low | TC-1834 is marked ✅ on a hypothetical transition, and NFR-022-AC-3's seven-day clock has no trigger | spec/tests.md:851 | correct-requirement-no-evidence |

## Finding detail

### FND-001 — the audit reads workflows for `toolchain:`, not for what a step runs

The new `stable_selections` block scans the Makefile plus every `.py` and `.sh`
under `scripts/` (minus `scripts/tests/` and the audit itself). Workflow YAML is
scanned by a *different* branch, which reads only the `toolchain:` key in the
five lines after a `rust-toolchain` action. Nothing applies the selection
patterns to a workflow's `run:` block.

Measured — adding one step to `.github/workflows/loom.yml`:

```yaml
      - name: zz mutant
        run: cargo +stable test --locked
```

`bash scripts/audits/check_tool_drift.sh` exits **0**.

NFR-022-AC-1 requires that "every stable CI/build declaration selects exact
1.98.1; no stable 1.94.1 or floating `stable` selection remains", and the
requirement's own Verification section says the static test "rejects old or
floating stable selections". A workflow step is the most likely place for one
to appear — `dtolnay/rust-toolchain` is not the only way a job picks a
compiler — and it is the one place the audit looks past.

`build_scripts` is already a list. Adding `(root / ".github/workflows").glob("*.yml")`
to it closes this, and M5/M6 show the file type is already parsed.

### FND-002 — the strengthened audit has no hosted enforcement

`scripts/audits/` holds eleven scripts. The `audit-static` job in
`.github/workflows/ci.yml` names seven of them, one `run:` step each. The four
it does not name are:

```
check_semantic_boundary.sh
check_spec_structure.sh
check_status_agreement.sh
check_tool_drift.sh      <- strengthened by this PR
```

`make audit-static` does run all eleven — I confirmed it, and the Makefile
carries a long comment explaining why:

> This target ENUMERATED seven scripts. CR-124 asserted it "runs every
> `scripts/audits/*.sh`"; it never did, so the gate answering the outside
> review's [P1] sat in the tree unrun from the commit that added it until #353
> added an eighth line by hand. […] So the list is gone. A new
> `scripts/audits/*.sh` now runs because it exists, not because somebody
> remembered.

The hosted job is the same defect, in the same repository, one file away, and
it is the one that omits the audit this PR exists to strengthen. The Makefile's
own reasoning applies verbatim, including the empty-glob guard: a `for` loop
over no scripts exits 0 and looks exactly like every audit passing.

This does not weaken the change — `make ci` includes `audit-static`, so the
local gate the PR body reports is genuine. It means the drift audit's hosted
enforcement does not exist, which is worth knowing before NFR-022-AC-1 is
treated as continuously held.

### FND-003 — `nightly` is not in the selection alternation

The workflow branch requires a nightly to carry an exact date:

```python
elif installed.group(1).startswith("nightly-"):
    if not re.fullmatch(r"nightly-\d{4}-\d{2}-\d{2}", installed.group(1)):
```

The script/Makefile branch has no equivalent. Its three patterns match only
`(stable|\d+\.\d+\.\d+)`, so `rustup default nightly` is not a selection the
audit recognises — measured, exit 0 with that line appended to the Makefile.

Low because the repository's nightly lanes are workflow-declared and correctly
dated, so nothing today exploits it. It is one alternation — `(stable|nightly|nightly-\S+|\d+\.\d+\.\d+)`
with the same exact-date rule — and it makes the two branches state the same
policy.

### FND-004 — an inspection AC marked passing on a hypothetical

NFR-022-AC-3 requires that "within seven calendar days after a newer stable
Rust release, the owner records the same compatibility matrix and either
advances all stable declarations together or records an AC-4 hold".

TC-1834 is marked `✅` with the evidence "Hypothetical 1.99.0 transition
inspected in SR-106". Inspecting a hypothetical establishes that the *procedure*
is well-formed, which is what an `inspection` verification method can do — the
AC's own choice of method is honest and I am not asking for a test here.

What the `✅` implies and the evidence does not carry is the recurring
obligation. Nothing observes a Rust release, nothing dates the last recorded
matrix, and nothing will fail on day eight. A one-line dated record — the
release observed, the date, the outcome — would make the clock inspectable
rather than remembered, and NFR-022's Dependencies section already names
"Rust 1.98.1, released 2026-09-01" as exactly that kind of observation.

## Checked and found harmless

- **The lockfile advances from `version = 3` to `version = 4`**, which the PR
  body does not mention in a scope statement that otherwise enumerates its lock
  change precisely. I expected this to raise the minimum Cargo that can read the
  lock and filed it as a finding before measuring. It does not:
  `cargo +1.75.0 metadata --locked` reads the v4 lock and exits 0. No
  consequence, so no finding — recorded only so the next reader does not repeat
  the hypothesis.
- **`cargo deny check` emits a wildcard-dependency warning** for the
  `ix-trace-rs` git dependency, which carries no version requirement. Pre-existing,
  non-failing, and `bans` still reports ok.
- **`make audit-static` reports 52 status-agreement findings.** They are
  explicitly advisory — "Advisory until calibrated: #347 measured 22 rows and
  this reads 52. Promotion to a gate is a separate decision." — and the target
  still exits 0. SR-106 describes this accurately.
- **`sanitize.yml` and `fuzz.yml` keep `nightly-2026-08-27`.** NFR-022's Scope
  explicitly puts separately pinned nightly lanes outside the stable-version
  declaration, and the audit's nightly branch requires the exact date. Correct
  on both counts.

## On the retained reviews

SR-106 and SR-107 hold up against independent measurement. SR-106's FND-4171
(RUSTSEC-2026-0190 on locked `anyhow` 1.0.102) is a real high correctly found
and closed; its qualification matrix matches what I reproduced, including the
609-library-test figure and the nightly-only rustfmt warning. SR-107's claim
that the `is_none_or` rewrites are "ownership-neutral, allocate nothing,
preserve short-circuiting" is accurate at all twelve sites.

One sentence in SR-107 is wider than the evidence: "The source-inspection audit
… is mutation-tested against the old numeric pins and floating `stable`." That
is true of the Makefile, the scripts, and the workflow `toolchain:` key, and
untrue of a workflow `run:` step — which is FND-001.
