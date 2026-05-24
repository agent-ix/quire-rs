# quire-rs

High-performance Rust templating + parsing engine for the Filament/Quire ecosystem.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check licenses
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make ci             # fmt-check + lint + test + deny + audit-unsafe
```

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.75` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping. CI fails on drift.
- `rust-toolchain.toml` pins to stable + rustfmt + clippy.

## Design taste

Write idiomatic Rust. Lean on the type system to encode invariants rather
than re-checking them at runtime: prefer enums + exhaustive `match` over
booleans + flags, `Option`/`Result` over sentinel values, newtypes over
raw strings/ints when the meaning matters, and `&str`/borrowed slices over
allocating clones on hot paths. Where the TS/Py references rely on dynamic
shapes (e.g. JSON `Value` blobs), the Rust port should look for a stronger
typed representation when it doesn't break parity. "Stringly-typed" code
and `unwrap()`s outside of tests are smells — surface errors via
`thiserror`-derived enums per NFR rather than panicking.

## Layout

```
src/lib.rs             # crate root
tests/integration.rs   # end-to-end tests
benches/               # criterion benchmarks (opt-in; add criterion to dev-deps)
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # local tooling
```
