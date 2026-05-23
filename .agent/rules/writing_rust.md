# Writing Rust in this repo

## Format & lint

- `make fmt` before committing. CI gate (`make fmt-check`) is `cargo fmt -- --check` — drift fails.
- `make lint` runs `cargo clippy --all-targets -- -D warnings`. Warnings are errors.
- 100-char max width (rustfmt). `StdExternalCrate` import grouping with crate-level `imports_granularity`.

## Unsafe

- Every `unsafe {` block must have a `// SAFETY: <reason>` comment within the 3 lines above it.
- `make audit-unsafe` enforces this. CI runs the same check.
- Pre-existing baselines (legacy unsafe without comments) live in `scripts/unsafe_comment_baseline.txt`. Regenerate with `bash scripts/check_unsafe_comments.sh --update-baseline`.

## Dependencies

- `make deny` enforces the allowlisted licenses in `deny.toml` (MIT, Apache-2.0, BSD-2/3, CDLA-Permissive-2.0, ISC, Unicode-3.0, Zlib).
- New crates with other licenses require an explicit `deny.toml` exception with a comment explaining why.
- Unknown registries and unknown git sources are denied. Add explicit entries to `[sources]` if you must.

## Tests

- Unit tests inline with `#[cfg(test)]`. Integration tests in `tests/`.
- Aim for property tests (`proptest`) on parsers / format encoders / anything with adversarial inputs.

## Don't

- Don't add `#[allow(...)]` to silence a clippy lint without a comment explaining the trade-off.
- Don't disable a CI gate to unblock a PR. Fix the underlying issue.
- Don't introduce `unsafe` without a `// SAFETY:` comment.
