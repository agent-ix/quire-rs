#!/usr/bin/env bash
# check_no_shared_mutable.sh — FR-024-AC-9, TC-502
#
# Enforce: the parallel-walk code path uses NO hand-written shared-mutable
# synchronization. The parse fan-out is data-parallel — each task returns an
# owned result and the results are collected, never pushed into a shared
# buffer. This is the invariant that makes the NFR-017 loom check valid and
# keeps the engine free of locks/atomics on the hot concurrency path.
#
# Scoped to src/corpus/ (the only module that uses rayon). Bans Mutex,
# RwLock, and the std::sync::atomic family in first-party source.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SCOPE="$ROOT/src/corpus"

if [[ ! -d "$SCOPE" ]]; then
  echo "check_no_shared_mutable: src/corpus/ not present yet — skipping (OK)."
  exit 0
fi

# Match Mutex, RwLock, and Atomic* type references (not substrings of larger
# identifiers). Covers `std::sync::Mutex`, bare `Mutex<`, `AtomicUsize`, etc.
PATTERN='(::Mutex[^A-Za-z0-9_]|::RwLock[^A-Za-z0-9_]|[^A-Za-z0-9_]Mutex<|[^A-Za-z0-9_]RwLock<|::Atomic[A-Za-z0-9]+|[^A-Za-z0-9_]Atomic[A-Za-z0-9]+<)'

if grep -REn --include='*.rs' -E "$PATTERN" "$SCOPE" >/dev/null 2>&1; then
  echo "check_no_shared_mutable: FAIL — shared-mutable sync in the parallel path (FR-024-AC-9):" >&2
  grep -REn --include='*.rs' -E "$PATTERN" "$SCOPE" >&2 || true
  echo "  The parallel parse must collect owned results — no Mutex/RwLock/Atomic." >&2
  exit 1
fi

echo "check_no_shared_mutable: OK"
