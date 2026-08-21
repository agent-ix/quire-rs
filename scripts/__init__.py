"""`scripts/` as an importable package (#219).

The scripts are flat executables first: run directly
(`python3 scripts/slash_tag_sweep.py`), CPython puts `scripts/` itself on
`sys.path` and their `from corpus import …` sibling imports resolve from any
cwd. That entry does not exist under `python -m scripts.<name>` or under
pytest, so it is added explicitly here (for `-m`) and in
`scripts/tests/conftest.py` (for pytest) — the same one path decision, stated
instead of relied on by accident.
"""

import sys
from pathlib import Path

_SCRIPTS_DIR = str(Path(__file__).resolve().parent)
if _SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, _SCRIPTS_DIR)
