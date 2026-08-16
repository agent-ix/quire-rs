#!/usr/bin/env bash
# check_no_shared_mutable.sh — FR-024-AC-9 (amended CR-047, hardened CR-053)
#
# TC-502: this script IS the enforcement identity of FR-024-AC-9's
# "Inspection" verification — it runs in `make audit-static`, in `make ci`,
# and in the ci.yml `audit-static` job.
#
# Enforce: the parallel-walk fan-out is LOCK-FREE. The parse fan-out is
# data-parallel — each task returns an owned result and the results are
# collected (`par_iter().map().collect()`), never pushed into a shared
# buffer; diagnostics are gathered after the parallel region. This is the
# invariant that makes the NFR-017 loom check valid and keeps the engine
# free of locks/atomics on the hot concurrency path.
#
# Scoped to src/corpus/ and src/python/ — the two modules that run rayon.
# (src/python was added by CR-053: `python::load_repo` opens its own rayon
# region over corpus state, so an audit blind to it could not see the one
# place first-touch actually happens in parallel.)
#
# Bans Mutex, RwLock, the std::sync::atomic family, OnceLock/OnceCell
# (CR-047) and, since CR-053, LazyLock, `once_cell::sync::Lazy`,
# Cell/RefCell, `thread_local!`, `static mut` and `unsafe impl Sync`.
#
# The ONLY sanctioned interior mutability is the named exemption list below.
# An exemption states WHAT is exempt and WHY, and three rules keep it from
# rotting into a silent suppression:
#
#   1. `file` is a repo-relative PATH, not a basename — a future
#      `src/corpus/deep/body_cache.rs` inherits nothing.
#   2. `match` is the EXACT trimmed source line, not a substring — a new
#      `OnceLock` in an already-exempt file fails until someone justifies
#      that line. (Reformatting the line also fails: re-justifying an
#      exemption you just rewrote is the point.)
#   3. An exemption matching NOTHING fails as STALE — delete the code and
#      the entry cannot rot on silently.
#
# Every exemption applied is printed with its `why`, so `make ci` output
# records what is sanctioned rather than leaving it to a reader of this file.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SCOPES=("$ROOT/src/corpus" "$ROOT/src/python")

present=()
for scope in "${SCOPES[@]}"; do
  [[ -d "$scope" ]] && present+=("$scope")
done
if [[ "${#present[@]}" -eq 0 ]]; then
  echo "check_no_shared_mutable: no audited module present yet — skipping (OK)."
  exit 0
fi

# Type references, not substrings of larger identifiers. Covers
# `std::sync::Mutex`, bare `Mutex<`, `AtomicUsize`, `OnceLock<...>`,
# `OnceLock::new()`, `LazyLock`, `once_cell::sync::Lazy`, `RefCell<`,
# `Cell<`, `thread_local!`, `static mut` and `unsafe impl Sync`.
PATTERN='(::Mutex[^A-Za-z0-9_]|::RwLock[^A-Za-z0-9_]|[^A-Za-z0-9_]Mutex<|[^A-Za-z0-9_]RwLock<|::Atomic[A-Za-z0-9]+|[^A-Za-z0-9_]Atomic[A-Za-z0-9]+<|::Once(Lock|Cell)[^A-Za-z0-9_]|[^A-Za-z0-9_]Once(Lock|Cell)<|::LazyLock[^A-Za-z0-9_]|[^A-Za-z0-9_]LazyLock<|once_cell::sync::Lazy|[^A-Za-z0-9_](Ref)?Cell<|thread_local!|static[[:space:]]+mut[[:space:]]|unsafe[[:space:]]+impl[[:space:]]+([A-Za-z0-9_]+[[:space:]]+for[[:space:]]+)?Sync)'

# Named exemptions, one per line: repo-relative-path|exact-trimmed-line|why
EXEMPTIONS=(
  "src/corpus/body_cache.rs|pub(crate) struct LazyBody(std::sync::OnceLock<QuireDocument>);|FR-025 lazy body tier: per-document once-init cell behind Arc<SpecInner>; not walk state — the FR-024 rayon fan-out builds every LoadedDocument with an empty cell and parses no body. Exactly-once + agreed-value proven by the NFR-017 loom model (TC-815) and raced for real under TSAN (tests/corpus_concurrency.rs, TC-816)."
  "src/corpus/declared_tables.rs|static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();|compile-once static regex, idempotent deterministic init, outside the parallel region (pre-existing; CR-047's widened pattern made it visible instead of silent)."
)

violations=()
used=()
for _ in "${EXEMPTIONS[@]}"; do used+=(0); done

while IFS= read -r hit; do
  [[ -n "$hit" ]] || continue
  file="${hit%%:*}"
  rel="${file#"$ROOT"/}"
  rest="${hit#*:}" # "<line>:<text>"
  text="${rest#*:}"
  trimmed="$(printf '%s' "$text" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"

  exempt=0
  for i in "${!EXEMPTIONS[@]}"; do
    entry="${EXEMPTIONS[$i]}"
    e_file="${entry%%|*}"
    e_rest="${entry#*|}"
    e_match="${e_rest%%|*}"
    if [[ "$rel" == "$e_file" && "$trimmed" == "$e_match" ]]; then
      exempt=1
      used[$i]=1
      break
    fi
  done
  [[ "$exempt" -eq 0 ]] && violations+=("$rel:${rest%%:*}: $trimmed")
done < <(grep -RIn --include='*.rs' -E "$PATTERN" "${present[@]}" || true)

failed=0

if [[ "${#violations[@]}" -gt 0 ]]; then
  echo "check_no_shared_mutable: FAIL — un-exempted shared-mutable/interior-mutability sync (FR-024-AC-9):" >&2
  printf '  %s\n' "${violations[@]}" >&2
  echo "  The parallel parse must collect owned results — no Mutex/RwLock/Atomic." >&2
  echo "  Interior mutability outside the walk requires a NAMED exemption in this script" >&2
  echo "  (path|exact-trimmed-line|why), stating what is exempt and why — never a silenced pattern." >&2
  failed=1
fi

stale=()
for i in "${!EXEMPTIONS[@]}"; do
  [[ "${used[$i]}" -eq 0 ]] && stale+=("${EXEMPTIONS[$i]%%|*}: ${EXEMPTIONS[$i]#*|}")
done
if [[ "${#stale[@]}" -gt 0 ]]; then
  echo "check_no_shared_mutable: FAIL — STALE exemption(s) matching nothing:" >&2
  printf '  %s\n' "${stale[@]}" >&2
  echo "  The code they sanction is gone or was rewritten. Delete the entry, or" >&2
  echo "  update it to the line that exists now — an exemption nobody re-reads is" >&2
  echo "  how a silent suppression starts." >&2
  failed=1
fi

[[ "$failed" -eq 1 ]] && exit 1

echo "check_no_shared_mutable: OK — sanctioned interior mutability:"
for i in "${!EXEMPTIONS[@]}"; do
  entry="${EXEMPTIONS[$i]}"
  echo "  ${entry%%|*}"
  echo "    why: ${entry##*|}"
done
