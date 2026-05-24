//! Relationship harvesting + resolution (FR-015, Tasks 017 + 023).
//!
//! `harvest_edges` reads the three edge sources documented in FR-015
//! (structured `relationships:` block, sugar fields, DSL `emit_edges`),
//! normalizes each target via a [`RelationshipResolver`], and dedups
//! the result.

pub mod harvest;
pub mod resolver;

pub use harvest::{harvest_edges, EdgeHarvest, SUGAR_FIELDS};
pub use resolver::{IdentityResolver, MockResolver, RelationshipResolver};
