# YAML migration differential

This standalone audit package is the reproducible two-engine comparator for
ADR-0012. It deliberately keeps retired `serde_yaml` outside the root crate's
production, development, fuzz, and `Cargo.lock` graphs. Its own lockfile pins
both comparator engines.

Each input is declared as four arguments:

```text
--source <kind> <name> <revision> <path>
```

The comparator recursively enumerates Markdown, extracts only complete leading
frontmatter blocks, hashes every input, and compares old/new success or failure
and the complete `serde_json::Value`. It emits the complete input manifest and
result as JSON. Any difference exits non-zero. `--inject-difference` is solely
the documented negative control for proving that the gate blocks a semantic
difference.
