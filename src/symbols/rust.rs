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
//! - **A `fn` is a container for its own body (PLAT-845).** Two same-named
//!   helpers nested in two different enclosing functions used to both mint
//!   `qualified_name="helper", container=None` — a genuine identity
//!   collision, not a style quirk (`FR-051-AC-2` requires distinct ids).
//!   [`walk`] now recurses into a function's own body with *that function's*
//!   qualified name as the container, exactly the `::`-joined threading
//!   `mod`/`struct`/`trait`/`impl` already used — so `fn a() { fn helper()
//!   {} }` yields `helper`'s `qualified_name="a::helper"`,
//!   `container=Some("a")`. The function's *own* qualified name is
//!   unaffected (only what nests inside it), so every non-nested symbol's
//!   name is byte-identical to before this change. This is a deliberate,
//!   plain-`::` scheme that **trades the original collision class for a
//!   narrower, rarer one**, not one that eliminates collisions outright: a
//!   `mod`/`fn` (or `struct`/`fn`) pair sharing a literal name in the same
//!   file, each nesting a same-named helper, now collides where it did not
//!   before this change, because nothing marks which kind of scope
//!   contributed a segment — see
//!   `a_mod_and_a_function_sharing_a_name_can_still_collide_on_nested_helpers`
//!   in this module's own tests (its doc comment works the before/after
//!   through explicitly), and the PR body for the corpus-wide duplicate-id
//!   sweep that measured this new class at zero real occurrences against
//!   the dozens of real collisions the original flattening bug caused.
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
//! - **A trailing comment on the previous declaration's own line no longer
//!   joins the *next* declaration's leading span** (PLAT-897). This
//!   adapter had never had the check `python.rs`/`typescript.rs` each
//!   independently added to close the identical defect in their own ports
//!   (PLAT-868 PR #479 F4, PLAT-882 PR #481): [`leading_span`] moved into
//!   `mod.rs` as a shared helper both call, so this file inherits the fix
//!   rather than needing its own copy. Measured at zero real occurrences in
//!   this repo's own corpus at consolidation time — see the PR's
//!   differential report.
//!
//! What did **not** change: the hand-written lexer subsystem this file used
//! to carry (`LexedLine`, `ScanState`, `lex`, `lex_line`,
//! `opens_char_literal`, `opens_raw_string`, `raw_prefix_len`, `closes_raw`,
//! `check_balanced`) is deleted outright as a consequence of parsing
//! correctly, not as a deliberate optimisation — no benchmark exists for
//! that site alone.

use quire_rust_extraction::tree_sitter::Node;
use quire_rust_extraction::{parse_file, Language};

use super::{leading_span, RawSymbol, SymbolKind};

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
/// including a function's own name when recursing into its own body, since
/// a `fn` is a container for its own body (PLAT-845; see this module's own
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
                let symbol = function_symbol(child, source, container.clone());
                // A `fn` IS a container for its own body (PLAT-845): recurse
                // with the function's own qualified name, the same `::`
                // threading `mod`/`struct`/`trait`/`impl` already use, so a
                // symbol nested inside it qualifies under it rather than
                // under whatever container the function itself was found
                // in. The function's own qualified name is unchanged by
                // this — only what nests *inside* it is affected — so a
                // non-nested symbol's name is byte-identical to before.
                //
                // Derived from the pushed symbol's own `qualified_name`
                // (review N3) rather than recomputed with a second
                // `qualify(&container, &name)` call: a second computation
                // that happens to agree with `function_symbol`'s today would
                // silently diverge from it if that function's naming ever
                // changed, and nothing would catch it — the same two-path
                // divergence class the `proptest!` scanner fix above closes.
                let inner_container = match &symbol {
                    Some(s) => Some(s.qualified_name.clone()),
                    None => container.clone(),
                };
                if let Some(symbol) = symbol {
                    out.push(symbol);
                }
                walk(child, source, inner_container, out, registered_benches);
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
        leading_line: leading_span(node, is_annotation_node),
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
        leading_line: leading_span(node, is_annotation_node),
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

/// Whether `node` is a sibling kind this adapter treats as part of a leading
/// annotation block: an attribute (outer or inner) or a comment — `//`,
/// `///`, `/* */` or `/** */` alike. tree-sitter represents all four comment
/// forms with the same two node kinds (`line_comment`, `block_comment`),
/// distinguished only by an optional doc-marker field this adapter does not
/// need to inspect, so a plain block comment joins the span exactly as a
/// block *doc* comment does (PLAT-846) — there is no ticket-stated reason to
/// special-case the non-doc form back out, and doing so would be more code
/// for a distinction this adapter never otherwise makes.
///
/// This predicate feeds [`leading_span`] (`super::leading_span`, shared with
/// `python.rs`/`typescript.rs` since PLAT-897), which stops the contiguous
/// run at the first rejected sibling, the first blank-line gap, or an
/// accepted sibling that is itself trailing on the line its own preceding
/// sibling ends on. The first two are what fixed PLAT-69 and PLAT-846: a
/// multi-line attribute or a block doc comment is one sibling node
/// regardless of how many lines it spans. **The third did not exist here
/// before PLAT-897**: this adapter had not yet independently hit the
/// trailing-comment defect `python.rs`/`typescript.rs` each found and fixed
/// in their own ports (PLAT-868 PR #479 F4, PLAT-882 PR #481) —
/// `fn a() {}  // note\nfn b() {}` read `// note` as `b`'s own leading
/// annotation, exactly their shape, one tree-sitter node kind earlier (see
/// `leading_span`'s own doc in `mod.rs` for why this is a property of
/// tree-sitter, not of any one grammar).
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
                    qualified_name: qualified_name.clone(),
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
                // The identical PLAT-845 fix as the plain-AST `walk`: this
                // matched `fn` is a container for its own body too, so a
                // helper nested inside a `proptest!`-declared test qualifies
                // under *that test*, not under whatever container held the
                // `proptest!` block itself — otherwise the property would
                // hold in the ordinary-AST path and not here.
                scan_token_tree_for_fns(*body, source, &Some(qualified_name), out);
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
/// token sequence, where an outer attribute is `#` followed by a `[...]`
/// `token_tree` rather than a single `attribute_item` node, and an **inner**
/// attribute (`#![proptest_config(...)]`, scoping to the rest of the
/// `proptest! { ... }` block rather than to one test) is `#`, `!`, then the
/// same `[...]` `token_tree` — three flat tokens instead of two (review F6:
/// the two-token match alone stopped the run at the first inner attribute
/// above a `#[test] fn`, truncating `leading_line` there instead of reaching
/// past it, the same class of defect PLAT-69/PLAT-846 fixed for a real
/// `attribute_item`/`inner_attribute_item` node).
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
            // Inner attribute: `#`, `!`, `[...]` token_tree — checked before
            // the outer-attribute arm below so a `#!` pair is never
            // mistaken for `#`, `[...]` with the `!` mis-swallowed.
            "token_tree"
                if j >= 3 && children[j - 2].kind() == "!" && children[j - 3].kind() == "#" =>
            {
                if boundary_row.saturating_sub(prev.end_position().row) > 1 {
                    break;
                }
                if attribute_path_is_test(prev, source) {
                    is_test = true;
                }
                boundary_row = children[j - 3].start_position().row;
                j -= 3;
            }
            // Outer attribute: `#`, `[...]` token_tree.
            "token_tree" if j >= 2 && children[j - 2].kind() == "#" => {
                if boundary_row.saturating_sub(prev.end_position().row) > 1 {
                    break;
                }
                if attribute_path_is_test(prev, source) {
                    is_test = true;
                }
                boundary_row = children[j - 2].start_position().row;
                j -= 2;
            }
            _ => break,
        }
    }
    (boundary_row + 1, is_test)
}

/// Whether an attribute's `[...]` `token_tree` (either outer or inner form)
/// names a `#[test]`-family path, read the same way for both shapes in
/// [`flat_leading_span_and_test`].
fn attribute_path_is_test(token_tree: Node, source: &str) -> bool {
    let Ok(inner) = token_tree.utf8_text(source.as_bytes()) else {
        return false;
    };
    let path = inner
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(['(', ','])
        .next()
        .unwrap_or("")
        .trim();
    path.rsplit("::").next() == Some("test")
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

    /// PLAT-897: two adjacent `///` lines with **no blank line** between
    /// them must both join the span — the control the trailing-comment
    /// guard in `super::leading_span` needs. Only `tree-sitter-rust`'s
    /// `line_comment` node for the **doc-comment** forms (`///`/`//!`,
    /// which carry an inner `doc_comment` child) reports
    /// `end_position().row` one row past its own text; a plain `//`
    /// `line_comment` does not (PLAT-897 PR #485 review F3 — an earlier
    /// version of this doc named `line_comment` generally, which was
    /// over-broad and propagated into a filed ticket before being
    /// corrected). A one-hop trailing check that compared a candidate only
    /// to its immediate predecessor would read that offset as "line two
    /// starts on the row line one's content ends on" and misclassify every
    /// second `///` line as trailing on the first — confirmed as a real
    /// regression writing the guard's first version: before walking back to
    /// real code (rather than the immediate predecessor) before comparing,
    /// this returned `leading_line: 3` (`fn f`'s own line, both doc lines
    /// dropped) instead of `1`. The current implementation is immune to
    /// this regardless of which sibling kind carries the offset, because it
    /// never compares two annotation-kind siblings to each other at all.
    #[test]
    fn two_adjacent_doc_comment_lines_with_no_gap_both_join_the_span() {
        let source = concat!("/// line one\n", "/// line two\n", "fn f() {}\n",);
        let symbols = parse(source).expect("valid");
        let f = symbols.iter().find(|s| s.qualified_name == "f").unwrap();
        assert_eq!(
            f.leading_line, 1,
            "both adjacent doc lines must join the span: {symbols:?}"
        );
    }

    /// PLAT-897 PR #485 review F1: two annotation-kind siblings sharing one
    /// physical line, both trailing on the *real code* before them, must
    /// both be excluded — not just the nearer one. `// y`'s own immediate
    /// predecessor is `/* x */`, itself an accepted annotation sibling, so
    /// a trailing check that only compares a candidate to its immediate
    /// predecessor never reaches `fn a() {}` (the real code both comments
    /// trail on) and wrongly accepts `// y` as `b`'s leading annotation.
    /// Measured through the shared helper before this fix: `b.leading_line`
    /// came out `1` (both comments absorbed) instead of `2`.
    #[test]
    fn two_trailing_annotation_siblings_on_one_line_are_both_excluded() {
        let source = "fn a() {} /* x */ // y\nfn b() {}\n";
        let symbols = parse(source).expect("valid");
        let b = symbols.iter().find(|s| s.qualified_name == "b").unwrap();
        assert_eq!(
            b.leading_line, 2,
            "a block comment then a line comment, both trailing on a's line, \
             must not become b's leading annotation: {symbols:?}"
        );
    }

    /// PLAT-897 PR #485 review F2: the blank-line-gap stop is not just
    /// documented, it is tested. Plain `//` comments are offset-clean
    /// (see `two_adjacent_doc_comment_lines_with_no_gap_both_join_the_span`'s
    /// own doc), so this fixture isolates the gap check from the row-offset
    /// question entirely: a blank line between the comment block and `f`
    /// must exclude the comments, leaving `f`'s own line as its leading
    /// line. Confirmed as a real gap in coverage, not a redundant test:
    /// mutating the gap threshold (`mod.rs`'s `> 1` to `> 100`, disabling
    /// the stop) left every existing test green before this one existed.
    #[test]
    fn a_blank_line_before_a_comment_block_excludes_it() {
        let source = "// a\n// b\n\nfn f() {}\n";
        let symbols = parse(source).expect("valid");
        let f = symbols.iter().find(|s| s.qualified_name == "f").unwrap();
        assert_eq!(
            f.leading_line, 4,
            "a blank line must stop the comment block from joining f's span: {symbols:?}"
        );
    }

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

    /// FR-051-AC-2 (PLAT-845): a `fn` **is** a container for its own body —
    /// the same mechanism `mod`/`struct`/`trait`/`impl` already use, plain
    /// `::`, no new separator — so two same-named functions nested in two
    /// differently-named enclosing functions qualify (and therefore hash)
    /// distinctly. This replaces
    /// `nested_helpers_in_different_functions_share_one_identity`, which
    /// pinned the flattening bug as correct behaviour (PLAT-843 audit item
    /// c) — a test that fails once the code becomes right is the sharpest
    /// version of the defect this campaign keeps finding, so it is rewritten
    /// in place rather than left beside its replacement.
    #[test]
    fn nested_functions_in_different_outer_functions_receive_distinct_identities() {
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
            .filter(|s| s.qualified_name.ends_with("helper"))
            .collect();
        assert_eq!(
            helpers.len(),
            2,
            "both nested helpers are extracted: {symbols:?}"
        );
        assert_eq!(
            helpers
                .iter()
                .map(|s| s.qualified_name.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            2,
            "each nests under its own enclosing function, so the qualified \
             names must differ: {helpers:?}"
        );
        assert!(
            helpers
                .iter()
                .any(|s| s.qualified_name == "outer_one::helper"
                    && s.container.as_deref() == Some("outer_one")),
            "helper() {{ }} is qualified under outer_one: {helpers:?}"
        );
        assert!(
            helpers
                .iter()
                .any(|s| s.qualified_name == "outer_two::helper"
                    && s.container.as_deref() == Some("outer_two")),
            "helper() {{ }} is qualified under outer_two: {helpers:?}"
        );

        // The compound identity `Symbol::compute_id` actually hashes —
        // (language, path, qualified_name, kind) — must differ too, not
        // just the qualified name in isolation: this is what FR-051-AC-2
        // ("distinct ids") actually requires.
        let ids: std::collections::BTreeSet<String> = helpers
            .iter()
            .map(|s| {
                crate::symbols::Symbol::compute_id(
                    crate::traceability::SourceLanguage::Rust,
                    "src/example.rs",
                    &s.qualified_name,
                    s.kind,
                )
            })
            .collect();
        assert_eq!(
            ids.len(),
            2,
            "two nested functions in two different outer functions must \
             receive distinct symbol ids"
        );
    }

    /// PLAT-845 review (N4): composition is claimed in this module's own
    /// docs but was untested — nothing caught a regression in either shape.
    /// Two cases: a function nested three deep (`fn` inside `fn` inside
    /// `fn`, threading `::` at every level), and a function nested inside an
    /// `impl` method (crossing PLAT-843's impl-target-under-`for` rule,
    /// which this ticket must not disturb — the highest-risk interaction,
    /// since it is the one place two different container-threading rules
    /// compose).
    #[test]
    fn nesting_composes_through_functions_and_through_impl_methods() {
        let source = concat!(
            "fn a() {\n",
            "    fn mid() {\n",
            "        fn helper() {}\n",
            "    }\n",
            "}\n",
            "\n",
            "struct Foo;\n",
            "impl Foo {\n",
            "    fn bar() {\n",
            "        fn helper() {}\n",
            "    }\n",
            "}\n",
        );
        let symbols = parse(source).expect("valid");
        let by_name = |name: &str| symbols.iter().find(|s| s.qualified_name == name);

        let three_deep = by_name("a::mid::helper")
            .unwrap_or_else(|| panic!("three levels of fn nesting compose: {symbols:?}"));
        assert_eq!(three_deep.container.as_deref(), Some("a::mid"));
        assert_eq!(three_deep.kind, SymbolKind::Function);

        let impl_nested = by_name("Foo::bar::helper").unwrap_or_else(|| {
            panic!(
                "a fn nested inside an impl method qualifies under both the \
                 impl target and the method: {symbols:?}"
            )
        });
        assert_eq!(impl_nested.container.as_deref(), Some("Foo::bar"));
        assert_eq!(impl_nested.kind, SymbolKind::Function);

        // `Foo::bar` itself is unaffected by what nests inside it — still
        // qualified under the impl target alone, matching PLAT-843's rule.
        let bar = by_name("Foo::bar").unwrap_or_else(|| panic!("bar itself: {symbols:?}"));
        assert_eq!(bar.container.as_deref(), Some("Foo"));
    }

    /// PLAT-845 review (B2): **this PR introduces this collision.** Before
    /// this change, `mod parse { fn helper() {} }` and `fn parse() { fn
    /// helper() {} }` had *distinct* ids — the `mod`'s `helper` was already
    /// qualified `parse::helper` (containers were threaded through `mod`
    /// before this ticket), while the `fn`'s `helper` flattened to bare
    /// `helper` with `container=None` (PLAT-845's original bug). Fixing that
    /// flattening by threading a function's own qualified name into its
    /// body exactly like `mod`/`trait`/`impl` already do — plain `::`, no
    /// function-contributed segment marked differently — makes the `fn`'s
    /// `helper` qualify to `parse::helper` too, so it now collides with the
    /// `mod`'s `helper` where before it did not.
    ///
    /// This is **not** a pre-existing limitation the docstring can call
    /// "documented" as if it always existed — it is a new, narrower
    /// collision class this change trades the original, broader one for.
    /// The trade was approved conditioned on measurement, not assumed safe:
    /// the corpus-wide duplicate-id sweep (see the PR body) found **zero**
    /// real occurrences of this specific `mod`/`fn`-name-collision shape
    /// across the six measured repos, while the flattening bug it replaces
    /// was colliding dozens of real symbols in that same corpus (see the PR
    /// body's `tests::Identity` example). A distinguishing marker was
    /// explicitly declined in favour of qualified names that keep reading as
    /// real Rust paths. Pinned here, with this reasoning stated plainly, so
    /// a future change to the scheme is a deliberate, visible decision made
    /// with the trade-off in view, not a silent behaviour shift.
    #[test]
    fn a_mod_and_a_function_sharing_a_name_can_still_collide_on_nested_helpers() {
        let source = concat!(
            "mod parse {\n",
            "    fn helper() {}\n",
            "}\n",
            "fn parse() {\n",
            "    fn helper() {}\n",
            "}\n",
        );
        let symbols = parse(source).expect("valid");
        let helpers: Vec<_> = symbols
            .iter()
            .filter(|s| s.qualified_name == "parse::helper")
            .collect();
        assert_eq!(
            helpers.len(),
            2,
            "both nested helpers qualify identically: {symbols:?}"
        );
        assert!(
            helpers.iter().all(|s| s.kind == SymbolKind::Function),
            "both are plain functions, so kind does not disambiguate either: \
             {helpers:?}"
        );
        let ids: std::collections::BTreeSet<String> = helpers
            .iter()
            .map(|s| {
                crate::symbols::Symbol::compute_id(
                    crate::traceability::SourceLanguage::Rust,
                    "src/example.rs",
                    &s.qualified_name,
                    s.kind,
                )
            })
            .collect();
        assert_eq!(
            ids.len(),
            1,
            "this PR introduces this collision (see this test's own doc \
             comment) — accepted, measured at zero occurrences in the \
             six-repo corpus: {helpers:?}"
        );
    }

    /// `TC-1885` (`FR-051-AC-25`), PLAT-69: a rustfmt-split `#[cfg_attr(...)]`
    /// no longer truncates the leading span at the continuation line.
    /// Asserts `leading_line` directly, not only that the classification
    /// survives — a fixture checking binding alone can pass for the wrong
    /// reason.
    #[test]
    fn tc1885_a_split_attribute_does_not_truncate_the_leading_span() {
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

    /// `TC-1886` (`FR-051-AC-25`), PLAT-846: a block doc comment (`/** ...
    /// */`) joins the leading span, the same defect class as PLAT-69.
    /// Asserts `leading_line`.
    #[test]
    fn tc1886_a_block_doc_comment_joins_the_leading_span() {
        let source = "/** doc */\n#[test]\nfn documented() {\n}\n";
        let symbols = parse(source).expect("valid");
        let t = symbols
            .iter()
            .find(|s| s.qualified_name == "documented")
            .expect("documented");
        assert_eq!(t.leading_line, 1, "the block doc comment's own start line");
    }

    /// `TC-1879` (`FR-051-AC-25`), PLAT-305 regression pin (already passing
    /// on `main`, fixed by CR-061 — not a demonstrated improvement of this
    /// port): `#[ignore]` before `#[test]` is still found.
    #[test]
    fn tc1879_ignore_before_test_is_still_found() {
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
    fn tc1884_a_truncated_file_fails_loudly_naming_a_line() {
        let source = "pub fn truncated() {\n    if true {\n        let x = 1;\n";
        let err = parse(source).expect_err("a truly truncated file must fail");
        // Exact, not `contains("line 1")` (review F5): that substring also
        // matches "line 10".."line 19" — an assertion that cannot fail for
        // the reason it claims to test, this repo's own tracked defect class.
        assert_eq!(err, "line 1: unresolvable declaration structure (column 0)");
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

    /// Review F6: an inner attribute (`#![proptest_config(...)]`) directly
    /// above a `#[test] fn`, with no blank-line gap, must join the leading
    /// run the same way a real `inner_attribute_item` does for a top-level
    /// declaration (PLAT-69/PLAT-846) — `flat_leading_span_and_test`'s
    /// token-shape match previously only recognised the two-token outer form
    /// (`#`, `[...]`), so an inner attribute's three tokens (`#`, `!`,
    /// `[...]`) stopped the run one token early and `leading_line` landed on
    /// `#[test]` itself instead of the `#!...` line above it.
    #[test]
    fn proptest_inner_attribute_adjacent_to_test_joins_the_leading_span() {
        let source = concat!(
            "proptest! {\n",
            "    #![proptest_config(ProptestConfig::with_cases(10))]\n",
            "    #[test]\n",
            "    fn never_panics(s in \".*\") {\n",
            "        let _ = s;\n",
            "    }\n",
            "}\n",
        );
        let symbols = parse(source).expect("valid");
        let never_panics = symbols
            .iter()
            .find(|s| s.qualified_name == "never_panics")
            .expect("proptest test extracted");
        assert_eq!(never_panics.kind, SymbolKind::TestFunction);
        assert_eq!(
            never_panics.leading_line, 2,
            "the inner attribute, with no blank line before #[test], is part of the leading span"
        );
    }

    /// PLAT-845, condition 2 of the review: `scan_token_tree_for_fns` (the
    /// `proptest!` token-scanner) carried the identical flattening bug as
    /// `walk` — a helper nested inside a `proptest!`-declared test's own
    /// body did not qualify under that test, so two same-named helpers
    /// nested in two different `proptest!` tests would have shared an
    /// identity exactly like the plain-`fn` case this ticket fixes. Fixing
    /// only `walk` would have left the property true in one path and false
    /// in the other; this pins that both paths now agree.
    #[test]
    fn a_helper_nested_inside_a_proptest_declared_test_qualifies_under_that_test() {
        let source = concat!(
            "proptest! {\n",
            "    #[test]\n",
            "    fn test_one(s in \".*\") {\n",
            "        fn helper() {}\n",
            "        helper();\n",
            "        let _ = s;\n",
            "    }\n",
            "    #[test]\n",
            "    fn test_two(s in \".*\") {\n",
            "        fn helper() {}\n",
            "        helper();\n",
            "        let _ = s;\n",
            "    }\n",
            "}\n",
        );
        let symbols = parse(source).expect("valid");
        let helpers: Vec<_> = symbols
            .iter()
            .filter(|s| s.qualified_name.ends_with("helper"))
            .collect();
        assert_eq!(
            helpers.len(),
            2,
            "both nested helpers extracted: {symbols:?}"
        );
        assert!(
            helpers
                .iter()
                .any(|s| s.qualified_name == "test_one::helper"
                    && s.container.as_deref() == Some("test_one")),
            "helper nested in test_one qualifies under it: {helpers:?}"
        );
        assert!(
            helpers
                .iter()
                .any(|s| s.qualified_name == "test_two::helper"
                    && s.container.as_deref() == Some("test_two")),
            "helper nested in test_two qualifies under it: {helpers:?}"
        );

        // Same as this test's plain-AST sibling
        // (`nested_functions_in_different_outer_functions_receive_distinct_identities`):
        // the two paths exist to agree, so this pins the same compound
        // identity assertion, not just the qualified name.
        let ids: std::collections::BTreeSet<String> = helpers
            .iter()
            .map(|s| {
                crate::symbols::Symbol::compute_id(
                    crate::traceability::SourceLanguage::Rust,
                    "src/example.rs",
                    &s.qualified_name,
                    s.kind,
                )
            })
            .collect();
        assert_eq!(
            ids.len(),
            2,
            "two nested helpers in two different proptest!-declared tests \
             must receive distinct symbol ids, the same as the plain-AST path"
        );
    }

    /// TC-1924, FR-051-AC-14 (PLAT-897): a comment trailing on the same
    /// line as the declaration *before* it must not join the *next*
    /// declaration's leading span. This adapter had no such check before
    /// PLAT-897 hoisted `leading_span` into a shared helper both
    /// `python.rs` and `typescript.rs` already had their own copy of
    /// (PLAT-868 PR #479 F4, PLAT-882 PR #481) — confirmed as a real,
    /// live regression here first: before this fix, `b.leading_line` read
    /// `1` (`// note`'s own line, which is `a`'s line), not `2`.
    #[test]
    fn tc1924_a_trailing_comment_on_the_previous_line_is_not_a_leading_annotation() {
        let source = "fn a() {}  // note\nfn b() {}\n";
        let symbols = parse(source).expect("valid");
        let b = symbols.iter().find(|s| s.qualified_name == "b").unwrap();
        assert_eq!(
            b.leading_line, 2,
            "a same-line trailing comment must not be pulled into the leading span: {symbols:?}"
        );
    }

    /// A standalone comment on its own line, immediately above the
    /// declaration, is still its leading annotation — the control for
    /// `tc1924` above: the trailing-comment stop must not also reject a
    /// genuine leading comment.
    #[test]
    fn a_standalone_leading_comment_still_joins_the_span() {
        let source = "// leading\nfn f() {}\n";
        let symbols = parse(source).expect("valid");
        let f = symbols.iter().find(|s| s.qualified_name == "f").unwrap();
        assert_eq!(
            f.leading_line, 1,
            "the standalone comment is f's own leading line"
        );
    }
}
