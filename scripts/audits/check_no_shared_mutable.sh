#!/usr/bin/env bash
# check_no_shared_mutable.sh — FR-024-AC-9 (amended CR-047), TC-502
#
# Enforce: the parallel-walk fan-out is LOCK-FREE. The parse fan-out is
# data-parallel — each task returns an owned result and the results are
# collected (`par_iter().map().collect()`), never pushed into a shared
# buffer; diagnostics are gathered after the parallel region. This is the
# invariant that makes the NFR-017 loom check valid and keeps the engine
# free of locks/atomics on the hot concurrency path.
#
# Scoped to src/corpus/ (the only module that uses rayon). Bans Mutex,
# RwLock, the std::sync::atomic family, AND (since CR-047) OnceLock/OnceCell
# in first-party source. The ONLY sanctioned interior mutability in
# src/corpus is the named exemption list below — an exemption says WHAT is
# exempt and WHY, so nothing is suppressed silently. Matching is on file +
# matched text, never line numbers.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SCOPE="$ROOT/src/corpus"

if [[ ! -d "$SCOPE" ]]; then
  echo "check_no_shared_mutable: src/corpus/ not present yet — skipping (OK)."
  exit 0
fi

# Match Mutex, RwLock, Atomic*, and OnceLock/OnceCell type references (not
# substrings of larger identifiers). Covers `std::sync::Mutex`, bare
# `Mutex<`, `AtomicUsize`, `OnceLock<...>`, `OnceLock::new()`, etc.
PATTERN='(::Mutex[^A-Za-z0-9_]|::RwLock[^A-Za-z0-9_]|[^A-Za-z0-9_]Mutex<|[^A-Za-z0-9_]RwLock<|::Atomic[A-Za-z0-9]+|[^A-Za-z0-9_]Atomic[A-Za-z0-9]+<|::Once(Lock|Cell)[^A-Za-z0-9_]|[^A-Za-z0-9_]Once(Lock|Cell)<)'

# Named exemptions, one per line: file|match-substring|why
# A hit is exempt only when its basename equals `file` AND the matched line
# contains `match-substring`. Anything else fails, with the lines echoed.
EXEMPTIONS=(
  "body_cache.rs|OnceLock<QuireDocument>|FR-025 lazy body tier: per-document once-init cell behind Arc<SpecInner>; not walk state — the rayon fan-out never touches it; exactly-once + agreed-value proven by the NFR-017 loom model (TC-815) and raced for real under TSAN (tests/corpus_concurrency.rs)."
  "declared_tables.rs|OnceLock|compile-once static regex, idempotent deterministic init, outside the parallel region (pre-existing; the widened pattern makes it visible instead of silent)."
)

violations=()
while IFS= read -r hit; do
  [[ -n "$hit" ]] || continue
  file="${hit%%:*}"
  base="$(basename "$file")"
  rest="${hit#*:}"   # "<line>:<text>"
  text="${rest#*:}"
  exempt=0
  for entry in "${EXEMPTIONS[@]}"; do
    e_file="${entry%%|*}"
    e_rest="${entry#*|}"
    e_match="${e_rest%%|*}"
    if [[ "$base" == "$e_file" && "$text" == *"$e_match"* ]]; then
      exempt=1
      break
    fi
  done
  if [[ "$exempt" -eq 0 ]]; then
    violations+=("$hit")
  fi
done < <(grep -RIn --include='*.rs' -E "$PATTERN" "$SCOPE" || true)

if [[ "${#violations[@]}" -gt 0 ]]; then
  echo "check_no_shared_mutable: FAIL — un-exempted shared-mutable/interior-mutability sync in src/corpus (FR-024-AC-9):" >&2
  printf '%s\n' "${violations[@]}" >&2
  echo "  The parallel parse must collect owned results — no Mutex/RwLock/Atomic." >&2
  echo "  Interior mutability outside the walk requires a NAMED exemption in this script" >&2
  echo "  (file|match-substring|why), stating what is exempt and why — never a silenced pattern." >&2
  exit 1
fi

echo "check_no_shared_mutable: OK"
