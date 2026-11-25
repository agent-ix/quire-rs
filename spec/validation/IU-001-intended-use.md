---
id: IU-001
type: IntendedUse
title: Intended use of quire-rs
tool: quire-rs
relationships: []
---

# Intended use of quire-rs

## Purpose

`quire-rs` is a Rust library for parsing and validating Quire documents and
computing repository-level traceability and coverage information. It is used
as an engine by command-line and application layers that present or retain
those results.

## Records produced

The library returns parsed `QuireDocument` values, document validation results,
repository `BundleReport` values, and `CoverageReport` and coverage-matrix
values. These are structured results returned to the caller; `quire-rs` does
not itself write them as files. In the `quire` command-line application,
`quire validate` presents validation results and `quire matrix` presents
coverage results. The command-line writer is supplied by `quire-cli`.

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
