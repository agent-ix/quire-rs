//! Rust source adapter (FR-051).
//!
//! Parses over a tree-sitter syntax tree, through the dependency-boundary
//! crate `quire-rust-extraction` (PLAT-843) — see that crate's own module
//! docs for why the tree-sitter dependency itself is never named here. This
//! adapter still answers exactly the questions FR-051 needs — which
//! declaration owns a span, what its qualified name and kind are, whether it
//! is a test — and nothing more: no build, no type resolution, no dependency
//! installation, and no execution of the extracted code (FR-051-CON-1).
//!
//! **Form matching is a seam this adapter does not cross.** Which declared
//! marker or legacy trace form appears inside a symbol's span is
//! `trace.rs`'s question, answered afterwards against the resolved span
//! (`Symbol::attached_source`); this file has no reference to trace forms,
//! regex, or `TraceabilityModel`, and the tree walk below must not gain one.
//!
//! Test classification is the `#[test]` **family**: any attribute in the
//! declaration's leading annotation block whose path ends in `test` —
//! `#[test]`, `#[tokio::test]`, `#[rstest]`, `#[wasm_bindgen_test]` — read
//! from attribute nodes rather than text.
//!
//! Two other verification methods produce leaf symbols and are classified as
//! such, because a matrix row verified by them can otherwise never be backed
//! however it is tagged (CR-061):
//!
//! - **Benchmarks.** A criterion bench carries no attribute — it is an
//!   ordinary `fn` *registered* by `criterion_group!(name, a, b)` or
//!   `criterion_group!(name = g; config = c(); targets = a, b)`, in either
//!   invocation form and across a multi-line one. So the registrations are
//!   collected in the same walk and the named functions are promoted
//!   afterwards. `#[bench]` is recognised directly.
//! - **Fuzz targets.** `fuzz_target!(|data: &[u8]| { … })` declares no `fn`
//!   at all, so nothing was extracted from those files before CR-061. One
//!   symbol is minted for the invocation, spanning the whole file — see
//!   [`fuzz_target`].
//!
//! ## What survives byte-for-byte from the line-structural scanner
//!
//! `Symbol::compute_id` hashes `(language, path, qualified_name, kind)`
//! (`src/symbols/mod.rs`) — qualified-name construction is an identity
//! surface, not a style choice, and every quirk below is preserved exactly
//! rather than "cleaned up" (PLAT-843):
//!
//! - **`impl` blocks are scopes, not symbols.** `impl Display for Foo { fn
//!   fmt(&self) {} }` mints `Container "Foo"` (from the `struct`) and
//!   `Function "Foo::fmt"`; the `impl` block itself mints nothing, and a
//!   method qualifies under the **target type after `for`**, never the
//!   trait — read from `impl_item`'s own `type` field, never `trait`.
//! - **`type`, `static`, `const` and `union` mint no symbols.** Only
//!   `fn`/`mod`/`struct`/`enum`/`trait`/`impl` are recognised, the same as
//!   before; simply not matching their tree-sitter node kinds is what keeps
//!   this true — no denylist needed.
//! - **A `fn` is never a container** (the duplicate-identity flattening,
//!   PLAT-845's to fix, not this ticket's): two same-named helpers nested in
//!   two different top-level `fn`s both mint `qualified_name="helper",
//!   container=None`. [`walk`] recurses into a function's own body with the
//!   *same* container it was called with, never the function's own qualified
//!   name, to keep this exact.
//! - **Brace-less declarations** (a unit struct, a trait method signature)
//!   end at the tree-sitter node's own last line — which is exactly "the
//!   first line ending in `;`, or file length" for the shapes this adapter
//!   recognises, with no separate brace-depth walk needed.
//! - **At most one `FuzzTarget` per file**, guarded explicitly (a second
//!   would mint a duplicate identity, FR-051-AC-2); `qualified_name` is the
//!   literal `"fuzz_target"` and `leading_line` is hardcoded to `1`, because
//!   a `#![no_main]` file's `//!` module header is where the tag is written
//!   and no sibling-walk reaches across the intervening `use` statements.
//!
//! ## What changed, and why each delta is free (PLAT-843)
//!
//! Per the ticket's cause-based rule — *free from parsing correctly → fix
//! here and name the delta; requires changing what a symbol **is** or how it
//! is **named** → a separate ticket* — three deltas are taken here, all free:
//!
//! - **`check_balanced` whole-file rejection is gone.** tree-sitter parses
//!   per node and recovers locally; there is no longer a single depth
//!   counter for one unresolved brace anywhere to desync and zero the whole
//!   file. A genuinely broken declaration still fails loudly — see
//!   [`parse`]'s own docs — but a brace living in a raw string, a lifetime,
//!   a char literal, or a nested block comment was never a real desync, and
//!   tree-sitter never treats it as one.
//! - **Leading spans no longer truncate at a rustfmt-split attribute
//!   continuation line** (PLAT-69). [`leading_span`] walks preceding
//!   *sibling nodes*, not lines: a multi-line `#[cfg_attr(\n  ...\n)]` is one
//!   `attribute_item` node regardless of how it wraps, so the span reaches
//!   its true start.
//! - **A block doc comment (`/** ... */`) joins the leading span**
//!   (PLAT-846, folded into this ticket). tree-sitter represents `//`,
//!   `///`, `/* */` and `/** */` as sibling `line_comment`/`block_comment`
//!   nodes with no distinction this adapter needs to make between them, so
//!   [`leading_span`] includes any of them rather than special-casing doc
//!   comments back out — the same defect class as PLAT-69, same file, same
//!   fix shape.
//!
//! What did **not** change: the hand-written lexer subsystem this file used
//! to carry (`LexedLine`, `ScanState`, `lex`, `lex_line`,
//! `opens_char_literal`, `opens_raw_string`, `raw_prefix_len`, `closes_raw`,
//! `check_balanced`) is deleted outright as a consequence of parsing
//! correctly, not as a deliberate optimisation — no benchmark exists for
//! that site alone.

use quire_rust_extraction::tree_sitter::Node;
use quire_rust_extraction::{parse_file, Language};

use super::{RawSymbol, SymbolKind};

/// Parse `source` into raw symbols, or return a per-file reason to skip it.
///
/// **A declaration-structure failure fails loudly, naming the line.**
/// `quire-code-parse`'s own diagnostic never gates the tree it attaches to —
/// see that crate's docs — but this adapter's own contract
/// (`Result<Vec<RawSymbol>, String>`, unchanged by this port) is all-or-
/// nothing per file: `src/symbols/mod.rs`'s `extend_with_file` pushes one
/// [`super::SymbolDiagnostic`] on `Err` and contributes zero symbols for that
/// file, the same as before this port (FR-051-AC-9, FR-051-CON-2 — one
/// unparseable file never aborts the rest of the tree). What changed is the
/// *diagnostic itself*: previously a generic "unbalanced braces" reason with
/// no location; now the tree-sitter diagnostic's own one-based line and
/// zero-based column, naming exactly where the declaration structure could
/// not be trusted.
///
/// A `Diagnostic::None` and no error is by far the common case: tree-sitter
/// recovers locally from a body-local error (an incomplete expression,
/// mid-edit) without losing the declaration's own name, so a file with a
/// broken function *body* still extracts every symbol in it, including that
/// function itself — the false-positive whole-file rejection this port
/// exists to end.
pub(crate) fn parse(source: &str) -> Result<Vec<RawSymbol>, String> {
    // The file identifier `quire-code-parse` attributes a diagnostic to is
    // unused here: `mod.rs`'s `SymbolDiagnostic.path` already carries the
    // real repo-relative path (this adapter's `parse(source: &str)` contract
    // predates and is unchanged by this port), so naming the file a second
    // time inside the reason string would be redundant rather than useful.
    let parsed =
        parse_file(Language::Rust, "<source>", source).map_err(|e| format!("parse: {e}"))?;
    if let Some(diagnostic) = parsed.diagnostic() {
        return Err(format!(
            "line {}: unresolvable declaration structure (column {})",
            diagnostic.line(),
            diagnostic.column()
        ));
    }

    let mut out: Vec<RawSymbol> = Vec::new();
    // Functions a `criterion_group!` registers as benchmarks. Collected
    // during the walk because the registration follows the functions it
    // names (CR-061).
    let mut registered_benches: Vec<String> = Vec::new();

    walk(
        parsed.root_node(),
        source,
        None,
        &mut out,
        &mut registered_benches,
    );

    // At most one per file: a second `fuzz_target!` would mint a second
    // symbol with the identical `(language, path, qualified_name, kind)`
    // identity, and two symbols sharing an id is malformed however unlikely
    // libfuzzer makes it in practice (FR-051-AC-2).
    if let Some(target) = fuzz_target(parsed.root_node(), source) {
        out.push(target);
    }

    // A registered bench is promoted after the whole file is walked, because
    // `criterion_group!` follows the functions it names. Only a top-level,
    // untested function qualifies: a name repeated inside a `mod` is a
    // different symbol, and `#[test] fn` stays a test.
    for symbol in out.iter_mut() {
        if symbol.kind == SymbolKind::Function
            && symbol.container.is_none()
            && registered_benches.contains(&symbol.qualified_name)
        {
            symbol.kind = SymbolKind::Benchmark;
        }
    }

    Ok(out)
}

/// Walk `node`'s named children, minting a [`RawSymbol`] for each
/// declaration and recursing to find every nested one — `fn`/`mod`/`impl`
/// bodies, and anywhere else a Rust item can legally appear (an `if`, a
/// `match` arm, a block), the same set the old line-structural scanner saw
/// regardless of statement context.
///
/// `container` is the qualified name of the innermost *container* scope —
/// never a function's own name, which is the byte-for-byte-preserved
/// flattening quirk (a `fn` is never a container; see this module's own
/// docs) — and `criterion_group!` registrations are collected into
/// `registered_benches` wherever they occur.
fn walk(
    node: Node,
    source: &str,
    container: Option<String>,
    out: &mut Vec<RawSymbol>,
    registered_benches: &mut Vec<String>,
) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "function_item" | "function_signature_item" => {
                if let Some(symbol) = function_symbol(child, source, container.clone()) {
                    out.push(symbol);
                }
                // A `fn` is never a container (PLAT-845's flattening bug,
                // preserved deliberately): recurse into its own body with
                // the *same* container, not the function's own name.
                walk(child, source, container.clone(), out, registered_benches);
            }
            "mod_item" => {
                let name = field_text(child, "name", source);
                if let Some(name) = name {
                    let qualified = qualify(&container, &name);
                    out.push(container_symbol(
                        child,
                        qualified.clone(),
                        container.clone(),
                    ));
                    walk(child, source, Some(qualified), out, registered_benches);
                } else {
                    walk(child, source, container.clone(), out, registered_benches);
                }
            }
            "struct_item" | "enum_item" => {
                if let Some(name) = field_text(child, "name", source) {
                    let qualified = qualify(&container, &name);
                    out.push(container_symbol(child, qualified, container.clone()));
                }
                // A struct/enum body is a field or variant list, never a
                // declaration list: nothing recognised by this adapter can
                // nest inside one, so there is nothing to recurse into.
            }
            "trait_item" => {
                let name = field_text(child, "name", source);
                if let Some(name) = name {
                    let qualified = qualify(&container, &name);
                    out.push(container_symbol(
                        child,
                        qualified.clone(),
                        container.clone(),
                    ));
                    walk(child, source, Some(qualified), out, registered_benches);
                } else {
                    walk(child, source, container.clone(), out, registered_benches);
                }
            }
            "impl_item" => {
                // The `impl` block itself mints nothing; its methods qualify
                // under the target type named after `for` (or the plain
                // type, for an inherent impl) — never the trait, which is
                // why this reads the `type` field and not `trait`.
                let target = field_text(child, "type", source).and_then(|text| ident(&text));
                let inner_container = match target {
                    Some(name) => Some(qualify(&container, &name)),
                    // An unresolvable target (the old adapter's own
                    // limitation: a target type not starting with an
                    // identifier character, e.g. a reference or tuple type)
                    // makes the impl transparent, exactly as before — no new
                    // scope, methods inherit whatever container was already
                    // in effect.
                    None => container.clone(),
                };
                walk(child, source, inner_container, out, registered_benches);
            }
            "macro_invocation" => {
                collect_criterion_registrations(child, source, registered_benches);
                // `proptest! { ... }` declares tests through its own custom
                // argument syntax (`name(pattern in strategy) { body }`),
                // which tree-sitter tokenizes but never parses as
                // `function_item` nodes — so without this, a test declared
                // this way (and load-bearing trace tags on it) would
                // silently stop being extracted (see this module's own
                // docs). The macro invocation itself is never a container
                // (the pre-PLAT-843 adapter's line scanner never pushed a
                // scope for it either — its declarations qualify under
                // whatever scope already contained the invocation).
                collect_proptest_tests(child, source, &container, out);
                walk(child, source, container.clone(), out, registered_benches);
            }
            _ => {
                walk(child, source, container.clone(), out, registered_benches);
            }
        }
    }
}

/// `container::name`, or `name` alone when `container` is `None`.
fn qualify(container: &Option<String>, name: &str) -> String {
    match container {
        Some(prefix) => format!("{prefix}::{name}"),
        None => name.to_string(),
    }
}

/// A `RawSymbol` for a `mod`/`struct`/`enum`/`trait` declaration. Its span is
/// the node's own: `leading_span` for the leading annotation block, and the
/// node's last line for `end_line` — correct for both a brace body and a
/// brace-less one (a unit struct, `;`-terminated), since tree-sitter's own
/// node span already includes whichever terminator the grammar recognised.
///
/// `container` is the qualified name of the *enclosing* scope this
/// declaration was found in (not its own — a top-level declaration gets
/// `None`), matching `RawSymbol::container`'s own meaning everywhere else in
/// this adapter.
fn container_symbol(node: Node, qualified_name: String, container: Option<String>) -> RawSymbol {
    RawSymbol {
        qualified_name,
        kind: SymbolKind::Container,
        line: node.start_position().row + 1,
        leading_line: leading_span(node),
        end_line: node.end_position().row + 1,
        container,
    }
}

/// A `RawSymbol` for a `function_item`/`function_signature_item`, classified
/// as a test, a bench, or a plain function. `None` only when the node has no
/// resolvable `name` field, which the grammar guarantees it always does for
/// these two kinds — kept as a guard rather than a panic, matching this
/// crate's no-panic-in-production-code convention.
fn function_symbol(node: Node, source: &str, container: Option<String>) -> Option<RawSymbol> {
    let name = field_text(node, "name", source)?;
    let qualified_name = qualify(&container, &name);
    let kind = if has_attribute(node, source, "test") {
        SymbolKind::TestFunction
    } else if has_attribute(node, source, "bench") {
        SymbolKind::Benchmark
    } else {
        SymbolKind::Function
    };
    Some(RawSymbol {
        qualified_name,
        kind,
        line: node.start_position().row + 1,
        leading_line: leading_span(node),
        end_line: node.end_position().row + 1,
        container,
    })
}

/// `node`'s field named `name`, `type` &c. as plain source text, or `None`
/// when the field is absent.
fn field_text(node: Node, field: &str, source: &str) -> Option<String> {
    node.child_by_field_name(field)
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .map(str::to_string)
}

/// The leading identifier of `s` — the same truncation the pre-PLAT-843
/// adapter applied to an `impl` target's text, preserved exactly so the same
/// set of `impl` forms resolve a container (a plain or generic type name)
/// and the same set stay unresolved (a reference, a tuple, anything not
/// starting with an identifier character).
fn ident(s: &str) -> Option<String> {
    let name: String = s
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// The 1-based first line of `node`'s leading annotation block: the
/// contiguous run of preceding `attribute_item`/`inner_attribute_item`/
/// `line_comment`/`block_comment` sibling nodes, stopping at the first
/// sibling of another kind or the first blank-line gap — the same two
/// stopping conditions the line-structural scanner used, now checked between
/// sibling nodes instead of between lines, which is what fixes PLAT-69 and
/// PLAT-846: a multi-line attribute or a block doc comment is one sibling
/// node regardless of how many lines it spans.
fn leading_span(node: Node) -> usize {
    let mut boundary_row = node.start_position().row;
    let mut current = node;
    while let Some(prev) = current.prev_sibling() {
        if !is_annotation_node(prev) {
            break;
        }
        // A blank line between this annotation and the block already
        // collected breaks the run, exactly as an empty line did for the
        // line-structural scanner.
        if boundary_row.saturating_sub(prev.end_position().row) > 1 {
            break;
        }
        boundary_row = prev.start_position().row;
        current = prev;
    }
    boundary_row + 1
}

/// Whether `node` is a sibling kind this adapter treats as part of a leading
/// annotation block: an attribute (outer or inner) or a comment — `//`,
/// `///`, `/* */` or `/** */` alike. tree-sitter represents all four comment
/// forms with the same two node kinds (`line_comment`, `block_comment`),
/// distinguished only by an optional doc-marker field this adapter does not
/// need to inspect, so a plain block comment joins the span exactly as a
/// block *doc* comment does (PLAT-846) — there is no ticket-stated reason to
/// special-case the non-doc form back out, and doing so would be more code
/// for a distinction this adapter never otherwise makes.
fn is_annotation_node(node: Node) -> bool {
    matches!(
        node.kind(),
        "attribute_item" | "inner_attribute_item" | "line_comment" | "block_comment"
    )
}

/// True when `node`'s leading annotation block holds an attribute whose
/// final path segment is `segment` — `#[test]`, `#[tokio::test]`,
/// `#[rstest]`. Only the *outer* attribute path is read, never a nested
/// argument (`#[cfg_attr(feature = "x", test)]` does not count, matching the
/// pre-PLAT-843 adapter exactly): the same reading that already makes
/// PLAT-305 pass (`#[ignore]` before `#[test]` — every attribute in the
/// block is checked, not only the one immediately above the declaration).
fn has_attribute(node: Node, source: &str, segment: &str) -> bool {
    let mut current = node;
    while let Some(prev) = current.prev_sibling() {
        if !is_annotation_node(prev) {
            break;
        }
        let boundary_row = current.start_position().row;
        if boundary_row.saturating_sub(prev.end_position().row) > 1 {
            break;
        }
        if prev.kind() == "attribute_item" {
            if let Some(path) = prev
                .child_by_field_name("attribute")
                .or_else(|| prev.named_child(0))
                .and_then(|attribute| attribute.named_child(0))
                .and_then(|path_node| path_node.utf8_text(source.as_bytes()).ok())
            {
                if path.rsplit("::").next() == Some(segment) {
                    return true;
                }
            }
        }
        current = prev;
    }
    false
}

/// The names `criterion_group!(name, a, b)` or
/// `criterion_group!(name = g; config = c(); targets = a, b)` registers as
/// benchmarks, appended to `registered_benches` — `None` when `node` is not
/// such an invocation.
///
/// Only `criterion_group!` and `criterion::criterion_group!` are recognised,
/// matching the pre-PLAT-843 adapter's own two literal prefixes exactly
/// (never a generically-scoped re-export).
fn collect_criterion_registrations(node: Node, source: &str, registered_benches: &mut Vec<String>) {
    let Some(invocation) = macro_invocation(node) else {
        return;
    };
    let Some(macro_path) = invocation
        .child_by_field_name("macro")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
    else {
        return;
    };
    if macro_path != "criterion_group" && macro_path != "criterion::criterion_group" {
        return;
    }
    let Some(token_tree) = invocation
        .named_children(&mut invocation.walk())
        .find(|n| n.kind() == "token_tree")
    else {
        return;
    };
    registered_benches.extend(criterion_group_members(token_tree, source));
}

/// `node` itself if it is a `macro_invocation`, or the one it wraps if
/// `node` is the `expression_statement`/`item` around it — a `fuzz_target!`
/// or `criterion_group!` call is always a bare statement, so this only ever
/// needs to look one level down.
fn macro_invocation(node: Node) -> Option<Node> {
    if node.kind() == "macro_invocation" {
        return Some(node);
    }
    node.named_child(0)
        .filter(|n| n.kind() == "macro_invocation")
}

/// The identifiers registered inside a `criterion_group!` invocation's
/// `token_tree`, reading raw (named + anonymous) children so the literal
/// `=`/`;`/`,` punctuation tokens are visible as boundaries.
///
/// Two invocation forms: the plain list `(name, a, b)` — skip the first
/// identifier (the group's own name, not a benchmark) and collect the rest —
/// and the long `(name = g; config = c(); targets = a, b)` form, where a
/// `targets` identifier is matched as a whole token immediately followed by
/// `=` (never a group or bench merely *named* `targets_something`) and
/// everything after it, up to the next `;` or the end, is collected.
fn criterion_group_members(token_tree: Node, source: &str) -> Vec<String> {
    let children: Vec<Node> = token_tree.children(&mut token_tree.walk()).collect();
    let text = |n: &Node| n.utf8_text(source.as_bytes()).unwrap_or_default();

    if let Some(targets_at) = children
        .iter()
        .position(|n| n.kind() == "identifier" && text(n) == "targets")
    {
        if children.get(targets_at + 1).map(text) == Some("=") {
            return children[targets_at + 2..]
                .iter()
                .take_while(|n| n.kind() != ";")
                .filter(|n| n.kind() == "identifier")
                .map(|n| text(n).to_string())
                .collect();
        }
    }

    // Short form: skip the opening `(`, the group's own name, and its
    // trailing `,` — everything after is the member list.
    children
        .iter()
        .skip_while(|n| n.kind() != "identifier") // to the group name
        .skip(1) // past the group name itself
        .filter(|n| n.kind() == "identifier")
        .map(|n| text(n).to_string())
        .collect()
}

/// Mint a `RawSymbol` for every `#[test] fn NAME(pattern in strategy) {
/// body }` written inside `proptest! { ... }` / `proptest::proptest! { ...
/// }`, appended to `out` with `container`.
///
/// `proptest!`'s own argument syntax — `name(pattern in strategy) { body }`
/// — is not valid standalone Rust, so tree-sitter tokenizes a proptest body
/// but never resolves a `fn` written inside it as a `function_item` node.
/// The pre-PLAT-843 line-structural scanner had no such concept and matched
/// every `fn `-prefixed line regardless of context, so it read these
/// exactly like any other declaration — load-bearingly: quire-rs's own
/// `props_metamorphic` modules carry real `#[trace(...)]` tags on functions
/// declared this way. Losing them would be exactly the silent coverage
/// class this port exists to end, reintroduced by the one macro form the
/// grammar cannot see into. `criterion_group!`/`fuzz_target!` are handled
/// by reading the *registration*, because neither declares a `fn` inside an
/// opaque macro body; `proptest!` is the one case that does, so it alone
/// needs a token-level scan rather than a field read.
fn collect_proptest_tests(
    node: Node,
    source: &str,
    container: &Option<String>,
    out: &mut Vec<RawSymbol>,
) {
    let Some(invocation) = macro_invocation(node) else {
        return;
    };
    let Some(macro_path) = invocation
        .child_by_field_name("macro")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
    else {
        return;
    };
    if macro_path != "proptest" && macro_path != "proptest::proptest" {
        return;
    }
    if let Some(token_tree) = invocation
        .named_children(&mut invocation.walk())
        .find(|n| n.kind() == "token_tree")
    {
        scan_token_tree_for_fns(token_tree, source, container, out);
    }
}

/// Scan `token_tree`'s own flat (named + anonymous) children for a `fn
/// NAME(...) { ... }` token sequence, minting a symbol for each and
/// recursing into any other nested `token_tree` — including a matched
/// `fn`'s own body, so a helper `fn` declared inside a proptest test's body
/// is still found, matching the pre-PLAT-843 scanner's own fully
/// context-free line scan.
fn scan_token_tree_for_fns(
    token_tree: Node,
    source: &str,
    container: &Option<String>,
    out: &mut Vec<RawSymbol>,
) {
    let children: Vec<Node> = token_tree.children(&mut token_tree.walk()).collect();
    let mut i = 0;
    while i < children.len() {
        let child = children[i];
        if child.kind() == "fn" {
            let name = children
                .get(i + 1)
                .filter(|n| n.kind() == "identifier")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok());
            let params = children.get(i + 2).filter(|n| n.kind() == "token_tree");
            let body = children.get(i + 3).filter(|n| n.kind() == "token_tree");
            if let (Some(name), Some(_params), Some(body)) = (name, params, body) {
                let qualified_name = qualify(container, name);
                let (leading_line, is_test) = flat_leading_span_and_test(&children, i, source);
                out.push(RawSymbol {
                    qualified_name,
                    kind: if is_test {
                        SymbolKind::TestFunction
                    } else {
                        SymbolKind::Function
                    },
                    line: child.start_position().row + 1,
                    leading_line,
                    end_line: body.end_position().row + 1,
                    container: container.clone(),
                });
                scan_token_tree_for_fns(*body, source, container, out);
                i += 4;
                continue;
            }
        }
        if child.kind() == "token_tree" {
            scan_token_tree_for_fns(child, source, container, out);
        }
        i += 1;
    }
}

/// For the `fn` token at `children[fn_idx]`: the 1-based line of the start
/// of its leading annotation run, and whether that run holds a `#[test]`-
/// family attribute — the same two questions `leading_span`/`has_attribute`
/// answer for a real `function_item`, asked instead over `proptest!`'s flat
/// token sequence, where an attribute is `#` followed by a `[...]`
/// `token_tree` rather than a single `attribute_item` node.
fn flat_leading_span_and_test(children: &[Node], fn_idx: usize, source: &str) -> (usize, bool) {
    let mut boundary_row = children[fn_idx].start_position().row;
    let mut is_test = false;
    let mut j = fn_idx;
    loop {
        if j == 0 {
            break;
        }
        let prev = children[j - 1];
        match prev.kind() {
            "line_comment" | "block_comment" => {
                if boundary_row.saturating_sub(prev.end_position().row) > 1 {
                    break;
                }
                boundary_row = prev.start_position().row;
                j -= 1;
            }
            "token_tree" if j >= 2 && children[j - 2].kind() == "#" => {
                if boundary_row.saturating_sub(prev.end_position().row) > 1 {
                    break;
                }
                if let Ok(inner) = prev.utf8_text(source.as_bytes()) {
                    let path = inner
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .split(['(', ','])
                        .next()
                        .unwrap_or("")
                        .trim();
                    if path.rsplit("::").next() == Some("test") {
                        is_test = true;
                    }
                }
                boundary_row = children[j - 2].start_position().row;
                j -= 2;
            }
            _ => break,
        }
    }
    (boundary_row + 1, is_test)
}

/// One symbol for a `fuzz_target!` invocation at the top level of the file,
/// or `None`.
///
/// The macro declares no `fn`, so before CR-061 a fuzz-target file yielded no
/// symbol at all and its trace tag bound to nothing.
///
/// **Its span is the whole file from line 1.** A `#![no_main]` fuzz-target
/// file declares exactly one entry point, so its `//!` module header *is*
/// the invocation's annotation block — which is where the tag is naturally
/// written, and no sibling-walk reaches it across the intervening `use`
/// statements.
fn fuzz_target(root: Node, source: &str) -> Option<RawSymbol> {
    let mut cursor = root.walk();
    // Every top-level macro invocation, not just the first one: an earlier
    // unrelated invocation (`lazy_static!`, `include!`, ...) must not hide a
    // `fuzz_target!` that comes after it. `find_map`+`filter` on the first
    // match alone silently dropped the whole symbol in exactly that shape —
    // a regression this rewrite introduced, not one the old line scanner
    // (which read every line regardless of position) ever had.
    let invocation = root
        .named_children(&mut cursor)
        .filter_map(macro_invocation)
        .find(|invocation| {
            invocation
                .child_by_field_name("macro")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                == Some("fuzz_target")
        })?;
    // The statement wrapping the invocation (its `;`) is what the old
    // line-structural scanner's own `block_end` walked to — its span is the
    // invocation's own parent when that parent is the wrapping statement.
    let span_node = invocation.parent().unwrap_or(invocation);
    Some(RawSymbol {
        qualified_name: "fuzz_target".to_string(),
        kind: SymbolKind::FuzzTarget,
        line: invocation.start_position().row + 1,
        leading_line: 1,
        end_line: span_node.end_position().row + 1,
        container: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-804, FR-051-AC-15 (CR-040, PLAT-843): a file is not rejected for
    /// braces that live in a raw string, a lifetime, a char literal or a
    /// nested comment. This is the test the 33-file whole-file-rejection
    /// incident is pinned by (see this module's own docs); it must go on
    /// passing against the AST with no edit, because it asserts an outcome
    /// (extracted symbols and their spans), not the deleted lexer's own
    /// intermediate state.
    #[test]
    fn tc804_rust_lexing_is_string_and_lifetime_aware() {
        let source = concat!(
            "fn schema() -> &'static str {\n",
            "    r#\"{\"type\":\"object\",\"required\":[\"id\"]}\"#\n",
            "}\n",
            "\n",
            "fn borrow<'a>(s: &'a str) -> &'a str {\n",
            "    let brace = '{';\n",
            "    let _ = brace;\n",
            "    s\n",
            "}\n",
            "\n",
            "/* outer /* nested } */ still a comment { */\n",
            "#[test]\n",
            "fn tc804_covers_it() {\n",
            "    assert!(true);\n",
            "}\n",
        );

        let symbols = parse(source).expect("a valid file must not be rejected");
        let test_fn = symbols
            .iter()
            .find(|s| s.qualified_name == "tc804_covers_it")
            .expect("the test function is a symbol");
        assert_eq!(test_fn.kind, SymbolKind::TestFunction);
        // The span must stop at the function's own closing brace.
        assert_eq!(test_fn.line, 13);
        assert_eq!(test_fn.end_line, 15);
        // The other two declarations survive as ordinary functions.
        assert!(symbols.iter().any(|s| s.qualified_name == "schema"));
        assert!(symbols.iter().any(|s| s.qualified_name == "borrow"));
    }

    /// TC-804 successor (PLAT-843): the *property* the deleted lexer's own
    /// `tc804_lexer_counts_only_code_braces` pinned at the mechanism level —
    /// a brace, quote or comment delimiter inside a string literal, a raw
    /// string, a char literal, a doc comment or a nested block comment does
    /// not change which symbols are extracted, does not move any symbol's
    /// span, and does not cause the file to be rejected — asserted directly
    /// against `parse`'s outcome instead of against `lex_line`'s deltas,
    /// which no longer exist.
    ///
    /// Confirmed to fail first, against a deliberately broken stand-in that
    /// counts braces textually rather than parsing: a naive
    /// `source.matches('{').count() == source.matches('}').count()` check on
    /// each of these adversarial fixtures reports an *imbalance*, which is
    /// exactly the false rejection this property forbids — so this test is
    /// not satisfiable by an implementation that regresses to text scanning.
    #[test]
    fn tc804_delimiters_in_string_and_comment_content_do_not_move_symbols() {
        // Each fixture is `fn kept() { <one line holding the adversarial
        // delimiter> }`, so `end_line` is always the closing brace on the
        // fixture's own last line — asserted directly per case, rather than
        // against an unrelated baseline whose line count could differ.
        let adversarial = [
            // A brace inside a raw string, and an unbalanced one at that.
            "fn kept() {\n    let s = r#\"{ unmatched\"#;\n}\n",
            // A `//` inside a raw string spanning a continuation line.
            "fn kept() {\n    let s = r#\"a // not a comment\n{}\"#;\n}\n",
            // A nested block comment.
            "fn kept() {\n    /* outer /* } { */ still here */\n}\n",
            // A brace inside a char literal.
            "fn kept() {\n    let c = '{';\n}\n",
            // A brace inside a doc comment.
            "fn kept() {\n    /// a stray { in a doc comment\n}\n",
        ];
        for source in adversarial {
            let symbols = parse(source).expect("must not be rejected");
            assert_eq!(
                symbols.len(),
                1,
                "exactly one symbol, the function itself: {source:?} -> {symbols:?}"
            );
            let kept = &symbols[0];
            assert_eq!(kept.qualified_name, "kept");
            assert_eq!(kept.kind, SymbolKind::Function);
            assert_eq!(kept.line, 1);
            let expected_end = source.lines().count();
            assert_eq!(
                kept.end_line, expected_end,
                "the function's own closing brace, not a false positive from \
                 content inside the string/comment: {source:?}"
            );
        }
    }

    /// TC-804 successor (PLAT-843): the other half of the deleted
    /// `tc804_string_state_carries_across_lines` — a string or raw string
    /// left open on one line still shields a brace on the *next* line from
    /// being counted, asserted through `parse`'s outcome (a function
    /// spanning both lines extracts with the right `end_line`) rather than
    /// through `ScanState` fields that no longer exist.
    #[test]
    fn tc804_a_delimiter_carried_across_lines_does_not_move_symbols() {
        let source = concat!(
            "fn kept() -> &'static str {\n",
            "    r#\"\n",
            "    a url // not a comment, and a brace {\n",
            "    \"#\n",
            "}\n",
        );
        let symbols = parse(source).expect("must not be rejected");
        let f = symbols
            .iter()
            .find(|s| s.qualified_name == "kept")
            .expect("kept is a symbol");
        assert_eq!(f.line, 1);
        assert_eq!(f.end_line, 5);
    }

    /// TC-827, FR-051-AC-17 (CR-061): a criterion bench carries no
    /// attribute — it is an ordinary `fn` that `criterion_group!`
    /// registers — so the registration is what classifies it. Both
    /// invocation forms, wrapped or not, and `#[bench]` directly.
    #[test]
    fn tc827_criterion_registrations_classify_benchmarks() {
        let source = concat!(
            "fn bench_parse(c: &mut Criterion) {\n",
            "    let _ = c;\n",
            "}\n",
            "fn bench_validate(c: &mut Criterion) {\n",
            "    let _ = c;\n",
            "}\n",
            "fn helper() -> usize {\n",
            "    1\n",
            "}\n",
            "mod inner {\n",
            // Same name, nested: a different symbol, and not registered.
            "    fn bench_parse() {\n",
            "        let _ = 1;\n",
            "    }\n",
            "}\n",
            "criterion_group!(\n",
            "    benches,\n",
            "    bench_parse,\n",
            "    bench_validate\n",
            ");\n",
            "criterion_main!(benches);\n",
        );

        let symbols = parse(source).expect("valid file");
        let kind = |name: &str| {
            symbols
                .iter()
                .find(|s| s.qualified_name == name)
                .unwrap_or_else(|| panic!("{name} is a symbol"))
                .kind
        };
        assert_eq!(kind("bench_parse"), SymbolKind::Benchmark);
        assert_eq!(kind("bench_validate"), SymbolKind::Benchmark);
        assert_eq!(
            kind("helper"),
            SymbolKind::Function,
            "an unregistered function is not a benchmark"
        );
        assert_eq!(
            kind("inner::bench_parse"),
            SymbolKind::Function,
            "a nested namesake is a different symbol and was never registered"
        );

        // The single-line short form, and the long `targets =` form.
        let short = parse("fn b() {\n}\ncriterion_group!(g, b);\n").expect("valid");
        assert_eq!(short[0].kind, SymbolKind::Benchmark);
        let long = parse(concat!(
            "fn b() {\n}\n",
            "criterion_group!(name = g; config = c(); targets = b);\n",
        ))
        .expect("valid");
        assert_eq!(long[0].kind, SymbolKind::Benchmark);

        // And the attribute form needs no registration at all.
        let attribute =
            parse("#[bench]\nfn b(x: &mut Bencher) {\n    let _ = x;\n}\n").expect("valid");
        assert_eq!(attribute[0].kind, SymbolKind::Benchmark);
    }

    /// TC-827, FR-051-AC-17 (CR-061): the registration is read from the
    /// macro's own argument nodes, not from a trimmed-line offset, so a
    /// trailing comment beside the invocation cannot hide it.
    #[test]
    fn tc827_a_trailing_comment_does_not_hide_the_registration() {
        let symbols = parse(concat!(
            "fn b() {\n",
            "}\n",
            "criterion_group!(g, b); // the registration, with a comment after it\n",
            "criterion_main!(g);\n",
        ))
        .expect("valid");
        assert_eq!(symbols[0].kind, SymbolKind::Benchmark);
    }

    /// TC-827, FR-051-AC-17 (CR-061): `targets` is matched as a whole
    /// identifier token immediately followed by `=`, so a bench or group
    /// whose *name* merely starts with it is still registered rather than
    /// eaten by the long-form parse.
    #[test]
    fn tc827_a_name_beginning_with_targets_is_not_the_targets_clause() {
        let symbols =
            parse("fn targets_parse() {\n}\ncriterion_group!(g, targets_parse);\n").expect("valid");
        assert_eq!(
            symbols[0].kind,
            SymbolKind::Benchmark,
            "the short form's list must survive a name starting with `targets`"
        );
    }

    /// TC-827, FR-051-AC-17 (CR-061): `fuzz_target!` declares no `fn`, so
    /// before this a fuzz-target file yielded **no symbol at all** and its
    /// tag bound to nothing. The symbol's span starts at line 1: the file's
    /// `//!` module header is the invocation's annotation block, and it is
    /// where the tag is naturally written.
    #[test]
    fn tc827_a_fuzz_target_is_one_symbol_spanning_its_file() {
        let source = concat!(
            "#![no_main]\n",
            "//! NFR-019 fuzz target (TC-579).\n",
            "\n",
            "use libfuzzer_sys::fuzz_target;\n",
            "\n",
            "fn helper() -> usize {\n",
            "    1\n",
            "}\n",
            "\n",
            "fuzz_target!(|data: &[u8]| {\n",
            "    let _ = data;\n",
            "});\n",
        );

        let symbols = parse(source).expect("valid file");
        let target = symbols
            .iter()
            .find(|s| s.kind == SymbolKind::FuzzTarget)
            .expect("the invocation mints a symbol");
        assert_eq!(target.qualified_name, "fuzz_target");
        assert_eq!(target.line, 10);
        assert_eq!(target.leading_line, 1, "the module header is in the span");
        assert_eq!(target.end_line, 12);
        assert!(target.container.is_none());

        // The helper beside it stays an ordinary function.
        assert_eq!(
            symbols
                .iter()
                .find(|s| s.qualified_name == "helper")
                .expect("helper")
                .kind,
            SymbolKind::Function
        );
    }

    /// PLAT-843 review finding F1: an unrelated top-level macro invocation
    /// *before* `fuzz_target!` must not hide it. [`fuzz_target`] used to
    /// inspect only the *first* top-level macro invocation and then check
    /// whether it was `fuzz_target` — so a preceding `lazy_static!`,
    /// `include!`, or (as here) any other macro silently dropped the whole
    /// symbol, with no diagnostic. Reverting the `filter_map`+`find` fix back
    /// to `find_map`+`filter` makes this fail.
    #[test]
    fn a_preceding_unrelated_macro_does_not_hide_the_fuzz_target() {
        let source = concat!(
            "#![no_main]\n",
            "//! NFR-019 fuzz target.\n",
            "\n",
            "use libfuzzer_sys::fuzz_target;\n",
            "\n",
            "lazy_static::lazy_static! {\n",
            "    static ref CONFIG: usize = 1;\n",
            "}\n",
            "\n",
            "fuzz_target!(|data: &[u8]| {\n",
            "    let _ = data;\n",
            "});\n",
        );

        let symbols = parse(source).expect("valid file");
        let target = symbols
            .iter()
            .find(|s| s.kind == SymbolKind::FuzzTarget)
            .expect("the invocation mints a symbol despite the earlier macro");
        assert_eq!(target.qualified_name, "fuzz_target");
    }

    /// FR-051-AC-1: `impl` blocks are scopes, not symbols — the highest-risk
    /// identity behaviour in this port (PLAT-843's own audit). The block
    /// itself mints nothing, and a method qualifies under the target type
    /// named after `for`, never the trait.
    #[test]
    fn impl_blocks_are_scopes_not_symbols() {
        let source = concat!(
            "struct Foo;\n",
            "impl std::fmt::Display for Foo {\n",
            "    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n",
            "        write!(f, \"Foo\")\n",
            "    }\n",
            "}\n",
        );
        let symbols = parse(source).expect("valid");
        assert!(
            symbols
                .iter()
                .all(|s| s.qualified_name != "Display" && s.qualified_name != "std::fmt::Display"),
            "the impl block itself must mint nothing: {symbols:?}"
        );
        let method = symbols
            .iter()
            .find(|s| s.qualified_name == "Foo::fmt")
            .expect("the method qualifies under the target type: {symbols:?}");
        assert_eq!(method.container.as_deref(), Some("Foo"));
    }

    /// FR-051 (PLAT-843 audit item d): `type`/`static`/`const`/`union` mint
    /// no symbols — only fn/mod/struct/enum/trait/impl are recognised.
    #[test]
    fn type_static_const_union_mint_no_symbols() {
        let source = concat!(
            "type Alias = u32;\n",
            "static X: u32 = 1;\n",
            "const Y: u32 = 2;\n",
            "union U { a: u32, b: f32 }\n",
        );
        let symbols = parse(source).expect("valid");
        assert!(
            symbols.is_empty(),
            "none of these declare a symbol: {symbols:?}"
        );
    }

    /// FR-051-AC-2 (PLAT-843 audit item c): the duplicate-identity
    /// flattening is preserved exactly, not fixed here (PLAT-845's ticket).
    #[test]
    fn nested_helpers_in_different_functions_share_one_identity() {
        let source = concat!(
            "fn outer_one() {\n",
            "    fn helper() {}\n",
            "}\n",
            "fn outer_two() {\n",
            "    fn helper() {}\n",
            "}\n",
        );
        let symbols = parse(source).expect("valid");
        let helpers: Vec<_> = symbols
            .iter()
            .filter(|s| s.qualified_name == "helper")
            .collect();
        assert_eq!(
            helpers.len(),
            2,
            "both nested helpers are extracted: {symbols:?}"
        );
        assert!(
            helpers.iter().all(|s| s.container.is_none()),
            "a fn is never a container, so both share container=None: {helpers:?}"
        );
    }

    /// PLAT-69: a rustfmt-split `#[cfg_attr(...)]` no longer truncates the
    /// leading span at the continuation line. Asserts `leading_line`
    /// directly, not only that the classification survives — a fixture
    /// checking binding alone can pass for the wrong reason.
    #[test]
    fn plat69_a_split_attribute_does_not_truncate_the_leading_span() {
        let source = concat!(
            "#[cfg_attr(\n",
            "    feature = \"x\",\n",
            "    ignore\n",
            ")]\n",
            "#[test]\n",
            "fn t() {\n",
            "}\n",
        );
        let symbols = parse(source).expect("valid");
        let t = symbols.iter().find(|s| s.qualified_name == "t").expect("t");
        assert_eq!(t.kind, SymbolKind::TestFunction);
        assert_eq!(t.leading_line, 1, "the split attribute's own start line");
    }

    /// PLAT-846: a block doc comment (`/** ... */`) joins the leading span,
    /// the same defect class as PLAT-69. Asserts `leading_line`.
    #[test]
    fn plat846_a_block_doc_comment_joins_the_leading_span() {
        let source = "/** doc */\n#[test]\nfn documented() {\n}\n";
        let symbols = parse(source).expect("valid");
        let t = symbols
            .iter()
            .find(|s| s.qualified_name == "documented")
            .expect("documented");
        assert_eq!(t.leading_line, 1, "the block doc comment's own start line");
    }

    /// PLAT-305 regression pin (already passing on `main`, fixed by CR-061 —
    /// not a demonstrated improvement of this port): `#[ignore]` before
    /// `#[test]` is still found.
    #[test]
    fn plat305_ignore_before_test_is_still_found() {
        let source = "#[ignore]\n#[test]\nfn t() {\n}\n";
        let symbols = parse(source).expect("valid");
        assert_eq!(symbols[0].kind, SymbolKind::TestFunction);
    }

    /// `mod tests { }` must not mark its non-`#[test]` helpers as tests —
    /// the false positive `quire-code-rs`'s own generic
    /// `"tests".starts_with("test")` heuristic would introduce if this
    /// adapter classified by module name rather than by attribute.
    /// `tc743_test_classification_per_language`
    /// (`src/symbols/mod.rs`) pins the same behaviour end to end; this pins
    /// it directly against `parse`.
    #[test]
    fn mod_tests_does_not_mark_its_non_test_helpers_as_tests() {
        let source = concat!(
            "mod tests {\n",
            "    #[test]\n",
            "    fn it_works() {\n",
            "    }\n",
            "    fn helper() {\n",
            "    }\n",
            "}\n",
        );
        let symbols = parse(source).expect("valid");
        let kind_of = |name: &str| {
            symbols
                .iter()
                .find(|s| s.qualified_name == name)
                .unwrap()
                .kind
        };
        assert_eq!(kind_of("tests::it_works"), SymbolKind::TestFunction);
        assert_eq!(
            kind_of("tests::helper"),
            SymbolKind::Function,
            "a non-#[test] helper inside mod tests must not classify as a test"
        );
    }

    /// FR-051-AC-9: a genuinely truncated file (nothing after the point of
    /// truncation for tree-sitter to resync against) still fails loudly,
    /// naming a line — the exact fixture `tests/fixtures/symbols/broken/
    /// truncated.rs` is, and what `tc749_unparseable_file_degrades_per_file`
    /// (`src/symbols/mod.rs`) pins through the full `extract_tree` path.
    #[test]
    fn a_truncated_file_fails_loudly_naming_a_line() {
        let source = "pub fn truncated() {\n    if true {\n        let x = 1;\n";
        let err = parse(source).expect_err("a truly truncated file must fail");
        assert!(err.contains("line 1"), "the reason must name a line: {err}");
    }

    /// Regression pin, found by the PLAT-843 differential against quire-rs's
    /// own source tree: a `#[test] fn` declared inside `proptest! { ... }`
    /// is invisible to tree-sitter (the macro's own argument syntax is not
    /// parsed as Rust items), so without `collect_proptest_tests` this class
    /// of test — including ones carrying real `#[trace(...)]` tags in this
    /// repo — would silently stop being extracted. Mirrors the exact shape
    /// of `src/parser/mod.rs`'s `tests::never_panics_on_arbitrary_utf8`.
    #[test]
    fn proptest_declared_tests_are_extracted_with_their_enclosing_container() {
        let source = concat!(
            "mod tests {\n",
            "    fn helper() -> u32 { 1 }\n",
            "\n",
            "    proptest! {\n",
            "        #![proptest_config(ProptestConfig::with_cases(10))]\n",
            "\n",
            "        /// a doc comment above the test\n",
            "        #[trace(\"TC-819\", \"FR-005-AC-7\")]\n",
            "        #[test]\n",
            "        fn never_panics(s in \".*\") {\n",
            "            let _ = s;\n",
            "        }\n",
            "\n",
            "        #[test]\n",
            "        fn tiers_compose(s in \".*\") {\n",
            "            let _ = s;\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let symbols = parse(source).expect("valid");
        let by_name = |name: &str| symbols.iter().find(|s| s.qualified_name == name);

        let never_panics = by_name("tests::never_panics").expect("proptest test extracted");
        assert_eq!(never_panics.kind, SymbolKind::TestFunction);
        assert_eq!(never_panics.container.as_deref(), Some("tests"));
        assert_eq!(
            never_panics.leading_line, 7,
            "the leading span reaches back through both attributes and the doc comment"
        );

        let tiers_compose =
            by_name("tests::tiers_compose").expect("second proptest test extracted");
        assert_eq!(tiers_compose.kind, SymbolKind::TestFunction);

        assert_eq!(
            by_name("tests::helper").expect("ordinary sibling fn").kind,
            SymbolKind::Function,
            "an ordinary function beside the proptest! block still extracts"
        );

        // `mod tests` itself is still the container symbol; the macro
        // invocation mints nothing of its own.
        assert!(symbols
            .iter()
            .all(|s| s.qualified_name != "proptest" && s.qualified_name != "tests::proptest"));
    }
}
