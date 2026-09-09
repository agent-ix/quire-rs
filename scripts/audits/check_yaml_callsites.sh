#!/usr/bin/env bash
# TC-1830 / NFR-009-AC-14: every YAML call site is classified.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
python3 "$ROOT/scripts/audits/check_yaml_callsites.py"
