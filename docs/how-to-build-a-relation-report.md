---
title: Build a relation coverage report
type: how-to
description: Build a relation coverage report from a corpus and its loaded module.
---

# Build a relation coverage report

Use a relation coverage report to see which documents satisfy a module's declared relations and how their criteria are covered.

## Before you start

You need a module that declares `traceability.required_relations`, a corpus with a `spec/` directory, and a coverage matrix for each corpus member. The matrix records each criterion's test coverage status.

## Steps

1. **Declare the relation.** This example selects every `Concern` and looks for an incoming `addresses` edge from a `Rule`:

   ```yaml
   required_relations:
     - name: concern-is-addressed
       from: Concern
       edges: [addresses]
       to: [Rule]
       direction: incoming
       check: unaddressed-concern
   ```

   The module must also declare the `Concern` and `Rule` archetypes and allow the `addresses` link.

2. **Compute coverage and build the report.** Pass one `RelationReportMember` for each corpus member. Its matrix comes from the same coverage computation used to report criterion coverage.

   ```rust
   use quire_rs::coverage::compute;
   use quire_rs::corpus::{relation_report, RelationReportMember, Spec};
   use quire_rs::symbols::{extract_tree, trace};
   use quire_rs::Registry;
   use std::path::Path;

   fn report_json(repo: &Path, module: &Path) -> Result<String, String> {
       let registry = Registry::load_module(module).map_err(|error| error.to_string())?;
       let spec = Spec::from_path(&repo.join("spec"));
       let model = registry.traceability().cloned().unwrap_or_default();
       let extracted = extract_tree(&repo.join("src"));
       let graph = trace::bind(&extracted, &model);
       let coverage = compute(&spec, &registry, &graph, repo)
           .map_err(|error| error.to_string())?;
       let members = [RelationReportMember {
           root: repo,
           matrix: &coverage.coverage_matrix,
       }];
       let report = relation_report(&spec, &registry, &members, &[])
           .map_err(|error| error.to_string())?;
       serde_json::to_string_pretty(&report).map_err(|error| error.to_string())
   }
   ```

3. **Read the rows.** `state` says whether the declared relation is satisfied. `verification` summarizes the row's document and linked documents using the supplied coverage matrix.

## Expected output

For an unlinked concern, the row reports:

```text
state: uncovered
verification: not-tracked
```

## Verify the result

Run the report after adding an `addresses` link from a `Rule` to that concern. The row changes to `covered`; its `linked` array names the rule. Its verification follows the rule's criteria in the coverage matrix.

From the repository root, run the focused integration checks:

```sh
cargo test --locked --test it relation_report
```
