# Render Parity Fixture Provenance

Records the upstream source-of-truth versions used to generate the `expected.md` fixtures in this directory.

The fixtures encode byte-exact ground truth from the Python Jinja2 reference renderer. If any layer below changes, the fixtures MUST be regenerated.

## Pin

| Layer | Version | Source |
|---|---|---|
| Python interpreter | 3.13.x | `scripts/regenerate_parity_fixtures.sh` invokes `python3.13` explicitly |
| Jinja2 | 3.1.4 | `scripts/parity-venv.txt` |
| PyYAML | 6.0.2 | `scripts/parity-venv.txt` |
| jsonschema (Python validator, for input validation only) | 4.23.0 | `scripts/parity-venv.txt` |
| `spec-artifacts-iso` renderer code | TBD — record git SHA at first regeneration | `~/dev/spec-artifacts-iso` |
| `spec-artifacts-app` renderer code | TBD | `~/dev/spec-artifacts-app` |
| `spec-artifacts-process` renderer code | TBD | `~/dev/spec-artifacts-process` |

The TBD entries above are filled in by `scripts/regenerate_parity_fixtures.sh` on each run, replacing the line in-place.

## Regeneration

```bash
bash scripts/regenerate_parity_fixtures.sh
```

The script:
1. Creates / activates the pinned venv from `scripts/parity-venv.txt`.
2. Walks `tests/render_parity/corpus.yaml`.
3. For each `(archetype, input.json)` pair, invokes the Python reference renderer.
4. Writes `expected.md`.
5. Updates this PROVENANCE.md with the spec-artifacts-* git SHAs at regeneration time.

## CI staleness check

CI compares the SHA recorded here against `git -C ~/dev/spec-artifacts-iso rev-parse HEAD`. If the recorded SHA is older than the working tree's, the parity suite is run against the new SHA and the diff is surfaced. A non-zero diff fails the build with a "regenerate fixtures" instruction.
