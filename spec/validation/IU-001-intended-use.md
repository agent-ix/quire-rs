---
id: IU-001
type: IntendedUse
title: Intended use of quire-rs
tool: quire-rs
relationships: []
---

# Intended use of quire-rs

## Purpose

`quire-rs` provides the Rust library engine used by Quire interfaces to derive
document and repository results from authored specification repositories.

## Records produced

The library returns structured results to its caller; it has no command that
writes result files. The `quire-cli` interface presents or emits the results
from these commands:

| Record kind | Command that writes it |
|---|---|
| Document and repository validation findings (`ValidationResult`, `BundleReport`) | `quire validate` (emits the result to standard output) |
| Repository coverage and traceability matrix (`CoverageReport`, coverage matrix) | `quire matrix` (emits the result to standard output) |

## Who relies on them

Users of the `quire` command-line application and applications built on the
library may rely on its parse, validation, and traceability results when
reviewing specification repositories. A person remains responsible for
interpreting those results and deciding what action to take.

## Limits

`quire-rs` is not a compliance oracle. Its results do not establish regulatory
compliance, product safety, or clinical safety, and do not make a product or
clinical safety decision. Results are limited to the supplied documents,
configuration, and supported checks.

## Operating context

The library reads local Markdown documents, repository files, and supplied
module schemas and manifests. It runs as Rust code in the calling process and
uses the host filesystem for repository inputs. The library has no network
client and does not invoke external executables at runtime. A caller such as
`quire-cli` supplies command-line handling and any persistence or display of
the returned results. Building the library uses the Rust toolchain and Cargo;
those are build-time tools, not runtime dependencies.
