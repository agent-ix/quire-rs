#!/usr/bin/env python3
"""Python Jinja2 reference renderer for the parity suite.

Reads tests/render_parity/corpus.yaml, finds every (archetype, input,
expected) triple under cases/<archetype>/, renders the corresponding
module's templates/<archetype>.md.j2 against the input JSON using
Jinja2's stock environment (matching MiniJinja's strict/no-include
config), and writes the result to <expected>.

Run via scripts/regenerate_parity_fixtures.sh, which sets up the
pinned venv first.
"""
from __future__ import annotations

import json
import pathlib
import sys

import jinja2
import yaml

ROOT = pathlib.Path(__file__).resolve().parent.parent
PARITY = ROOT / "tests" / "render_parity"


def main() -> int:
    corpus = yaml.safe_load((PARITY / "corpus.yaml").read_text())
    modules_by_name: dict[str, pathlib.Path] = {}
    for m in corpus["modules"]:
        modules_by_name[m["name"]] = (PARITY / m["path"]).resolve()

    # For now we assume the archetype's owning module is the first one
    # whose templates/<archetype>.md.j2 exists. That matches the
    # quire-rs first-wins resolution across modules.
    written = 0
    for case in corpus.get("cases", []):
        archetype = case["archetype"]
        input_path = (PARITY / case["input"]).resolve()
        expected_path = (PARITY / case["expected"]).resolve()
        template_source = locate_template(modules_by_name, archetype)
        if template_source is None:
            print(
                f"!! no template for archetype '{archetype}'; skipping",
                file=sys.stderr,
            )
            continue
        data = json.loads(input_path.read_text())
        env = jinja2.Environment(
            undefined=jinja2.StrictUndefined,
            autoescape=False,
            # keep_trailing_newline=False (default) to match MiniJinja,
            # which strips the trailing newline of the template source
            # at compile time.
            loader=None,  # disables {% include %} / {% extends %}
        )
        template = env.from_string(template_source)
        try:
            rendered = template.render(**data)
        except jinja2.UndefinedError as e:
            print(f"!! undefined while rendering {archetype}/{case['input']}: {e}",
                  file=sys.stderr)
            continue
        expected_path.parent.mkdir(parents=True, exist_ok=True)
        expected_path.write_text(rendered)
        print(f"wrote {expected_path.relative_to(ROOT)}")
        written += 1
    print(f"-- regenerated {written} fixture(s)")
    return 0


def locate_template(
    modules: dict[str, pathlib.Path], archetype: str
) -> str | None:
    candidates = [
        f"{archetype.lower()}.md.j2",
        f"{archetype}.md.j2",
    ]
    for module_root in modules.values():
        manifest_path = module_root / "manifest.yaml"
        if not manifest_path.exists():
            continue
        manifest = yaml.safe_load(manifest_path.read_text())
        for at in manifest.get("artifact_types", []) or []:
            if at.get("name") == archetype or at.get("name", "").lower() == archetype:
                tref = at.get("template_ref")
                if tref:
                    tpath = module_root / tref
                    if tpath.exists():
                        return tpath.read_text()
        # Fallback: glob templates/ for a filename match.
        for cand in candidates:
            tpath = module_root / "templates" / cand
            if tpath.exists():
                return tpath.read_text()
    return None


if __name__ == "__main__":
    sys.exit(main())
