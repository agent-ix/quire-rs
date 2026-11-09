---
id: SR-16362
title: "Integrity review of PLAT-362 shell symbol adapter (quire-rs PR 614)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md, spec/tests.md (PR 614 diff only, head 63ac3f0); context FR-051 unchanged sections, FR-062, src/symbols/mod.rs, src/traceability.rs"
review_set: subset
---

## Summary

Ticket: PLAT-362. This pass checked CR-213 for consistency with the FR-051 text it did not amend, with FR-062, and with the declared model in src/symbols/mod.rs and src/traceability.rs. The `entry_point` binding rule matches the code's complement rule (`binds_trace_ids` and `carries_implements`, src/symbols/mod.rs:96-121). But the PR patched CON-1 and left three normative passages that the shell adapter now contradicts. It also makes symbol extraction depend on module data, against AC-19 and AC-23.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The "Language adapters" statement is not amended, and the shell adapter contradicts it. The statement says every adapter is behind its own Cargo feature, that each "SHALL parse its file to a syntax tree with that language's grammar and SHALL read every symbol ... from the tree's nodes", and that "unparseable" means the grammar's tree carries an error. AC-29 still says the `wasm` build "drops all three adapter features". The spec does not say whether shell has a feature, whether it is available on wasm, or what "unparseable" means for a line-structural adapter (AC-39). Only CON-1 got the exception. | spec/functional/FR-051-source-symbol-extraction.md:60-77, spec/functional/FR-051-source-symbol-extraction.md:198, spec/functional/FR-051-source-symbol-extraction.md:206 |
| FND-002 | high | The "Trace-tag grammar" section is not amended, and AC-38 contradicts it. The section defines "a trace id in a line comment (`# TC-041`)" as the legacy class, which CON-3 retires. It also says markers are read "statically from the syntax tree — the decorator, attribute and call-argument nodes". AC-38 declares a `#` comment canonical, and the shell adapter has no syntax tree. No intended shell marker form is listed next to the Python, Rust and TypeScript ones, so nothing separates a canonical shell marker from the legacy bare-id comment it must outlive. | spec/functional/FR-051-source-symbol-extraction.md:113-143, spec/functional/FR-051-source-symbol-extraction.md:205, spec/functional/FR-051-source-symbol-extraction.md:164 |
| FND-003 | high | AC-36 mints an `entry_point` only when a module-declared `entry_paths` glob matches. That makes the extracted symbol population depend on module data. AC-19 says "the candidate population does not depend on declared patterns" (CR-142). AC-23 reports the symbol table "as the engine built it", with binding "not asked" when no module is supplied. Under AC-36 the same audit file has an evidence symbol with one module and none without. `extract_tree` today takes no grammar. Reconcile them: either amend AC-19 and AC-23 and give extraction the glob as input (as `source_exclude` is), or move the glob to the binding stage. | spec/functional/FR-051-source-symbol-extraction.md:203, spec/functional/FR-051-source-symbol-extraction.md:188, spec/functional/FR-051-source-symbol-extraction.md:191, src/symbols/mod.rs:336 |
| FND-004 | medium | "A shell function is a plain function and binds nothing." Under FR-062's complement rule a `Function` carries `implements` (`carries_implements` matches `Function` and `Container`, src/symbols/mod.rs:119-121). A `language: shell` implements form would therefore bind. The sentence should say "binds no `verifies`", or state that shell declares no implements channel. FR-062 point 3 lists the evidence kinds as `TestFunction`/`Benchmark`/`FuzzTarget`. It is not amended for `entry_point`. | spec/functional/FR-051-source-symbol-extraction.md:102-104, spec/functional/FR-062-implements-relation.md:50-52 |
| FND-005 | medium | AC-38 introduces `syntax: comment` (`TraceMarkerSyntax` has only `RustAttribute`, src/traceability.rs:1049) and a new `entry_paths` field. No grammar text states where they are legal. Is `syntax: comment` allowed on a non-shell form, or on the `implements` or `legacy` lists? Is `entry_paths` allowed on a shell `implements` form? AC-38 covers only "`entry_paths` on a non-shell form is refused". FR-062 ties `rust_attribute` to Rust forms, and no equivalent rule exists for `comment`. | spec/functional/FR-051-source-symbol-extraction.md:205, src/traceability.rs:1049-1081, spec/functional/FR-062-implements-relation.md:68-74 |

## Verdict

Examined: the Shell adapter section, FR-051-CON-1, FR-051-AC-34..40 and the CR-213 note, against FR-051's Symbols, Language adapters and Trace-tag grammar sections, AC-19, AC-22, AC-23, AC-29 and FR-062. Confirmed: an `entry_point` that is listed in `binds_trace_ids` and not in `carries_implements` is the FuzzTarget precedent (src/symbols/rust.rs:79 and :183) and satisfies FR-062. The release-order note is correct. `SourceLanguage` is a closed enum (src/traceability.rs:25), and the `TraceMarkerForm` wire struct is `deny_unknown_fields` and guards `syntax` by language. An older engine would therefore refuse `language: shell`, `syntax: comment` and `entry_paths` at load, so the engine must ship first. FND-001..003 must be fixed in this PR. Choosing tree-sitter-bash (SR-16360 FND-001) removes FND-001 and most of FND-002.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The Language adapters statement names four adapters with `shell-symbols` and says the shell adapter parses with `tree-sitter-bash` through `quire-code-extraction`. AC-29 says four adapter features. AC-45 defines an unparseable `.sh` file by the grammar tree. |
| FND-002 | fixed | The Trace-tag grammar lists the shell form (`# trace: FR-007-AC-01`, `language: shell`, `syntax: comment`) as canonical, read from comment nodes and not retired by CON-3. The legacy line-comment class is narrowed to Rust, Python and TypeScript. |
| FND-003 | fixed | AC-37 mints an `entry_point` for every `.sh` file with or without a module, and the Shell adapter text cites AC-19 and AC-23. `entry_paths` decides only eligibility to bind, at the binding stage (AC-38). |
| FND-004 | fixed | A shell function "binds no `verifies`, and carries `implements` only through a `language: shell` form the module declares in `trace_tags.implements`". FR-062 point 3 now lists `EntryPoint` among the evidence kinds. |
| FND-005 | fixed | AC-43 rejects `syntax: comment` off a shell form or in `trace_tags.legacy`, and rejects `entry_paths` off a shell `markers` form, each naming the key. It accepts a shell `implements` form, which carries no `entry_paths` and binds no `entry_point`. |
