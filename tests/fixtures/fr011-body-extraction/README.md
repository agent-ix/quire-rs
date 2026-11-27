# FR-011 body-extraction input corpus

This directory snapshots only data from the five `spec-objects-*` repositories
named by FR-011-AC-5. It contains the original module manifests, all tracked
JSON schemas from those module packages, and the Markdown files under each
repository's `tests/fixtures/`. It contains no source code and no generated
`ExtractionResult` values.

`inventory.json` records the exact source commit, source path and SHA-256 for
every copied file. Each body-extraction DSL is keyed by module and type, with a
canonical DSL hash and explicit fixture paths. A type with no fixture whose
frontmatter `object` or `type` matches it is marked `missing_fixture`; that is
an unresolved corpus gap, not a passing sample.

The origin/main source commits recorded here declare 72 object types in total:

| Module | Source commit | Object types |
| --- | --- | ---: |
| architecture | `5f9287ddb075b5422ae018e10f8e19f2c5a70e99` | 15 |
| business | `80af08c36306b7e7ad2f09d93fc3e69dfe54f3fd` | 13 |
| enterprise | `9aa8624316a0fff0ddc55d1b88c10fb8c3aaaa38` | 7 |
| operational | `68bee81a764909d1ba2cbd157df942ae9f7e89b7` | 9 |
| security | `817ae0e04e30e600ee3aaf8a47367c5d66573b50` | 28 |

The five manifests all declare `body_extraction` for these 72 types. The
inventory maps existing fixture documents to 48 types; 24 types have no
matching source fixture document and remain explicitly flagged as that gap.
Each of those 24 types also has one generated minimal Markdown input under its
module's `generated/` directory. The generated inputs contain concrete values
for every locator in the pinned DSL, including section-scoped fences and the
required table columns where used. Their provenance comment records the source
module commit, manifest path and hash, DSL hash, generated-only origin, and a
payload content hash. The inventory records a separate hash over each complete
generated file. Across source and generated inputs, all 72 DSLs now have an
exercisable input. Generated inputs have no expected extraction records.

All tracked module schema files are included. Direct manifest schema paths and
relative schema `$ref`s resolve inside the snapshot. The inventory lists
external canonical schema IDs that are not present in these five repositories
as schema-closure gaps. They currently point into 27 distinct versioned
`semantic-core` schema IDs. They are enumerated rather than fetched from an
unversioned source.

This is input data for a future independent oracle. It is not an oracle, does
not assert parity, and does not close FR-011-AC-5. The test checks source and
generated file hashes, manifest-to-inventory completeness, schema closure
gaps, and whether generated documents provide concrete locator substrates. It
does not assert extraction output. The independent Tier-2 DSL evaluator remains
an external dependency.
