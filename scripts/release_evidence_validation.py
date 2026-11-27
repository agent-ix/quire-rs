"""Shared validation helpers for release evidence packaging and verification."""

import re


# implements: FR-095-AC-6
def missing_required_release_bindings(suite_text, run_store, run_record):
    """Return required obligations lacking at least one passing supporting test."""
    required_range = re.search(
        r"`?SUITE-001`?\s+SHALL\s+bind each acceptance criterion in Quire-rs FR-095\s+\(AC-1 through AC-(\d+)\)",
        " ".join(suite_text.split()),
    )
    if required_range is None:
        raise ValueError(
            "release suite registry does not declare its required FR-095 bindings"
        )

    required_obligations = {
        f"FR-095-AC-{number}" for number in range(1, int(required_range.group(1)) + 1)
    }
    bindings = [
        entry
        for entry in run_store.get("bindings", [])
        if isinstance(entry, dict) and entry.get("suite") == "SUITE-001"
    ]
    passing_symbols = {
        entry.get("symbol")
        for entry in run_record.get("entries", [])
        if isinstance(entry, dict) and entry.get("outcome") == "pass"
    }
    missing = []
    for obligation in sorted(required_obligations):
        matching = [
            entry for entry in bindings if entry.get("obligation") == obligation
        ]
        if not matching:
            missing.append(obligation)
            continue
        supported_symbols = {
            symbol
            for entry in matching
            for symbol in (
                entry.get("symbols") if isinstance(entry.get("symbols"), list) else []
            )
            if isinstance(symbol, str)
        }
        if not supported_symbols.intersection(passing_symbols):
            missing.append(
                f"{obligation} (no passing supporting test: "
                + ", ".join(sorted(supported_symbols))
                + ")"
            )
    return missing
