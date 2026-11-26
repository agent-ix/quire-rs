---
id: SR-20261115183672
title: "Code review — quire-rs dependency move analysis"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/impact.rs; spec/functional/FR-099-diff-impact-analysis.md; tests in src/impact.rs"
review_set: subset
---

## Summary

Reviewed PR #747 at frozen head `5dc7bb4139452d1e3a876165cc1b86775b050139`, focusing on the new dependency-move analysis and its FR-099-AC-10 tests. The implementation is deterministic and reports missing upstream snapshots, but prefix substring matching can report an unrelated local reference and select a test for the wrong upstream item.

## Verdict

**CONDITIONAL** — one medium severity matching defect remains; focused impact tests pass. The full crate test run has an unrelated temporary-path failure, standard Clippy stops on an unchanged warning, and `cargo deny check` could not access its advisory lock under the read-only Cargo home.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Prefix substring matches are treated as exact upstream item references, so a changed `TC-37` can match a local `FSpec-TC-374` test, add a false test reason, and suppress the unmatched change from `uncovered`. | src/impact.rs:365-380 |

## Review notes

`FR-099-AC-10` is implemented by the changed API and helpers; the two new tests carry `#[trace("FR-099-AC-10")]`, and the computed matrix binds both. The FSL revision pair is `3200a4ed7f18fbbf5d934c22d3c3fa467914ddca` → `691ac5a6748edc324f014e26ca8370eb15b127d5`; `git diff` reports 92 changed paths. The changed schema definitions include `CertificateIdentity` and `ProbabilityCertificate`; FSL's local `provider_definition` reads the backend-provider and checked-package schema paths. The two requested tests call that helper, matching the reported selection. Lane rows 960–961 record the historical recipe at 0/2; this review did not build or test FSL.

The call path's raw name matching is a conservative approximation; the concrete defect here is the use of `str::contains` for item identity. For example, when `item == "TC-37"`, attached source for a test tagged `FSpec-TC-374` contains the shorter string, so it is inserted as a reference and can also be selected as a test. `uncovered` then treats the prefix as a successful match.

Verification: `cargo fmt --check` passed; focused `cargo test -p quire-rs --lib impact::tests::` passed 15 tests. Full `cargo test -p quire-rs` had 794 passed and one failure at unchanged `src/loader/paths.rs:289` because canonical `/private/var/...` differed from expected `/var/...`. Focused `cargo clippy -p quire-rs --lib -- -D warnings -A clippy::field-reassign-with-default` passed. Full `cargo clippy --workspace --all-targets --all-features -- -D warnings` stops on unchanged `src/semantic/record.rs:806` (`field_reassign_with_default`); `git diff origin/main...HEAD` confirms both paths are outside the PR, and `git blame origin/main` confirms pre-existing lines. `cargo deny check` failed before checks because it could not acquire `/Users/peter/.cargo/advisory-dbs/db.lock` in the read-only Cargo path. No FSL build or tests were run.
