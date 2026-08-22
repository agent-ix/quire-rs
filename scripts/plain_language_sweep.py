#!/usr/bin/env python3
"""Run the compiled FR-063 fit check over the canonical repository corpus."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

from corpus import repos


def main() -> int:
    dev_root = Path(__file__).resolve().parents[2]
    selected = repos(dev_root)
    env = dict(os.environ)
    env.setdefault("CARGO_TARGET_DIR", "/tmp/quire-rs-fr063-sweep-target")
    subprocess.run(
        [
            "cargo",
            "run",
            "--quiet",
            "--example",
            "fr063_fit_check",
            "--",
            *(str(repo) for repo in selected),
        ],
        cwd=Path(__file__).resolve().parents[1],
        env=env,
        check=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
