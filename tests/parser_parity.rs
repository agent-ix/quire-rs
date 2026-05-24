//! Parser-parity test binary.
//!
//! Each module below ports one upstream test suite from the TS reference
//! (`~/dev/quire/tests/core/`) or the Python reference
//! (`~/dev/quire-py/tests/`). Intentional divergences vs. the references
//! are listed in `tests/parser_parity/divergences.md` and reflected in
//! the corresponding test assertions.
//!
//! Cargo only auto-discovers `tests/*.rs` as integration binaries; the
//! per-suite files live in `tests/parser_parity/` and are mounted as
//! submodules here.

mod parser_parity {
    pub mod ast_py;
    pub mod frontmatter_py;
    pub mod frontmatter_ts;
    pub mod parser_py;
    pub mod parser_ts;
    pub mod query_ts;
}
