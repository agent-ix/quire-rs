"""Make `scripts/` importable under pytest from any rootdir/cwd (#217, #219).

The scripts are executables first: run directly, CPython puts `scripts/` on
`sys.path` and their flat `import corpus` resolves. Pytest imports them as
modules from wherever it was invoked, so the same path entry is added here,
once, before any test module imports one.
"""

import sys
from pathlib import Path

SCRIPTS_DIR = str(Path(__file__).resolve().parents[1])
if SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SCRIPTS_DIR)
