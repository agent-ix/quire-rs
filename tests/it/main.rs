//! The root crate's integration tests, linked as one test binary.
//!
//! Cargo builds every file directly under `tests/` as its own executable and
//! links the whole library into each. These suites are modules of one target
//! instead (PLAT-940). `concurrency` (`cfg(loom)`) and `corpus_concurrency`
//! (sanitizer lane) stay separate `[[test]]` targets; see `Cargo.toml`.

mod assurance_boundary;
mod assurance_export;
mod audit_table;
mod bundle_validate;
mod clause_publishable;
mod combinatorial;
mod corpus_case;
mod corpus_cases;
mod corpus_recall;
mod coverage_architecture;
mod coverage_baseline;
mod coverage_matrix;
mod coverage_rollup;
mod cross_repo;
mod determinism;
mod filament_boundary_audit;
mod filament_core_fixtures;
mod id_column_lists;
#[cfg(feature = "rust-symbols")]
mod implements_relation;
mod implements_structural;
mod integration;
mod inverse_edges;
#[cfg(feature = "rust-symbols")]
mod legacy_implements_warning;
mod located_findings;
mod markdown_validation;
mod no_hidden_recompile;
mod obligations;
mod output_contract;
#[cfg(feature = "rust-symbols")]
mod outside_marker_advisory;
mod parser_golden;
mod parser_parity;
mod parser_real_documents;
mod plain_language;
mod plat_validation_batch;
mod props_fr_040;
mod props_metamorphic;
mod quality_lints;
mod relation_report;
mod required_relations;
mod required_relations_where;
mod robustness;
mod semantic_boundary;
mod semantic_clauses;
mod semantic_contract;
mod semantic_embedded_schema;
mod semantic_features;
mod semantic_fixtures;
mod semantic_model;
mod semantic_properties;
mod semantic_record_sections;
mod semantic_relations;
mod semantic_surface;
mod semantic_systems;
#[cfg(feature = "shell-symbols")]
mod shell_symbols;
mod spec_dogfood;
mod trace_dogfood;
mod trace_reference_kinds;
mod trace_references;
mod trace_target_sources;
mod verification_catalog;
mod vocabulary_coverage;
mod vocabulary_refs;
