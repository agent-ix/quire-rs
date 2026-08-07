//! Body-extraction DSL shape + structural validation (FR-011).
//!
//! Authors ship a `body_extraction:` block in their object-type
//! manifest. Each block has a `yield_pattern` that's either
//! `match: {key: Locator}` (single-yield, zero-or-one record) or
//! `iterate_over: { ... }` + `per_match: { ... }` (multi-yield, one
//! record per iteration unit) — but NEVER both (FR-011-AC-6).
//!
//! Load-time validation rejects DSLs that set both, set neither, or
//! contain unknown keys (FR-011-AC-7).

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::error::QuireError;
use crate::extract::locator::{Locator, LocatorAssert, LocatorKind, LocatorPrimitive};

/// One body-extraction DSL — the parsed form of `body_extraction:` in
/// an object-type manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionDsl {
    pub yield_pattern: YieldPattern,
    #[serde(default)]
    pub emit_edges: Option<Vec<EmitEdge>>,
}

/// `yield_pattern:` — single (`match`) XOR multi (`iterate_over` +
/// `per_match`). Authors set one or the other.
///
/// We deliberately do NOT use `#[serde(untagged)]` here; the
/// validator below picks one branch by which key is present, surfacing
/// the load error message itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YieldPattern {
    #[serde(default, rename = "match")]
    pub r#match: Option<IndexMap<String, Locator>>,
    #[serde(default)]
    pub iterate_over: Option<IterateOver>,
    #[serde(default)]
    pub per_match: Option<IndexMap<String, Locator>>,
}

/// `iterate_over:` — describes how the multi-yield evaluator picks
/// iteration units (one record per unit).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IterateOver {
    /// Heading path to the iteration root (e.g. `["Algorithms"]`).
    pub section_path: Vec<String>,
    /// What unit the evaluator iterates: child headings, list items,
    /// or table rows.
    pub kind: IterateKind,
    /// Heading depth (relative under `section_path`). Defaults to 1.
    #[serde(default)]
    pub depth: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IterateKind {
    Heading,
    ListItem,
    TableRow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmitEdge {
    #[serde(rename = "type")]
    pub edge_type: String,
    pub target: EdgeTarget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EdgeTarget {
    Static(String),
    Locator(Locator),
}

/// Structural validation: `match` XOR `iterate_over` + `per_match`
/// (FR-011-AC-6). The `archetype` argument is the owning archetype's
/// name — included in the error so DSL violations point at the right
/// declaration.
pub fn validate_dsl(archetype: &str, dsl: &ExtractionDsl) -> Result<(), QuireError> {
    let has_match = dsl.yield_pattern.r#match.is_some();
    let has_iter = dsl.yield_pattern.iterate_over.is_some();
    let has_per = dsl.yield_pattern.per_match.is_some();
    if has_match && (has_iter || has_per) {
        return Err(QuireError::DslValidationError {
            archetype: archetype.to_string(),
            reason: "match and iterate_over are mutually exclusive".to_string(),
        });
    }
    if !has_match && !has_iter {
        return Err(QuireError::DslValidationError {
            archetype: archetype.to_string(),
            reason: "yield_pattern must declare either match or iterate_over".to_string(),
        });
    }
    if has_iter && !has_per {
        return Err(QuireError::DslValidationError {
            archetype: archetype.to_string(),
            reason: "iterate_over requires per_match".to_string(),
        });
    }

    // FR-033-AC-5: validate every locator's `assert` facet at load time.
    for (key, loc) in dsl.yield_pattern.r#match.iter().flatten() {
        validate_locator_asserts(archetype, key, loc)?;
    }
    for (key, loc) in dsl.yield_pattern.per_match.iter().flatten() {
        validate_locator_asserts(archetype, key, loc)?;
    }
    Ok(())
}

/// Validate every primitive's `assert` facet inside a [`Locator`]
/// (primitive or fallback chain) at load time (FR-033-AC-5).
fn validate_locator_asserts(
    archetype: &str,
    key: &str,
    locator: &Locator,
) -> Result<(), QuireError> {
    match locator {
        Locator::Primitive(p) => validate_primitive_assert(archetype, key, p),
        Locator::Fallback(chain) => {
            for p in chain {
                validate_primitive_assert(archetype, key, p)?;
            }
            Ok(())
        }
    }
}

fn validate_primitive_assert(
    archetype: &str,
    key: &str,
    p: &LocatorPrimitive,
) -> Result<(), QuireError> {
    let Some(assert) = p.assert() else {
        return Ok(());
    };
    validate_assert_for_kind(archetype, key, p.kind(), assert)
}

/// Reject `assert` keys that are nonsensical for the locator kind
/// (FR-033-AC-5), e.g. `columns` on a `section_body` locator. Surfaces
/// as a `DslValidationError` naming the archetype + locator key so the
/// loader reports it as an `ArchetypeLoadFailure` at load time, not at
/// validate time.
pub fn validate_assert_for_kind(
    archetype: &str,
    key: &str,
    kind: LocatorKind,
    assert: &LocatorAssert,
) -> Result<(), QuireError> {
    let reject = |field: &str| -> Result<(), QuireError> {
        Err(QuireError::DslValidationError {
            archetype: archetype.to_string(),
            reason: format!(
                "assert key `{field}` is not valid on a `{}` locator (key '{key}')",
                kind.as_str()
            ),
        })
    };

    // `level` is only meaningful for headings / section bodies.
    if assert.level.is_some() && !matches!(kind, LocatorKind::SectionBody | LocatorKind::Heading) {
        return reject("level");
    }
    // Table-only keys.
    let table_only = matches!(kind, LocatorKind::TableRow);
    // CR-023: checked before `columns` so a doubly-illegal assert names the
    // more specific key. `optional_columns` is table-only, and every name it
    // lists MUST appear in `columns` — an optional column that is not a
    // declared column would silently do nothing, which is worse than a reject.
    if let Some(opt) = &assert.optional_columns {
        if !table_only {
            return reject("optional_columns");
        }
        let declared = assert.columns.as_deref().unwrap_or(&[]);
        if opt.iter().any(|o| !declared.iter().any(|c| c == o)) {
            return reject("optional_columns");
        }
    }
    if assert.columns.is_some() && !table_only {
        return reject("columns");
    }
    if assert.min_rows.is_some() && !table_only {
        return reject("min_rows");
    }
    if assert.id_column.is_some() && !table_only {
        return reject("id_column");
    }
    // List-only key.
    if assert.min_items.is_some() && !matches!(kind, LocatorKind::ListItem) {
        return reject("min_items");
    }
    // `id_pattern` checks the located scalar value (FR-034). Per
    // FR-033-AC-7 it is legal on `table_row`, `heading`, `section_body`,
    // `list_item`, and `frontmatter_field` (scalar) — but NOT on
    // `code_block`, whose value is a fenced source blob, not an id token.
    if assert.id_pattern.is_some() && matches!(kind, LocatorKind::CodeBlock) {
        return reject("id_pattern");
    }
    // `matches` asserts the located content against a regex (FR-033). It is
    // legal on every content locator EXCEPT `table_row`, whose structure is
    // checked via `columns`/`min_rows`/`id_pattern`, not a flat content match.
    if assert.matches.is_some() && matches!(kind, LocatorKind::TableRow) {
        return reject("matches");
    }
    // `choices` is an enum constraint on a located scalar value (CR-010). Like
    // scalar `id_pattern`/`matches` it applies to a scalar locator — legal on
    // `heading`/`section_body`/`list_item`/`frontmatter_field`, illegal on
    // `table_row` (use `column_choices`) and `code_block`.
    if assert.choices.is_some() && matches!(kind, LocatorKind::TableRow | LocatorKind::CodeBlock) {
        return reject("choices");
    }
    // Per-column table value asserts (CR-010) — `table_row` only.
    if assert.column_choices.is_some() && !table_only {
        return reject("column_choices");
    }
    if assert.column_patterns.is_some() && !table_only {
        return reject("column_patterns");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> ExtractionDsl {
        serde_yaml::from_str(yaml).expect("parse")
    }

    #[test]
    fn match_pattern_round_trips_through_yaml() {
        let dsl = parse(
            r#"
yield_pattern:
  match:
    id:
      from: frontmatter_field
      path: [id]
      required: true
    purpose:
      from: section_body
      after_heading: Purpose
"#,
        );
        validate_dsl("ot", &dsl).expect("ok");
        let m = dsl.yield_pattern.r#match.as_ref().unwrap();
        assert!(m.contains_key("id"));
        assert!(m.contains_key("purpose"));
    }

    #[test]
    fn iterate_over_pattern_validates() {
        let dsl = parse(
            r#"
yield_pattern:
  iterate_over:
    section_path: [Algorithms]
    kind: heading
    depth: 1
  per_match:
    name:
      from: heading
"#,
        );
        validate_dsl("ot", &dsl).expect("ok");
    }

    // FR-011-AC-6
    #[test]
    fn both_match_and_iterate_over_is_dsl_error() {
        let dsl = parse(
            r#"
yield_pattern:
  match:
    a:
      from: heading
  iterate_over:
    section_path: [x]
    kind: heading
  per_match:
    b:
      from: heading
"#,
        );
        let err = validate_dsl("ot", &dsl).expect_err("collision");
        assert!(matches!(err, QuireError::DslValidationError { .. }));
    }

    #[test]
    fn neither_match_nor_iterate_over_is_dsl_error() {
        let dsl: ExtractionDsl = serde_yaml::from_str("yield_pattern: {}\n").unwrap();
        let err = validate_dsl("ot", &dsl).expect_err("empty");
        assert!(matches!(err, QuireError::DslValidationError { .. }));
    }

    // FR-011-AC-7
    #[test]
    fn unknown_key_in_yield_pattern_fails_yaml_parse() {
        let r: Result<ExtractionDsl, _> = serde_yaml::from_str(
            r#"
yield_pattern:
  unknown_key: x
"#,
        );
        assert!(r.is_err(), "deny_unknown_fields should reject unknown keys");
    }

    // TC-538 (FR-033-AC-5): an unknown assert key fails YAML parse
    // (deny_unknown_fields on LocatorAssert).
    #[test]
    fn unknown_assert_key_fails_yaml_parse() {
        let r: Result<ExtractionDsl, _> = serde_yaml::from_str(
            r#"
yield_pattern:
  match:
    s:
      from: section_body
      after_heading: Purpose
      assert:
        bogus_key: 1
"#,
        );
        assert!(
            r.is_err(),
            "deny_unknown_fields should reject unknown assert keys"
        );
    }

    // TC-538 (FR-033-AC-5): `columns` on a `section_body` locator is a
    // load-time DslValidationError naming the locator.
    #[test]
    fn columns_on_section_body_is_load_error() {
        let dsl = parse(
            r#"
yield_pattern:
  match:
    s:
      from: section_body
      after_heading: Purpose
      assert:
        columns: [ID, Criteria]
"#,
        );
        let err = validate_dsl("FR", &dsl).expect_err("nonsensical assert");
        match err {
            QuireError::DslValidationError { archetype, reason } => {
                assert_eq!(archetype, "FR");
                assert!(reason.contains("columns"), "{reason}");
                assert!(reason.contains("section_body"), "{reason}");
                assert!(reason.contains("'s'"), "{reason}");
            }
            other => panic!("expected DslValidationError, got {other:?}"),
        }
    }

    // TC-538: `min_items` on a `table_row` locator is rejected.
    #[test]
    fn min_items_on_table_row_is_load_error() {
        let dsl = parse(
            r#"
yield_pattern:
  match:
    t:
      from: table_row
      under_section: AC
      assert:
        min_items: 1
"#,
        );
        let err = validate_dsl("FR", &dsl).expect_err("nonsensical assert");
        assert!(matches!(err, QuireError::DslValidationError { .. }));
    }

    // A well-formed assert passes load-time validation.
    #[test]
    fn valid_table_assert_passes_load() {
        let dsl = parse(
            r#"
yield_pattern:
  match:
    t:
      from: table_row
      under_section: AC
      assert:
        columns: [ID, Criteria]
        min_rows: 1
        id_column: ID
        id_pattern: '^AC-\d+$'
"#,
        );
        validate_dsl("FR", &dsl).expect("valid assert");
    }

    // TC-570 (FR-033-AC-7): the assert-key × locator-kind legality matrix.
    // For every (kind, key) cell: legal cells pass `validate_assert_for_kind`,
    // illegal cells produce an `ArchetypeLoadFailure` (DslValidationError)
    // naming the archetype + offending key. Table-driven across every cell.
    #[test]
    fn tc570_assert_key_locator_kind_legality_matrix() {
        use LocatorKind::*;

        // One LocatorAssert carrying exactly the named key.
        fn assert_with(key: &str) -> LocatorAssert {
            let mut a = LocatorAssert::default();
            match key {
                "level" => a.level = Some(2),
                "columns" => a.columns = Some(vec!["ID".to_string()]),
                "min_rows" => a.min_rows = Some(1),
                "min_items" => a.min_items = Some(1),
                "id_column" => a.id_column = Some("ID".to_string()),
                "id_pattern" => a.id_pattern = Some("^X-".to_string()),
                "matches" => a.matches = Some("^x".to_string()),
                "choices" => a.choices = Some(vec!["x".to_string()]),
                "column_choices" => {
                    a.column_choices = Some(
                        [("ID".to_string(), vec!["x".to_string()])]
                            .into_iter()
                            .collect(),
                    )
                }
                "column_patterns" => {
                    a.column_patterns = Some(
                        [("ID".to_string(), "^X-".to_string())]
                            .into_iter()
                            .collect(),
                    )
                }
                // CR-023: `optional_columns` only means anything alongside
                // `columns`, and load-time validation rejects it otherwise, so
                // the matrix case carries both.
                "optional_columns" => {
                    a.columns = Some(vec!["ID".to_string(), "Priority".to_string()]);
                    a.optional_columns = Some(vec!["Priority".to_string()]);
                }
                other => panic!("unknown key {other}"),
            }
            a
        }

        let kinds = [
            FrontmatterField,
            SectionBody,
            CodeBlock,
            TableRow,
            ListItem,
            Heading,
        ];
        let keys = [
            "level",
            "columns",
            "min_rows",
            "min_items",
            "id_column",
            "id_pattern",
            "matches",
            "choices",
            "column_choices",
            "column_patterns",
            "optional_columns",
        ];

        // Legality per FR-033-AC-7.
        fn legal(kind: LocatorKind, key: &str) -> bool {
            match key {
                "level" => matches!(kind, SectionBody | Heading),
                "columns" | "min_rows" | "id_column" => matches!(kind, TableRow),
                "min_items" => matches!(kind, ListItem),
                "id_pattern" => matches!(
                    kind,
                    TableRow | Heading | SectionBody | ListItem | FrontmatterField
                ),
                "matches" => !matches!(kind, TableRow),
                // CR-010: `choices` is a scalar enum (not table/code_block);
                // `column_*` are table-only.
                "choices" => {
                    matches!(kind, Heading | SectionBody | ListItem | FrontmatterField)
                }
                "column_choices" | "column_patterns" => matches!(kind, TableRow),
                // CR-023: declarable-optional columns are table-only.
                "optional_columns" => matches!(kind, TableRow),
                _ => unreachable!(),
            }
        }

        for &kind in &kinds {
            for &key in &keys {
                let assert = assert_with(key);
                let result = validate_assert_for_kind("FR", "k", kind, &assert);
                if legal(kind, key) {
                    assert!(
                        result.is_ok(),
                        "cell ({}, {key}) should be legal, got {result:?}",
                        kind.as_str()
                    );
                } else {
                    match result {
                        Err(QuireError::DslValidationError { archetype, reason }) => {
                            assert_eq!(archetype, "FR");
                            assert!(
                                reason.contains(key) && reason.contains(kind.as_str()),
                                "cell ({}, {key}) reason should name key+kind: {reason}",
                                kind.as_str()
                            );
                        }
                        other => panic!(
                            "cell ({}, {key}) should be illegal, got {other:?}",
                            kind.as_str()
                        ),
                    }
                }
            }
        }
    }
}
