//! Parser primitives (Task 001).
//!
//! Four pure, dependency-free primitives that Task 002's `parse_document`
//! orchestrates:
//!
//! - [`frontmatter::extract_frontmatter`] (FR-006)
//! - [`walk::walk_headings`] (FR-007)
//! - [`slice::slice_section_content`] (FR-008)
//! - [`slug::slug`] / [`slug::slug_line_id`] (FR-009)

pub mod frontmatter;
pub mod slice;
pub mod slug;
pub mod walk;

pub use frontmatter::{extract_frontmatter, FrontmatterResult};
pub use slice::{line_offsets, slice_section_content};
pub use slug::{slug, slug_line_id};
pub use walk::{walk_headings, Heading};
