//! TypeScript source adapter (FR-051).
//!
//! Parses over a tree-sitter syntax tree, through the dependency-boundary
//! crate `quire-rust-extraction` (PLAT-843 for Rust; the crate's TypeScript
//! feature wired by PLAT-851; this file is the adapter that finally uses
//! it, PLAT-882) — see that crate's own module docs for why the
//! tree-sitter dependency itself is never named here. This adapter still
//! answers exactly the questions FR-051 needs — which declaration owns a
//! span, what its qualified name and kind are, whether it is a test — and
//! nothing more: no build, no type resolution, no dependency installation,
//! and no execution of the extracted code (FR-051-CON-1).
//!
//! **Form matching is a seam this adapter does not cross.** Which declared
//! marker or legacy trace form appears inside a symbol's span is
//! `trace.rs`'s question, answered afterwards against the resolved span
//! (`Symbol::attached_source`); this file has no reference to trace forms,
//! regex, or `TraceabilityModel`, and the tree walk below must not gain
//! one — including a query over `@decorator` nodes, which would look
//! tempting and belongs to PLAT-866 instead (explicit `@trace(...)`
//! constructs), landing after both this port and PLAT-868's Python port.
//!
//! Test classification follows the vitest/jest convention: a `test(...)` or
//! `it(...)` registration is a test symbol, and **its registered title is
//! the qualified name** (FR-051) — that is the identity a report or a trace
//! marker refers to, not an anonymous arrow function. A `describe(...)` /
//! `suite(...)` registration is a **container** symbol (CR-119): it groups
//! tests rather than being one, so it is a scope the registrations inside
//! it are parented to — never a `TestFunction`, which would make the suite
//! evidence in its own right.
//!
//! ## The defect this port exists to close (PLAT-163, FR-051-AC-25)
//!
//! The line-structural scanner this file used to be counted `{`/`}` as text,
//! one pass over the whole file, carrying comment/string/template state by
//! hand. A `{` inside a **regex literal** — `/[^}]*\{[^}]*\}/`, an ordinary
//! quantifier or character class — was never guarded the way a string or
//! comment was, so it desynchronised the depth counter, `check_balanced`
//! rejected the file outright, and every symbol in it — every trace tag —
//! vanished silently. PLAT-840/851 measured this **live** in this exact
//! corpus: three `filament-ide-rs` files abandoned, all this shape or its
//! sibling. tree-sitter tokenizes a `regex` node as its own grammar
//! construct; a `{` inside one is never a code brace to begin with, so
//! there is no counter left to desynchronise — the fix is a structural
//! consequence of parsing correctly, not a special case added here. See
//! `tc1881_a_brace_inside_a_regex_literal_is_content` below, and this
//! ticket's own differential report for the three files by name.
//!
//! ## What survives byte-for-byte from the line-structural scanner
//!
//! `Symbol::compute_id` hashes `(language, path, qualified_name, kind)`
//! (`src/symbols/mod.rs`) — qualified-name construction is an identity
//! surface, not a style choice, and every quirk below is preserved exactly
//! rather than "cleaned up" (PLAT-882, following PLAT-843's own rule):
//!
//! - **The file itself is a container symbol** (the module), named by its
//!   extension-less path, spanning the whole file.
//! - **A suite does not qualify its members.** `describe(...)`/`suite(...)`
//!   opens a scope (CR-119) that a nested registration's `container` points
//!   to, but a registration's `qualified_name` is always its own literal
//!   title — never prefixed by an enclosing suite's title, which would
//!   change the identity of every test in the ecosystem sitting inside one.
//! - **Only a `class`/`abstract class` qualifies.** `qualify` walks the
//!   scope stack for the nearest scope with `qualifies: true` — a class,
//!   never a suite — so `Foo.bar` for a method, but a bare title for a test
//!   however deeply nested in suites.
//! - **A function never becomes a container for its own body** (unlike the
//!   Rust adapter post-PLAT-845): a helper declared inside a top-level
//!   function, or inside a registration's callback, qualifies under
//!   whatever scope was already active when the function was entered, not
//!   under the function itself. The pre-port scanner never pushed a scope
//!   for a function body either (only `describe`/`suite`/`class` did), and
//!   this port makes no identity-changing decision beyond what PLAT-882
//!   asks for.
//! - **A registration's chain may name a suite anywhere along it**
//!   (CR-121): `test.describe(...)` and `it.describe.only(...)` classify
//!   exactly as `describe(...)` does, because a harness spells its suite as
//!   a member of its own test namespace.
//! - **A `constructor` method mints no symbol.** Preserved from the old
//!   scanner's `RESERVED` denylist, the one entry of it that still applies
//!   once real declaration structure replaces line matching (see below).
//! - **Only a `const`/`let`/`var` declarator whose value is a *parenthesized*
//!   arrow function is a function symbol.** `const f = (x) => {}` is one;
//!   `const f = x => x` (the bare, unparenthesized single-parameter form)
//!   is not, matching the old regex's own `\(` requirement exactly — read
//!   here off the grammar's own distinction between an `arrow_function`'s
//!   `parameters` field (parenthesized) and its `parameter` field (bare).
//!   A class field's arrow value (`readonly ready = () => true;`, no
//!   `const`/`let`/`var`) is not a function symbol either, for the same
//!   reason the old regex never matched it: no declaration keyword at the
//!   line's own start.
//! - **Only a plain identifier method name mints a symbol.** A method whose
//!   name is computed (`[key]() {}`), a string literal, or a private field
//!   (`#foo() {}`) mints nothing — the old regex's identifier-class match
//!   never recognised any of these either.
//! - **An `interface`, `enum`, or `type` alias mints no symbol at all** — not
//!   the declaration itself and not its members (an interface's
//!   `method_signature`/`property_signature` bodies are type-only, never
//!   evidence of executable structure). `walk` names no node kind for any of
//!   the three, so they fall through to the same default recursion every
//!   other unrecognised node kind does — nothing mints, but a declaration
//!   nested inside one (there is never one) would still be found. The old
//!   regex's `NAME(...) {`/`const NAME = (...) =>` shapes never matched an
//!   interface member or a `type X = {...}` alias either, so this is the
//!   same behaviour carried forward, made explicit here because it was
//!   previously true only by the old regex's own accident of shape
//!   (PLAT-882 review finding F5) — see
//!   `tc1923_an_interface_method_signature_mints_no_symbol` below for the
//!   regression that pins it.
//!
//! ## What changed, and why each delta is free (PLAT-882, following
//! PLAT-843's own cause-based rule: *free from parsing correctly → fix here
//! and name the delta; requires changing what a symbol **is** or how it is
//! **named** → a separate ticket*)
//!
//! - **The whole-file `check_balanced` rejection is gone** — see "The
//!   defect this port exists to close" above. tree-sitter recovers locally
//!   from a body-local error (the body-node rule `quire-code-parse` itself
//!   implements: an error is body-local iff it lies within a declaration's
//!   own executable body), so a broken function body no longer abandons the
//!   whole file either — the same false-positive-whole-file-rejection shape
//!   PLAT-843 closed for Rust, now closed here.
//! - **No more `TITLE_LOOKAHEAD_LINES` window.** The old scanner had to
//!   bound how many physical lines past a registration's opening line it
//!   would scan for a title, because it had no notion of "this call's first
//!   argument" beyond line position. A tree-sitter `call_expression`'s
//!   `arguments` field names that argument directly regardless of how the
//!   call is wrapped across lines, so the bound (and the class of bug it
//!   guarded — a title written arbitrarily far down a long argument list
//!   being read as this call's own) is structurally impossible rather than
//!   merely bounded.
//! - **A `get`/`set` accessor method now mints a symbol.** The old regex
//!   matched `NAME(...) {`  immediately — `get foo() {}` does not match
//!   that shape (`get` itself would have to be the captured name, and then
//!   `foo` — not `(` — follows it), so accessor methods were silently never
//!   extracted. tree-sitter's `method_definition` node covers a getter or
//!   setter the same as an ordinary method; recognising it is the same free
//!   recovery Rust's port made for `pub(in path)` and `unsafe impl` — an
//!   accidental gap in what the old matcher's shape could recognise, not a
//!   deliberate exclusion (`constructor` alone was deliberate, and stays
//!   excluded, see above).
//! - **A leading comment or decorator is found through an `export`
//!   keyword.** `// docs\nexport class Foo {}` — the comment is a sibling
//!   of the `export_statement` wrapping the class, not of the
//!   `class_declaration` itself, so [`leading_span`] resolves the span's
//!   true syntactic top ([`stmt_anchor`]) before walking preceding
//!   siblings. The old line scanner never had this problem (it read
//!   `export class Foo {` as one line regardless of any wrapping node), so
//!   this is new code closing a gap tree-sitter's own AST shape introduces,
//!   not a behaviour the old scanner had and this one lost.
//! - **A title containing an escaped quote is read in full.** The old
//!   `quoted()` took the raw byte span up to the *first* occurrence of the
//!   matching quote character, with no escape awareness — `"it's \"real\""`
//!   would have been cut short at the first embedded `\"`. tree-sitter's
//!   `string`/`template_string` node span is the lexer's own correct
//!   accounting of where the literal actually ends, so reading the node's
//!   own text is a free correctness fix in the same shape as the Rust
//!   adapter's move to reading the AST's `body` field instead of counting
//!   bytes.
//!
//! What did **not** change: the hand-written lexer subsystem this file used
//! to carry (`lex`, `lex_line`, `ScanState`, `LexedLine`, `check_balanced`,
//! `registration_open`, `close_paren`, `read_title`, `quoted`,
//! `block_end`, the `re_arrow_const`/`re_method` regexes and the
//! `RESERVED` keyword-shape denylist they needed) is deleted outright as a
//! consequence of parsing correctly, not as a deliberate optimisation.

use quire_rust_extraction::tree_sitter::Node;
use quire_rust_extraction::{parse_file, Language};

use super::{leading_span, RawSymbol, SymbolKind};

/// Parse `source` into raw symbols, or return a per-file reason to skip it.
///
/// **A declaration-structure failure fails loudly, naming the line** — the
/// same contract the Rust adapter documents (`src/symbols/rust.rs`), backed
/// by the same `quire-code-parse` diagnostic and the same body-node rule.
pub(crate) fn parse(path: &str, source: &str) -> Result<Vec<RawSymbol>, String> {
    // `.tsx` needs the TSX grammar (JSX syntax is a genuine parse ambiguity
    // against a bare type assertion in `.ts`); every other extension this
    // adapter is ever called for is `.ts` (`language_of` in `mod.rs` binds
    // only these two extensions to `SourceLanguage::Typescript`).
    let language = if path.ends_with(".tsx") {
        Language::Tsx
    } else {
        Language::TypeScript
    };
    let parsed = parse_file(language, "<source>", source).map_err(|e| format!("parse: {e}"))?;
    if let Some(diagnostic) = parsed.diagnostic() {
        return Err(format!(
            "line {}: unresolvable declaration structure (column {})",
            diagnostic.line(),
            diagnostic.column()
        ));
    }

    let module = module_name(path);
    let line_count = source.lines().count().max(1);
    let mut out = vec![RawSymbol {
        qualified_name: module.clone(),
        kind: SymbolKind::Container,
        line: 1,
        leading_line: 1,
        end_line: line_count,
        container: None,
    }];

    let mut scopes: Vec<Scope> = Vec::new();
    walk(parsed.root_node(), source, &module, &mut scopes, &mut out);
    Ok(out)
}

/// One open lexical scope: what it is called, and whether declarations
/// inside it are *named through* it.
///
/// The two axes are separate because a suite is a scope that does not
/// rename its members (CR-119). A class qualifies — `Harness.ready` — but a
/// registration's qualified name **is its registered title** (FR-051), so
/// prefixing it with the enclosing `describe(…)` title would change the
/// identity of every test in the ecosystem that sits inside one.
struct Scope {
    name: String,
    qualifies: bool,
}

/// The nearest enclosing scope that names its members — a class, never a
/// suite.
fn qualify(scopes: &[Scope], name: &str) -> String {
    match scopes.iter().rev().find(|s| s.qualifies) {
        Some(scope) => format!("{}.{name}", scope.name),
        None => name.to_string(),
    }
}

/// The nearest enclosing scope of **any** kind, or the file's own module.
///
/// A suite counts here even though it does not qualify: `container` is what
/// records that a `describe(…)` groups the registrations written inside it.
fn scope_container(scopes: &[Scope], module: &str) -> Option<String> {
    scopes
        .last()
        .map(|scope| scope.name.clone())
        .or_else(|| Some(module.to_string()))
}

/// The callees this adapter recognises as opening a **test** registration.
/// Its title is the test symbol's qualified name (FR-051).
const TEST_NAMES: &[&str] = &["test", "it"];

/// The callees this adapter recognises as opening a **suite** registration
/// — a container, not evidence (CR-119). `context` is deliberately absent;
/// see the pre-port scanner's own measured rationale, preserved unchanged:
/// zero `context(`/`suite(` registrations and zero `context.` false
/// positives would have been admitted or excluded differently across the
/// measured corpus, so nothing here is lost by leaving it out.
const SUITE_NAMES: &[&str] = &["describe", "suite"];

/// Walk `node`'s named children, minting a [`RawSymbol`] for each
/// declaration or registration and recursing to find every nested one.
///
/// `scopes` is the open scope stack (CR-119): a `describe`/`suite`
/// registration or a `class` declaration pushes one before recursing into
/// its own body and pops it afterward: exactly the syntactic nesting the
/// old scanner approximated with brace-depth bookkeeping, now read directly
/// off the tree.
fn walk(node: Node, source: &str, module: &str, scopes: &mut Vec<Scope>, out: &mut Vec<RawSymbol>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "class_declaration" | "abstract_class_declaration" => {
                if let Some(name) = field_text(child, "name", source) {
                    let qualified = qualify(scopes, &name);
                    let container = scope_container(scopes, module);
                    out.push(symbol_at(
                        stmt_anchor(child),
                        qualified.clone(),
                        SymbolKind::Container,
                        container,
                    ));
                    scopes.push(Scope {
                        name: qualified,
                        qualifies: true,
                    });
                    walk(child, source, module, scopes, out);
                    scopes.pop();
                } else {
                    // An anonymous default-export class (`export default
                    // class { ... }`) mints no symbol — the old scanner's
                    // `ident()` truncation of an empty name matched
                    // nothing either — but its body is still walked so any
                    // declaration nested inside it is found, under
                    // whichever scope was already active.
                    walk(child, source, module, scopes, out);
                }
            }
            "function_declaration" => {
                if let Some(name) = field_text(child, "name", source) {
                    let qualified = qualify(scopes, &name);
                    let container = scope_container(scopes, module);
                    out.push(symbol_at(
                        stmt_anchor(child),
                        qualified,
                        SymbolKind::Function,
                        container,
                    ));
                }
                // A function never becomes a container for its own body
                // (see this module's own docs) — recurse with `scopes`
                // unchanged, exactly as the old scanner's flat line scan
                // never pushed a scope for a function either.
                walk(child, source, module, scopes, out);
            }
            "method_definition" => {
                if let Some(name) = method_name(child, source) {
                    if name != "constructor" {
                        let qualified = qualify(scopes, &name);
                        let container = scope_container(scopes, module);
                        out.push(symbol_at(child, qualified, SymbolKind::Function, container));
                    }
                }
                walk(child, source, module, scopes, out);
            }
            "lexical_declaration" | "variable_declaration" => {
                mint_arrow_const_declarators(child, source, module, scopes, out);
                walk(child, source, module, scopes, out);
            }
            "call_expression" => match registration(child, source) {
                Some(Registration {
                    title,
                    kind: RegistrationKind::Test,
                }) => {
                    let container = scope_container(scopes, module);
                    out.push(symbol_at(
                        stmt_anchor(child),
                        title,
                        SymbolKind::TestFunction,
                        container,
                    ));
                    walk(child, source, module, scopes, out);
                }
                Some(Registration {
                    title,
                    kind: RegistrationKind::Suite,
                }) => {
                    let container = scope_container(scopes, module);
                    out.push(symbol_at(
                        stmt_anchor(child),
                        title.clone(),
                        SymbolKind::Container,
                        container,
                    ));
                    scopes.push(Scope {
                        name: title,
                        qualifies: false,
                    });
                    walk(child, source, module, scopes, out);
                    scopes.pop();
                }
                None => walk(child, source, module, scopes, out),
            },
            _ => walk(child, source, module, scopes, out),
        }
    }
}

/// Mint a symbol for every `const`/`let`/`var` declarator in `decl` whose
/// value is a *parenthesized* arrow function — see this module's own docs
/// for why the bare single-parameter form (`x => x`) does not qualify, and
/// why a class field's arrow value does not either (it is never a
/// `lexical_declaration`/`variable_declaration` in the first place).
///
/// Every declarator in a comma-separated statement is minted, not only the
/// first — a genuine, free recovery over the old regex, which matched only
/// the first `NAME = (` shape in its own captured group.
fn mint_arrow_const_declarators(
    decl: Node,
    source: &str,
    module: &str,
    scopes: &[Scope],
    out: &mut Vec<RawSymbol>,
) {
    let mut cursor = decl.walk();
    for declarator in decl.named_children(&mut cursor) {
        if declarator.kind() != "variable_declarator" {
            continue;
        }
        let Some(name_node) = declarator.child_by_field_name("name") else {
            continue;
        };
        if name_node.kind() != "identifier" {
            // A destructuring pattern (`const { a, b } = ...`) is never an
            // arrow-function declarator shape the old regex recognised.
            continue;
        }
        let Some(value) = declarator.child_by_field_name("value") else {
            continue;
        };
        if value.kind() != "arrow_function" {
            continue;
        }
        if value.child_by_field_name("parameters").is_none() {
            // The bare, unparenthesized single-parameter form — outside the
            // old regex's own `\(` requirement.
            continue;
        }
        let Ok(name) = name_node.utf8_text(source.as_bytes()) else {
            continue;
        };
        let qualified = qualify(scopes, name);
        let container = scope_container(scopes, module);
        out.push(symbol_at(
            stmt_anchor(decl),
            qualified,
            SymbolKind::Function,
            container,
        ));
    }
}

/// `node`'s plain-identifier method name, or `None` when it is computed, a
/// string literal, a numeric literal, or a private field — none of which
/// the old regex's identifier-class match ever recognised either.
fn method_name(node: Node, source: &str) -> Option<String> {
    let name_node = node.child_by_field_name("name")?;
    if name_node.kind() != "property_identifier" {
        return None;
    }
    name_node
        .utf8_text(source.as_bytes())
        .ok()
        .map(str::to_string)
}

/// `node`'s field named `field`, as plain source text, or `None` when the
/// field is absent.
fn field_text(node: Node, field: &str, source: &str) -> Option<String> {
    node.child_by_field_name(field)
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .map(str::to_string)
}

/// The syntactic top of a declaration or registration statement, for span
/// and leading-annotation purposes: `node` itself, or the `export_statement`
/// wrapping it when it is exported. A leading comment or decorator attaches
/// before `export`, not before the declaration `export` wraps — see this
/// module's own docs.
///
/// Also climbs `await_expression` (review round 3, F4): `await it('x', …)`
/// wraps the registration's `call_expression` in an `await_expression`
/// before the enclosing `expression_statement`, so without this a leading
/// comment attached to the statement was invisible through `await` — the
/// span search started one node too low to see it. Zero real occurrences in
/// either measured corpus, but the shape is legal TypeScript (an async
/// registration helper) and there is no reason `await` alone should change
/// where a leading annotation binds.
fn stmt_anchor(node: Node) -> Node {
    let mut current = node;
    while let Some(parent) = current.parent() {
        if matches!(
            parent.kind(),
            "expression_statement" | "export_statement" | "await_expression"
        ) {
            current = parent;
        } else {
            break;
        }
    }
    current
}

/// A `RawSymbol` anchored at `anchor` — its own start line, its own leading
/// annotation span, and its own end line — the one shape every declaration
/// and registration this adapter mints shares.
fn symbol_at(
    anchor: Node,
    qualified_name: String,
    kind: SymbolKind,
    container: Option<String>,
) -> RawSymbol {
    RawSymbol {
        qualified_name,
        kind,
        line: anchor.start_position().row + 1,
        leading_line: leading_span(anchor, is_annotation_node),
        end_line: anchor.end_position().row + 1,
        container,
    }
}

/// Whether `node` is a sibling kind this adapter treats as part of a
/// leading annotation block: a comment (`//`, `/* */`, `/** */` alike —
/// tree-sitter's TypeScript/TSX grammar represents all three with the same
/// `comment` node kind) or a decorator (`@Foo(...)`).
///
/// This predicate feeds [`leading_span`] (`super::leading_span`, shared
/// with `python.rs`/`rust.rs` since PLAT-897), which stops the contiguous
/// run of accepted siblings at the first sibling of another kind, the
/// first blank-line gap, or a sibling that is itself **trailing** on the
/// line of whatever precedes it — the same stopping conditions the
/// line-structural scanner used (a trailing `// ...` never started its own
/// line, so the old scanner never read it as an annotation line at all),
/// now checked between sibling nodes instead of between lines, which is
/// what lets a `/** ... */` JSDoc block or a multi-line `@Decorator(...)`
/// join the span as one sibling regardless of how many lines it spans (the
/// same class of fix PLAT-69/PLAT-846 made for the Rust adapter).
///
/// The trailing-comment stop exists because tree-sitter represents `const
/// re = /../; // note` as two siblings, a declaration and a `comment`, and
/// without it the comment — nearest-preceding, not itself preceded by a
/// blank-line gap — would be read as *the next declaration's* leading
/// annotation, silently pulling a trailing note on one statement into the
/// span (and binding search) of an unrelated one below it (discovered
/// authoring `tc803_one_reading_decides_whether_delimiters_are_code`'s span
/// assertions, PLAT-882 PR #481 review). `python.rs` independently hit and
/// fixed the identical defect the same week (PLAT-868 PR #479 F4); `rust.rs`
/// had not yet, until PLAT-897 moved this walk into `mod.rs` as one shared
/// implementation all three adapters call.
fn is_annotation_node(node: Node) -> bool {
    matches!(node.kind(), "comment" | "decorator")
}

/// What a matched registration call is, and its title.
struct Registration {
    title: String,
    kind: RegistrationKind,
}

enum RegistrationKind {
    Test,
    Suite,
}

/// Whether `call` is a `test`/`it`/`describe`/`suite` registration, and if
/// so its title and classification.
///
/// A registration whose chain names a suite **anywhere along it**
/// (CR-121) — `test.describe(...)`, `it.describe.only(...)` — classifies as
/// a suite exactly as `describe(...)` does, because a harness spells its
/// suite as a member of its own test namespace.
fn registration(call: Node, source: &str) -> Option<Registration> {
    let function = call.child_by_field_name("function")?;
    let (base, chain) = callee_chain(function, source)?;
    let is_test_name = TEST_NAMES.contains(&base.as_str());
    let is_suite_name = SUITE_NAMES.contains(&base.as_str());
    if !is_test_name && !is_suite_name {
        return None;
    }
    let names_a_suite =
        is_suite_name || chain.iter().any(|seg| SUITE_NAMES.contains(&seg.as_str()));

    let arguments = call.child_by_field_name("arguments")?;
    let mut cursor = arguments.walk();
    let mut args = arguments.named_children(&mut cursor);
    let first_arg = args.next()?;
    // A real registration always carries a callback as its second argument
    // — every positive shape in this adapter's own fixtures does — so a
    // bare `it('title')` with nothing else is not one. This is what the
    // old scanner's `TITLE_LOOKAHEAD_LINES` bound accidentally also
    // enforced for a title written far enough down the file (never
    // reaching a title with no callback argument at all is exactly what a
    // three-line lookahead cannot do on its own), and it is preserved here
    // deliberately now that a lookahead window is no longer needed (see
    // this module's own docs): dropping it would recognise `it('title')`
    // alone as a registration, which FR-051-AC-18's negative fixture
    // (`tests/fixtures/symbols/typescript/registration.test.ts`,
    // `it(\n\n\n\n  'title',\n)`) pins as registering nothing.
    args.next()?;
    let title = title_of(first_arg, source)?;

    // `names_a_suite` and `is_test_name && !names_a_suite` are exhaustive
    // over the two remaining cases: `is_test_name || is_suite_name` was
    // already required above, so whichever of the two is false, the other
    // is true — there is no third outcome to fall through to (PLAT-882
    // review finding F10: the old trailing `else { None }` here was
    // unreachable).
    if names_a_suite {
        Some(Registration {
            title,
            kind: RegistrationKind::Suite,
        })
    } else {
        Some(Registration {
            title,
            kind: RegistrationKind::Test,
        })
    }
}

/// Resolve a call's callee to its base identifier and the ordered list of
/// `.member` segments between the base and this call — `it.skipIf(cond)`
/// resolves to `("it", ["skipIf"])`; `it.concurrent.skip` resolves to
/// `("it", ["concurrent", "skip"])`. `None` when the callee is not a plain
/// identifier/member/curried-call chain (e.g. a subscript or computed
/// callee), matching the pre-port scanner's own miss for such shapes.
///
/// A curried modifier call (`it.skipIf(cond)`, the inner `call_expression`)
/// contributes only its own callee's chain; its own arguments (the
/// modifier's own call args, `cond` here) are never inspected — the same
/// discard the pre-port scanner applied to a curried group's contents.
fn callee_chain(function: Node, source: &str) -> Option<(String, Vec<String>)> {
    match function.kind() {
        "identifier" => {
            let name = function.utf8_text(source.as_bytes()).ok()?.to_string();
            Some((name, Vec::new()))
        }
        "member_expression" => {
            let object = function.child_by_field_name("object")?;
            let property = function.child_by_field_name("property")?;
            let (base, mut chain) = callee_chain(object, source)?;
            chain.push(property.utf8_text(source.as_bytes()).ok()?.to_string());
            Some((base, chain))
        }
        "call_expression" => {
            let inner = function.child_by_field_name("function")?;
            callee_chain(inner, source)
        }
        _ => None,
    }
}

/// The literal content of a `string`/`template_string` node — its own text
/// with the one leading and one trailing quote byte (`"`, `'` or `` ` ``)
/// stripped. `None` for any other argument shape (a variable, a number, a
/// template with no quote to strip), so a title held in a variable
/// registers nothing rather than something wrong, matching the pre-port
/// scanner's own refusal.
///
/// A **multi-line** template literal also returns `None` (FR-051-AC-18,
/// CR-084, preserved verbatim): the pre-port scanner read a title from a
/// single physical line, so a title spanning lines was already out of its
/// reach, and that exclusion is deliberate, not an accident of its
/// implementation — carrying it forward matters because `qualified_name`
/// feeds `Symbol::compute_id` and `plat843_audit_list` emits one row per
/// source line, so a literal newline in a qualified name would silently
/// split one symbol across two rows of a differential (PLAT-882 review
/// finding F1).
fn title_of(node: Node, source: &str) -> Option<String> {
    if !matches!(node.kind(), "string" | "template_string") {
        return None;
    }
    if node.kind() == "template_string" && node.start_position().row != node.end_position().row {
        return None;
    }
    let text = node.utf8_text(source.as_bytes()).ok()?;
    text.strip_prefix(['"', '\'', '`'])
        .and_then(|t| t.strip_suffix(['"', '\'', '`']))
        .map(str::to_string)
}

fn module_name(path: &str) -> String {
    let stem = path
        .strip_suffix(".tsx")
        .or_else(|| path.strip_suffix(".ts"))
        .unwrap_or(path);
    stem.to_string()
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::*;

    #[trace("TC-943", "FR-051-AC-18")]
    // a curried modifier registration, and a title (CR-084)
    // beginning on a later line, register a test symbol.
    //
    // `it.skipIf(cond)("TC-118 …")` produced **no symbol at all**, because the
    // single-line regex required the title's quote to follow `it(` immediately.
    // A registration the extractor cannot see is worse than a missed tag: the
    // test runs, passes, and neither a legacy comment id nor a canonical
    // `trace(…)` in its body has anything to bind to.
    #[test]
    fn tc943_curried_and_wrapped_registrations_bind() {
        let source = concat!(
            "describe(\"self-update\", () => {\n",
            "  it.skipIf(installed === null)(\n",
            "    \"TC-118 the installed CLI satisfies the pinned premise\",\n",
            "    () => { expect(1).toBe(1); },\n",
            "  );\n",
            "  it.each([1, 2])(\"TC-119 each case %i\", (n) => { expect(n).toBe(n); });\n",
            // Verbatim from `typesetter/tests/ToleranceTaxonomy.test.ts:69` —
            // the only occurrence in ~/dev carrying a trace id, and therefore
            // the only tag this change actually recovers. A shape copied from
            // the corpus, not one imagined for the test.
            "  it.each([...benignSamples, ...materialSamples])(\n",
            "    'FR-043-AC-2 the real corpus shape',\n",
            "    (id, input) => { expect(id).toBe(id); },\n",
            "  );\n",
            "  test(\n",
            "    'TC-120 a plain call wrapped for width',\n",
            "    () => { expect(2).toBe(2); },\n",
            "  );\n",
            "  it.concurrent.skip(\"TC-121 a chained modifier\", () => {});\n",
            "  it(\"TC-122 the ordinary form still binds\", () => {});\n",
            "});\n",
        );

        let symbols = parse("self-update.test.ts", source).expect("a valid file must parse");
        let tests: Vec<&str> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::TestFunction)
            .map(|s| s.qualified_name.as_str())
            .collect();

        assert_eq!(
            tests,
            vec![
                "TC-118 the installed CLI satisfies the pinned premise",
                "TC-119 each case %i",
                "FR-043-AC-2 the real corpus shape",
                "TC-120 a plain call wrapped for width",
                "TC-121 a chained modifier",
                "TC-122 the ordinary form still binds",
            ],
            "every registration form must yield exactly one test symbol",
        );
    }

    #[trace("TC-948", "FR-051-AC-18")]
    // the forward scan does not over-reach (CR-084;
    // re-idded from a duplicate TC-943 by CR-087 — one test-case id names one
    // source symbol).
    //
    // The failure this guards is subtler than the one it fixes. A scan that
    // hunted for *any* quote would name a test after an unrelated string, and a
    // wrong symbol name is worse than none: it binds a tag to the wrong
    // requirement instead of visibly binding nothing.
    #[test]
    fn tc948_the_forward_scan_refuses_what_is_not_a_title() {
        // A variable title. Must not walk on to any string elsewhere in the
        // call's arguments.
        let variable = concat!(
            "it(\n",
            "  name,\n",
            "  () => { expect(\"TC-500 not a title\").toBe(1); },\n",
            ");\n",
        );
        assert!(
            parse("a.test.ts", variable)
                .expect("parses")
                .iter()
                .all(|s| s.kind != SymbolKind::TestFunction),
            "a variable title registers nothing rather than something wrong",
        );

        // An identifier that merely starts with `it`.
        let lookalike = "iterate(\"TC-501 not a registration\", () => {});\n";
        assert!(
            parse("b.test.ts", lookalike)
                .expect("parses")
                .iter()
                .all(|s| s.kind != SymbolKind::TestFunction),
            "`iterate(` is not `it(`",
        );
    }

    #[trace("TC-798", "FR-051-AC-12")]
    // a `/*` inside a template literal is content, (CR-036)
    // not a comment opener.
    //
    // The stripper scanned raw characters, so one git refspec swallowed the
    // rest of the file: braces could not balance, `check_balanced` rejected it,
    // and a perfectly valid file yielded **zero** symbols. Nothing about that
    // is visible from the file — it parses, its tests pass, its trace tags are
    // present and greppable — so every tag in it bound to nothing in silence.
    // Asserted at the `parse` outcome level; the old mechanism-level
    // `lex_line`/`ScanState` assertions this test used to end with no longer
    // apply — there is no per-line lexer left to inspect (PLAT-882).
    #[test]
    fn tc798_comment_stripping_is_string_aware() {
        let source = concat!(
            "function gitConfig(url: string): string {\n",
            "  return `[remote \"origin\"]\\n\\turl = ${url}",
            "\\n\\tfetch = +refs/heads/*:refs/remotes/origin/*\\n`;\n",
            "}\n",
            "\n",
            "describe(\"FR-025-AC-9 remotes naming no organization\", () => {\n",
            "  /**\n",
            "   * Trace: FR-025-AC-9 — a local-path remote yields no organization.\n",
            "   */\n",
            "  test(\"never substitutes a path segment\", () => {\n",
            "    expect(originOrg(gitConfig(\"../repo\"))).toBeUndefined();\n",
            "  });\n",
            "});\n",
        );

        let symbols = parse("a.test.ts", source).expect("a valid file must parse");
        let test_symbol = symbols
            .iter()
            .find(|s| {
                s.qualified_name
                    .ends_with("never substitutes a path segment")
            })
            .expect("the registration is a test symbol");
        assert_eq!(test_symbol.kind, SymbolKind::TestFunction);

        // The span must reach back over the JSDoc, or the tag inside it binds to
        // nothing even though the file parsed.
        let span = source
            .lines()
            .skip(test_symbol.leading_line - 1)
            .take(test_symbol.end_line - test_symbol.leading_line + 1)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(span.contains("Trace: FR-025-AC-9"), "{span}");
    }

    /// TC-799, FR-051-AC-12 (CR-036): the same `/*`, on a **continuation** line
    /// of a multi-line template literal.
    ///
    /// A per-line scanner re-entered each line believing it was in code, so it
    /// re-opened the block comment one line later and the file was rejected
    /// exactly as before. tree-sitter tokenizes the whole template literal as
    /// one node regardless of line breaks, so there is no per-line state left
    /// to lose. Asserted at the `parse` outcome level (PLAT-882) — the old
    /// mechanism-level `lex_line`/`ScanState` carry-across-lines assertions no
    /// longer apply.
    #[test]
    fn tc799_template_literal_state_carries_across_lines() {
        let source = concat!(
            "const cfg = `\n",
            "[remote \"origin\"]\n",
            "fetch = +refs/heads/*:refs/remotes/origin/*\n",
            "`;\n",
            "\n",
            "describe(\"FR-001-AC-1 group\", () => {\n",
            "  /**\n",
            "   * Trace: FR-001-AC-1 — the thing holds.\n",
            "   */\n",
            "  test(\"holds\", () => {\n",
            "    expect(1).toBe(1);\n",
            "  });\n",
            "});\n",
        );

        let symbols = parse("a.test.ts", source).expect("a valid file must parse");
        let test_symbol = symbols
            .iter()
            .find(|s| s.qualified_name.ends_with("holds"))
            .expect("the registration is a test symbol");

        let span = source
            .lines()
            .skip(test_symbol.leading_line - 1)
            .take(test_symbol.end_line - test_symbol.leading_line + 1)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(span.contains("Trace: FR-001-AC-1"), "{span}");
    }

    /// A bare `{` inside a multi-line template literal must not count as a
    /// block open and must not unbalance the file (PLAT-882: tree-sitter
    /// tokenizes the whole template literal as one node, so there is no
    /// brace-depth counter left to desync in the first place). Asserted at
    /// the `parse` outcome level; the old mechanism-level `lex_line`
    /// assertion this test used to end with no longer applies.
    #[test]
    fn tc799_braces_inside_a_multiline_literal_do_not_unbalance() {
        let source = concat!(
            "const t = `\n",
            "a { bare brace\n",
            "`;\n",
            "test(\"holds\", () => {\n",
            "  expect(1).toBe(1);\n",
            "});\n",
        );
        let symbols = parse("a.test.ts", source).expect("a valid file must parse");
        assert!(
            symbols.iter().any(|s| s.qualified_name.ends_with("holds")),
            "the test after the literal must still be a symbol"
        );
    }

    /// TC-803, FR-051-AC-14 (CR-039): **successor to the retired
    /// `tc803_one_lex_serves_every_consumer`** (PLAT-882). That test asserted
    /// the internal state of the deleted single-pass lexer (`lex`,
    /// `check_balanced`, `LexedLine.delta`) — mechanism, not outcome, and
    /// that mechanism no longer exists. This keeps TC-803's own claim (one
    /// reading of a file decides what is code in it, for every consumer of
    /// that reading) but asserts it the only way it can be asserted now: at
    /// `parse`'s own outcome. `quire-code-parse` is the one reading here —
    /// one tree, one diagnostic — so a brace inside a block comment, inside a
    /// carried template literal, and after an unterminated quote are content
    /// to `parse` exactly because they were never `{`/`}` tokens in the tree
    /// to begin with, not because two different consumers happened to agree.
    ///
    /// Confirmed to fail first: reverting this file's `check_balanced` walk
    /// away from tree-sitter to a naive
    /// `source.matches('{').count() == source.matches('}').count()` text
    /// check reports an imbalance on every fixture below and rejects the
    /// file — exactly the false rejection this property forbids.
    ///
    /// FR-051-AC-14 names **two** things every consumer of "one reading"
    /// relies on: whether the file is accepted at all, **and the span a
    /// symbol ends at**. The first version of this successor asserted only
    /// acceptance and `kind`, dropping the span half the retired
    /// `tc803_one_lex_serves_every_consumer` used to cover — restored here
    /// (PLAT-882 review finding F3), with all three decoys placed in **one**
    /// file immediately before the target registration, so a wrong span
    /// contributed by any one decoy cannot hide behind the other two.
    #[test]
    fn tc803_one_reading_decides_whether_delimiters_are_code() {
        let source = concat!(
            "/* a comment holding a brace {\n   and closing here */\n", // 1-2
            "const cfg = `\na { bare brace in a literal\n`;\n",         // 3-5
            "const re = /['\"]/; // a comment after a quote-shaped regex {\n", // 6
            "test(\"holds\", () => {\n",                                // 7
            "  expect(1).toBe(1);\n",                                   // 8
            "});\n",                                                    // 9
        );
        let symbols = parse("a.test.ts", source).expect("the file balances");
        let test_symbol = symbols
            .iter()
            .find(|s| s.qualified_name == "holds")
            .expect("the registration is a test symbol");
        assert_eq!(test_symbol.kind, SymbolKind::TestFunction);
        // No decoy above line 7 is `test`'s own leading comment (none is
        // its immediate preceding sibling — line 6 is a statement), and the
        // span ends at its own closing `});`, not swallowed into or
        // truncated by anything above it.
        assert_eq!(test_symbol.leading_line, 7, "{test_symbol:?}");
        assert_eq!(test_symbol.line, 7, "{test_symbol:?}");
        assert_eq!(test_symbol.end_line, 9, "{test_symbol:?}");

        // Each adversarial shape confirmed independently too: none of the
        // three may desync the file on its own.
        let adversarial = [
            // A brace inside a block comment.
            "/* a comment holding a brace {\n   and closing here */\ntest(\"holds\", () => {\n  expect(1).toBe(1);\n});\n",
            // A brace inside a carried (multi-line) template literal.
            "const cfg = `\na { bare brace in a literal\n`;\ntest(\"holds\", () => {\n  expect(1).toBe(1);\n});\n",
            // A brace after what looks like an unterminated quote followed by
            // a trailing comment.
            "const re = /['\"]/; // a comment after a quote-shaped regex {\ntest(\"holds\", () => {\n  expect(1).toBe(1);\n});\n",
        ];
        for source in adversarial {
            let symbols = parse("a.test.ts", source).expect("the file balances");
            let test_symbol = symbols
                .iter()
                .find(|s| s.qualified_name.ends_with("holds"))
                .unwrap_or_else(|| panic!("the registration is a test symbol: {source:?}"));
            assert_eq!(test_symbol.kind, SymbolKind::TestFunction);
        }
    }

    /// TC-1881, FR-051-AC-25 (PLAT-163, PLAT-882): **the headline defect this
    /// ticket exists to close.** A brace inside a *regex literal* — the shape
    /// `agent-ix/filament-ide-rs` actually carries in
    /// `ui/tests/e2e/tc-784-786-789-project-switcher.spec.ts` and
    /// `ui/tests/native/it-019-sync-native.spec.ts` — desynchronised the old
    /// line-structural scanner's brace counter exactly as a brace in a string
    /// or comment did, and `check_balanced` abandoned the whole file: every
    /// trace tag in it bound to nothing, silently, even though the file is
    /// perfectly valid TypeScript. A regex literal was never guarded the way
    /// strings, comments and templates were.
    ///
    /// The control is the same file with the quantifier's braces removed —
    /// it must extract identically either way, proving the *symbols*, not
    /// merely "did it parse", are unaffected by the regex's own content.
    #[trace("TC-1881", "FR-051-AC-25")]
    #[test]
    fn tc1881_a_brace_inside_a_regex_literal_is_content() {
        let with_regex = concat!(
            "export type DiscoveredProjectPayload = {\n",
            "  path: string;\n",
            "};\n",
            "\n",
            "function parseInvoke(line: string) {\n",
            "  const pattern = /openDiscoveredProject:\\s*\\(([^)]*)\\)\\s*=>[^\\n]*",
            "__TAURI_INVOKE\\([^)]*\\{([^}]*)\\}/;\n",
            "  return pattern.exec(line);\n",
            "}\n",
            "\n",
            "describe(\"TC-1881 project switcher\", () => {\n",
            "  test(\"parses the discovered-project invoke call\", () => {\n",
            "    expect(parseInvoke(\"x\")).toBeNull();\n",
            "  });\n",
            "});\n",
        );

        let symbols = parse("project-switcher.spec.ts", with_regex)
            .expect("a brace inside a regex literal must not abandon the file");
        // `type X = {...}` mints no symbol either way (a type alias, like an
        // interface, is outside what either the old or the new adapter
        // recognises) — what matters is that the *file* is not abandoned,
        // so the declarations around it still extract.
        assert!(
            symbols.iter().any(|s| s.qualified_name == "parseInvoke"),
            "the function whose body holds the regex must still extract: {symbols:?}"
        );
        let test_symbol = symbols
            .iter()
            .find(|s| s.qualified_name == "parses the discovered-project invoke call")
            .expect("the test after the regex must still extract");
        assert_eq!(test_symbol.kind, SymbolKind::TestFunction);

        // A quantifier's `{n,m}` form is the same shape (FR-051-AC-25's own
        // wording) and must be equally inert.
        let with_quantifier =
            "const re = /a{1,3}b/;\ntest(\"holds\", () => {\n  expect(1).toBe(1);\n});\n";
        let symbols = parse("a.test.ts", with_quantifier).expect("a quantifier is not a block");
        assert!(symbols.iter().any(|s| s.qualified_name == "holds"));
    }

    #[trace("TC-961", "FR-051-AC-18")]
    // the widened grammar's edges, pinned one by one (CR-090).
    //
    // CR-084 loosened two boundaries the old regex held — exactly one
    // `\.\w+` modifier and no whitespace before `(` — and no test said where
    // the new boundary sits, so it could drift again unnoticed. Each case
    // here is one edge: what the grammar now admits, and what stays outside.
    #[test]
    fn tc961_the_widened_grammar_edges_are_pinned() {
        let registers = |line: &str| {
            parse("a.test.ts", &format!("{line}\n"))
                .expect("parses")
                .iter()
                .any(|s| s.kind == SymbolKind::TestFunction)
        };

        // Admitted: whitespace between the identifier/modifier chain and `(`.
        assert!(registers("test ('spaced call', () => {});"));
        assert!(registers("it.skip ('spaced after a modifier', () => {});"));
        // Admitted: whitespace between curried groups.
        assert!(registers(
            "it.skipIf(cond) ('spaced curried group', () => {});"
        ));
        // Admitted: an unbounded `.modifier` chain — the old regex took one.
        assert!(registers("it.a.b.c.d('a deep chain', () => {});"));

        // Admitted (PLAT-882, a delta from the pre-port scanner — see this
        // module's own docs): whitespace before the `.` no longer splits the
        // chain off `it`. Real TypeScript treats `it .skip(...)` and
        // `it.skip(...)` identically — whitespace around a member-access `.`
        // has no semantic meaning — and a `member_expression` node reads the
        // same either way; the old regex's rejection here was a textual
        // scanner artifact (it required the modifier-chain `.` to follow the
        // callee identifier with no intervening whitespace check applied
        // *before* the dot, only after it), not a deliberate exclusion.
        // Every positive fixture in this adapter's own corpus is
        // unspaced before its dots (prettier/eslint would reformat this
        // shape on save), so this delta is expected to recover nothing real.
        assert!(registers(
            "it .skip('now admitted, not split off', () => {});"
        ));
        // Outside: an empty modifier name is not valid TypeScript at all —
        // `it.(` has no property between the dot and the paren, which is a
        // genuine syntax error (not merely a non-matching shape), and this
        // adapter's own diagnostic contract (like the Rust adapter's) fails
        // loudly on one rather than silently finding no registration.
        assert!(parse("a.test.ts", "it.('empty modifier', () => {});\n").is_err());
        // Outside: an identifier continuing past `test`/`it`.
        assert!(!registers("test2('not a registration', () => {});"));
        // Outside: `await` glued to the callee is one identifier, not a form.
        assert!(!registers("awaitit('not awaited', () => {});"));
    }

    #[trace("TC-1039", "FR-051-AC-21")]
    // a suite registration is a CONTAINER that (CR-119)
    // parents its tests without renaming them.
    //
    // `describe(…)` registered nothing at all, so a trace tag on a suite header
    // had no symbol to attach to and the nearest enclosing symbol was the
    // file's own module — a container spanning line 1 to EOF, which names no
    // locus a reader can act on.
    #[test]
    fn tc1039_a_suite_is_a_container_that_parents_its_tests() {
        let source = concat!(
            "// TC-001: FR-001-AC-1 — the tag is on the BLOCK header.\n",
            "describe(\"warning default\", () => {\n",
            "  it(\"defaults every finding to warning\", () => {\n",
            "    expect(1 + 1).toBe(2);\n",
            "  });\n",
            "\n",
            "  suite(\"a nested suite under another harness's spelling\", () => {\n",
            "    test(\"still named by its own title\", () => {});\n",
            "  });\n",
            "\n",
            "  class Helper {\n",
            "    ready(): boolean { return true; }\n",
            "  }\n",
            "});\n",
            "\n",
            "it(\"a sibling outside every suite\", () => {});\n",
        );
        let symbols = parse("src/coverage.test.ts", source).expect("a valid file must parse");
        let by_name = |name: &str| {
            symbols
                .iter()
                .find(|s| s.qualified_name == name)
                .unwrap_or_else(|| panic!("`{name}` must be a symbol: {symbols:#?}"))
        };

        // The suite is a container, not evidence. `TestFunction` would make the
        // suite verify things in its own right, which is what CR-061 refused.
        let suite = by_name("warning default");
        assert_eq!(suite.kind, SymbolKind::Container);
        assert_eq!(suite.container.as_deref(), Some("src/coverage.test"));

        // Its span reaches back over the leading comment — that is the whole
        // point of registering it, since the tag sits there.
        assert_eq!(suite.leading_line, 1);
        assert_eq!(suite.line, 2);
        assert_eq!(suite.end_line, 14);

        // Members are PARENTED to the suite and NOT renamed by it: a
        // registration's qualified name is its own registered title (FR-051),
        // so `warning default.defaults every finding to warning` would change
        // the identity of every test in the ecosystem that sits in a suite.
        let inner = by_name("defaults every finding to warning");
        assert_eq!(inner.kind, SymbolKind::TestFunction);
        assert_eq!(inner.container.as_deref(), Some("warning default"));

        // Nesting composes, in both directions.
        let nested = by_name("a nested suite under another harness's spelling");
        assert_eq!(nested.kind, SymbolKind::Container);
        assert_eq!(nested.container.as_deref(), Some("warning default"));
        assert_eq!(
            by_name("still named by its own title").container.as_deref(),
            Some("a nested suite under another harness's spelling"),
        );

        // A class inside a suite still qualifies its own members, because a
        // class names them and a suite does not.
        assert_eq!(by_name("Helper").kind, SymbolKind::Container);
        assert_eq!(by_name("Helper.ready").kind, SymbolKind::Function);

        // And the scope closes with its braces.
        assert_eq!(
            by_name("a sibling outside every suite")
                .container
                .as_deref(),
            Some("src/coverage.test"),
        );
    }

    #[trace("TC-1039", "FR-051-AC-21")]
    // `context` is outside the suite grammar, and (CR-119)
    // the reason is measured rather than stylistic: across `~/dev` there are
    // 1,699 `describe(` registrations with a quoted title in 103 repository
    // roots, ZERO `suite(` and ZERO `context(` — while five lines begin
    // `context.`, method calls on an object that happens to carry the name.
    // Admitting it would put a grammar that reads a `.modifier` chain in front
    // of the one shape it cannot tell from a suite, for no occurrence it would
    // recover.
    #[test]
    fn tc1039_context_is_not_a_suite_and_a_lookalike_registers_nothing() {
        let containers = |source: &str| -> Vec<String> {
            parse("a.test.ts", source)
                .expect("parses")
                .into_iter()
                .filter(|s| s.kind == SymbolKind::Container && s.qualified_name != "a.test")
                .map(|s| s.qualified_name)
                .collect()
        };

        assert!(
            containers("context(\"mocha's tdd alias\", () => {});\n").is_empty(),
            "`context(` is not a declared suite name",
        );
        assert!(containers("context.setTransform(dpr, 0, 0, dpr, 0, 0);\n").is_empty());
        assert!(containers("await context.client.putSettings(key, newData);\n").is_empty());

        assert!(containers("describeAll(\"not a registration\", () => {});\n").is_empty());
        assert!(containers("describe(title, () => {});\n").is_empty());

        assert_eq!(
            containers("describe.each([1, 2])(\"a parametrised suite %i\", () => {});\n"),
            vec!["a parametrised suite %i".to_string()],
        );
    }

    #[trace("TC-1039", "FR-051-AC-21")]
    // a `describe(...)` whose arrow body's `{` falls on a (CR-119)
    // later line than the call still parents its members (PLAT-882 review
    // finding F6). Every positive `tc1039` fixture until now put the `{` on
    // the call's own opening line; this pins the tree-walk's structural
    // parenting — reading `describe`'s own `arguments`/`body` fields — does
    // not depend on that layout, unlike a line-structural scanner would.
    #[test]
    fn tc1039_a_late_brace_describe_still_parents_its_members() {
        let source = concat!(
            "describe(\n",
            "  \"brace on a later line\",\n",
            "  () =>\n",
            "  {\n",
            "    it(\"still parented\", () => {});\n",
            "  }\n",
            ");\n",
        );
        let symbols = parse("a.test.ts", source).expect("a valid file must parse");
        let inner = symbols
            .iter()
            .find(|s| s.qualified_name == "still parented")
            .expect("the nested registration is a symbol");
        assert_eq!(
            inner.container.as_deref(),
            Some("brace on a later line"),
            "{symbols:#?}"
        );
    }

    /// FR-051-AC-1 (PLAT-882 port audit item): `impl`-equivalent structure —
    /// a class's methods qualify under it, and the class itself mints a
    /// container, not two symbols for one declaration.
    #[test]
    fn a_class_qualifies_its_methods_and_a_suite_does_not() {
        let source = concat!(
            "class Foo {\n",
            "  bar() { return 1; }\n",
            "  static baz() {}\n",
            "}\n",
        );
        let symbols = parse("a.ts", source).expect("valid");
        assert!(symbols
            .iter()
            .any(|s| s.qualified_name == "Foo" && s.kind == SymbolKind::Container));
        assert!(symbols
            .iter()
            .any(|s| s.qualified_name == "Foo.bar" && s.kind == SymbolKind::Function));
        assert!(symbols
            .iter()
            .any(|s| s.qualified_name == "Foo.baz" && s.kind == SymbolKind::Function));
    }

    /// PLAT-882 port audit item: a `constructor` method mints no symbol,
    /// preserved from the old scanner's `RESERVED` denylist — the one entry
    /// of it that still applies once real declaration structure replaces
    /// line matching (every other entry existed only to keep control-flow
    /// keywords like `if`/`for`/`while` from being misread as method names,
    /// which a real `method_definition` node cannot be confused with).
    #[test]
    fn a_constructor_mints_no_symbol() {
        let source = concat!(
            "class Foo {\n",
            "  constructor(x: number) {\n",
            "    this.x = x;\n",
            "  }\n",
            "  bar() { return 1; }\n",
            "}\n",
        );
        let symbols = parse("a.ts", source).expect("valid");
        assert!(
            symbols
                .iter()
                .all(|s| s.qualified_name != "Foo.constructor"),
            "a constructor must mint no symbol: {symbols:?}"
        );
        assert!(symbols.iter().any(|s| s.qualified_name == "Foo.bar"));
    }

    /// PLAT-882 port audit item ("free" recovery, see this module's own
    /// docs): a `get`/`set` accessor method now mints a symbol, where the
    /// old regex's `NAME(...) {` shape never matched `get NAME() {}`.
    #[test]
    fn accessor_methods_mint_symbols() {
        let source = concat!(
            "class Foo {\n",
            "  get value(): number { return 1; }\n",
            "  set value(v: number) { this.stored = v; }\n",
            "}\n",
        );
        let symbols = parse("a.ts", source).expect("valid");
        assert!(
            symbols.iter().any(|s| s.qualified_name == "Foo.value"),
            "an accessor method must mint a symbol: {symbols:?}"
        );
    }

    /// PLAT-882 port audit item: a computed, string, numeric or private
    /// method name mints no symbol — the old regex's identifier-class match
    /// never recognised any of these shapes either.
    #[test]
    fn non_identifier_method_names_mint_no_symbol() {
        let source = concat!(
            "class Foo {\n",
            "  [computedName]() { return 1; }\n",
            "  \"a string name\"() { return 2; }\n",
            "  123() { return 3; }\n",
            "  #privateName() { return 4; }\n",
            "  ok() { return 5; }\n",
            "}\n",
        );
        let symbols = parse("a.ts", source).expect("valid");
        let names: Vec<&str> = symbols
            .iter()
            .filter(|s| s.container.as_deref() == Some("Foo"))
            .map(|s| s.qualified_name.as_str())
            .collect();
        assert_eq!(
            names,
            vec!["Foo.ok"],
            "only the plain identifier name mints: {names:?}"
        );
    }

    /// PLAT-882 port audit item: a leading comment or decorator is found
    /// through the `export` keyword — the comment/decorator is a sibling of
    /// the `export_statement` wrapping the declaration, not of the
    /// declaration node itself (see [`stmt_anchor`]'s own docs).
    #[test]
    fn export_wrapped_declarations_still_find_their_leading_annotation() {
        let class_source = concat!(
            "// TC-1 a leading comment before an exported class\n",
            "export class Foo {}\n",
        );
        let symbols = parse("a.ts", class_source).expect("valid");
        let foo = symbols
            .iter()
            .find(|s| s.qualified_name == "Foo")
            .expect("Foo is a symbol");
        assert_eq!(
            foo.leading_line, 1,
            "the comment before `export` must be found"
        );

        let const_source = concat!(
            "// TC-2 a leading comment before an exported const arrow\n",
            "export const f = (x: number) => x;\n",
        );
        let symbols = parse("a.ts", const_source).expect("valid");
        let f = symbols
            .iter()
            .find(|s| s.qualified_name == "f")
            .expect("f is a symbol");
        assert_eq!(f.leading_line, 1);

        let decorator_source = concat!("@Component({})\n", "export class Bar {}\n",);
        let symbols = parse("a.ts", decorator_source).expect("valid");
        let bar = symbols
            .iter()
            .find(|s| s.qualified_name == "Bar")
            .expect("Bar is a symbol");
        assert_eq!(
            bar.leading_line, 1,
            "the decorator before `export` must be found"
        );
    }

    /// PLAT-882 port audit item: only a *parenthesized* arrow function
    /// assigned via `const`/`let`/`var` is a function symbol — the bare
    /// single-parameter form is outside the old regex's own `\(`
    /// requirement, and a class field's arrow value is outside its
    /// `const`/`let`/`var`-at-line-start requirement.
    #[test]
    fn only_parenthesized_const_arrows_mint_function_symbols() {
        let source = concat!(
            "const paren = (x: number) => x;\n",
            "const bare = x => x;\n",
            "class Foo {\n",
            "  readonly ready = () => true;\n",
            "}\n",
        );
        let symbols = parse("a.ts", source).expect("valid");
        assert!(symbols.iter().any(|s| s.qualified_name == "paren"));
        assert!(
            symbols.iter().all(|s| s.qualified_name != "bare"),
            "the bare single-parameter form must not mint: {symbols:?}"
        );
        assert!(
            symbols
                .iter()
                .all(|s| s.qualified_name != "Foo.ready" && s.qualified_name != "ready"),
            "a class field's arrow value must not mint: {symbols:?}"
        );
    }

    /// TC-1924, FR-051-AC-14: a comment **trailing** on the same line as the
    /// statement before a declaration must not join that declaration's
    /// leading span — discovered authoring
    /// `tc803_one_reading_decides_whether_delimiters_are_code`'s restored
    /// span assertions (PLAT-882 PR #481 review finding F3): tree-sitter
    /// represents `const re = /../ ; // note` as two siblings (a
    /// declaration, then a `comment`), and without the fix in
    /// [`leading_span`], the nearest-preceding-sibling rule alone reads that
    /// trailing comment as the *next* declaration's own leading annotation
    /// — silently pulling a note that belongs to one statement into the
    /// span, and tag-binding search, of an unrelated one below it. The old
    /// line-structural scanner never had this failure mode: a line whose
    /// own trimmed text does not begin `//`/`/*`/`*` was never an
    /// annotation line at all, trailing or not.
    #[trace("TC-1924", "FR-051-AC-14")]
    #[test]
    fn tc1924_a_trailing_comment_does_not_leak_into_the_next_declarations_span() {
        let source = concat!(
            "const re = /a/; // a trailing note about `re`, not about `holds`\n",
            "test(\"holds\", () => {\n",
            "  expect(1).toBe(1);\n",
            "});\n",
        );
        let symbols = parse("a.test.ts", source).expect("a valid file must parse");
        let test_symbol = symbols
            .iter()
            .find(|s| s.qualified_name == "holds")
            .expect("the registration is a test symbol");
        assert_eq!(
            test_symbol.leading_line, 2,
            "a trailing comment on the PREVIOUS statement's line must not \
             become this declaration's own leading annotation: {test_symbol:?}"
        );

        // Control: the same comment written as its OWN standalone leading
        // line (not trailing on `re`'s line) still joins the span normally.
        let standalone = concat!(
            "const re = /a/;\n",
            "// a standalone leading comment, now genuinely `holds`'s own\n",
            "test(\"holds\", () => {\n",
            "  expect(1).toBe(1);\n",
            "});\n",
        );
        let symbols = parse("a.test.ts", standalone).expect("a valid file must parse");
        let test_symbol = symbols
            .iter()
            .find(|s| s.qualified_name == "holds")
            .expect("the registration is a test symbol");
        assert_eq!(test_symbol.leading_line, 2, "{test_symbol:?}");
    }

    /// TC-1926, FR-051-AC-14 (PLAT-882 review round 3, F4): a leading
    /// comment reaches an `await`-wrapped registration exactly as it
    /// reaches the non-awaited form — `stmt_anchor` used to climb only
    /// `expression_statement`/`export_statement`, stopping one node too low
    /// at the `await_expression` wrapping the call, so `leading_line`
    /// differed by one between `await it(...)` and `it(...)` with an
    /// identical leading comment. Zero real occurrences in either measured
    /// corpus, but the shape is legal (an async registration helper).
    #[trace("TC-1926", "FR-051-AC-14")]
    #[test]
    fn tc1926_a_leading_comment_reaches_an_awaited_registration() {
        let awaited = concat!(
            "// a tag\n",
            "await it(\"holds\", () => {\n",
            "  expect(1).toBe(1);\n",
            "});\n",
        );
        let plain = concat!(
            "// a tag\n",
            "it(\"holds\", () => {\n",
            "  expect(1).toBe(1);\n",
            "});\n",
        );
        for source in [awaited, plain] {
            let symbols = parse("a.test.ts", source).expect("a valid file must parse");
            let test_symbol = symbols
                .iter()
                .find(|s| s.qualified_name == "holds")
                .expect("the registration is a test symbol");
            assert_eq!(
                test_symbol.leading_line, 1,
                "a leading comment must reach the registration whether or not it is \
                 `await`-wrapped: {test_symbol:?}"
            );
        }
    }

    /// TC-1920, FR-051-AC-18 (CR-084, PLAT-882 review finding F1): a title
    /// held in a **multi-line** template literal registers nothing.
    /// `qualified_name` feeds `Symbol::compute_id`, and `plat843_audit_list`
    /// emits one row per source line, so an embedded newline would silently
    /// split one symbol across two rows of a differential — the pre-port
    /// scanner could never have produced one either, since it read a title
    /// from a single physical line.
    #[trace("TC-1920", "FR-051-AC-18")]
    #[test]
    fn tc1920_a_multiline_template_literal_title_registers_nothing() {
        let source = "it(`a title\nspanning lines`, () => {});\n";
        let symbols = parse("a.test.ts", source).expect("a valid file must parse");
        assert!(
            symbols.iter().all(|s| s.kind != SymbolKind::TestFunction),
            "a multi-line template literal title must register nothing: {symbols:?}"
        );
        assert!(
            symbols.iter().all(|s| !s.qualified_name.contains('\n')),
            "no qualified_name may contain a newline: {symbols:?}"
        );

        // Control: the same title on one line registers normally — the
        // exclusion is about the line span, not template literals in
        // general.
        let single_line = "it(`a single-line template title`, () => {});\n";
        let symbols = parse("a.test.ts", single_line).expect("a valid file must parse");
        assert!(symbols
            .iter()
            .any(|s| s.qualified_name == "a single-line template title"
                && s.kind == SymbolKind::TestFunction));
    }

    /// TC-1921, FR-051-AC-18 (PLAT-882 review finding F2, mutation E4): a
    /// registration whose title argument starts more than three physical
    /// lines after the call, but which still carries a second (callback)
    /// argument, must still register. This is the one shape that
    /// distinguishes "no title-lookahead window" (CR-180's own claim) from
    /// "a wider but still-bounded window" — a fixture with no callback
    /// argument at all (the adjacent negative fixture in
    /// `registration.test.ts`) cannot tell the two apart, because it fails
    /// under both for the unrelated, separately-enforced callback-argument
    /// reason.
    #[trace("TC-1921", "FR-051-AC-18")]
    #[test]
    fn tc1921_a_far_title_with_a_callback_still_registers() {
        let source = concat!(
            "it(\n",
            "\n",
            "\n",
            "\n",
            "\n",
            "  'a title four blank lines down, with a callback',\n",
            "  () => {},\n",
            ");\n",
        );
        let symbols = parse("a.test.ts", source).expect("a valid file must parse");
        assert!(
            symbols.iter().any(|s| s.qualified_name
                == "a title four blank lines down, with a callback"
                && s.kind == SymbolKind::TestFunction),
            "a far title with a real callback argument must register: {symbols:?}"
        );
    }

    /// TC-1922, FR-051-AC-1 (PLAT-882 review finding F2, mutation E5): a
    /// `const`/`let`/`var` declarator whose value is a *parenthesized
    /// non-arrow* expression — `const sum = (a + b);`, the actual 76-row
    /// false-positive class the differential names — must mint no symbol.
    /// `only_parenthesized_const_arrows_mint_function_symbols` above covers
    /// the bare-arrow and class-field-arrow exclusions but never this shape.
    #[trace("TC-1922", "FR-051-AC-1")]
    #[test]
    fn tc1922_a_parenthesized_non_arrow_value_mints_no_symbol() {
        let source = "const sum = (a + b);\n";
        let symbols = parse("a.ts", source).expect("valid");
        assert!(
            symbols.iter().all(|s| s.qualified_name != "sum"),
            "a parenthesized non-arrow expression must not mint: {symbols:?}"
        );
    }

    /// TC-1925, FR-051-AC-1 (PLAT-882 PR #481 review round 2, mutation
    /// E5-corrected): `mint_arrow_const_declarators` has **two** guards in
    /// sequence — `value.kind() != "arrow_function"`, then
    /// `value.child_by_field_name("parameters").is_none()`. The review's
    /// original mutation E5 (also accept a `parenthesized_expression` value
    /// in the first guard) does not kill `tc1922` above: `(a + b)` parses to
    /// `parenthesized_expression`, which carries no `parameters` field
    /// regardless of what it wraps, so the *second* guard rejects it either
    /// way — the two guards are redundant for that specific input, and
    /// mutating only the first is invisible to it.
    ///
    /// That does **not** make the first guard dead code: `function_expression`
    /// and `generator_function` values (`const f = function() {}`,
    /// `const g = function*() {}`) *do* carry a `parameters` field (checked
    /// directly, not assumed), so only the first guard's `arrow_function`
    /// check rejects them. This is the isolating fixture the original E5
    /// finding needed and did not have: a value.kind() that is non-arrow yet
    /// still clears the second guard, so only the mutated line's own check
    /// stands between it and minting. Confirmed as a real mutation kill
    /// (widening the first guard to also accept `function_expression` makes
    /// this test fail alone; `tc1922` stays green, since `parenthesized_expression`
    /// is untouched by that particular widening).
    #[trace("TC-1925", "FR-051-AC-1")]
    #[test]
    fn tc1925_a_non_arrow_function_expression_value_mints_no_symbol() {
        for source in [
            "const f = function() { return 1; };\n",
            "const g = function*() { yield 1; };\n",
        ] {
            let symbols = parse("a.ts", source).expect("valid");
            assert!(
                symbols
                    .iter()
                    .all(|s| s.qualified_name != "f" && s.qualified_name != "g"),
                "a non-arrow function-expression value must not mint, even though it carries \
                 a `parameters` field like an arrow function does: {symbols:?}"
            );
        }
    }

    /// TC-1923, FR-051-AC-1 (PLAT-882 review finding F2/F5, mutation E6): an
    /// `interface`'s `method_signature` member mints no symbol — the largest
    /// structural exclusion this adapter makes (the differential's 19-row
    /// interface-signature false-positive class), pinned here so it cannot
    /// silently regress. See this module's own docs for the rule stated in
    /// full.
    #[trace("TC-1923", "FR-051-AC-1")]
    #[test]
    fn tc1923_an_interface_method_signature_mints_no_symbol() {
        let source = concat!(
            "interface Foo {\n",
            "  bar(): void;\n",
            "  readonly baz: number;\n",
            "}\n",
        );
        let symbols = parse("a.ts", source).expect("valid");
        assert!(
            symbols
                .iter()
                .all(|s| s.qualified_name != "Foo.bar" && s.qualified_name != "bar"),
            "an interface method signature must not mint: {symbols:?}"
        );
        // Review round 3 (F5): the module doc claims an interface mints no
        // symbol "not the declaration itself and not its members" — the
        // assertion above covered only the member half. `walk`'s
        // `_ => walk(child, ...)` catch-all is what currently gives
        // `interface_declaration` this behaviour (it is not explicitly
        // matched, so it neither mints nor is excluded on purpose); adding
        // `"interface_declaration"` to the `class_declaration` arm, which
        // would make every interface mint a `Container` named `Foo`, passed
        // every other test in this module silently. This closes that gap.
        assert!(
            symbols.iter().all(|s| s.qualified_name != "Foo"),
            "an interface declaration itself must not mint a symbol: {symbols:?}"
        );
    }
}
