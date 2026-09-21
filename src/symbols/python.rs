//! Python source adapter (FR-051).
//!
//! Parses over a tree-sitter syntax tree, through the dependency-boundary
//! crate `quire-rust-extraction` (PLAT-843, widened to Python by PLAT-851) —
//! see that crate's own module docs for why the tree-sitter dependency
//! itself is never named here. This adapter still answers exactly the
//! questions FR-051 needs — which declaration owns a span, what its
//! qualified name and kind are, whether it is a test — and nothing more: no
//! build, no type resolution, no dependency installation, and no execution
//! of the extracted code (FR-051-CON-1).
//!
//! **Form matching is a seam this adapter does not cross.** Which declared
//! marker or legacy trace form appears inside a symbol's span is
//! `trace.rs`'s question, answered afterwards against the resolved span
//! (`Symbol::attached_source`); this file has no reference to trace forms,
//! regex, or `TraceabilityModel`, and the tree walk below must not gain one.
//!
//! ## The PLAT-14 trap this port exists to close
//!
//! Every Python method sits at **depth 2** — `class_definition` → `body`
//! (a `block`) → `function_definition` — never at the root or its direct
//! children. `quire-code-parse`'s own declaration-structure predicate
//! ([`quire_rust_extraction::parse_file`]'s docs) checks every nesting
//! depth for exactly this reason: a shallower predicate would make every
//! Python method structurally invisible, which is PLAT-14's original defect
//! (a hand-rolled scanner desyncing mid-file and returning zero symbols for
//! the rest of it, with nothing in the return type distinguishing that from
//! "this file declares nothing"). The rule this adapter relies on, unchanged
//! by anything here: **an error is body-local, and does not gate the whole
//! file, iff it lies within the declaration's own executable body node** —
//! so a broken statement inside one test function's body never costs its
//! *siblings*, at any nesting depth, their own symbols.
//!
//! Test classification is the pytest convention: a `test_`-prefixed
//! function, or a method of a `Test`-prefixed class whose own name is
//! `test_`-prefixed. Direct imported unittest.TestCase bases additionally
//! identify test classes without a naming convention (FR-051-AC-3, #407).
//!
//! ## What survives byte-for-byte (a compatibility surface, PLAT-868)
//!
//! `Symbol::compute_id` hashes `(language, path, qualified_name, kind)`
//! (`src/symbols/mod.rs`) — qualified-name construction is an identity
//! surface, not a style choice, and every quirk below is preserved exactly
//! rather than "cleaned up", the same discipline PLAT-843 documented for
//! Rust:
//!
//! - **Only a `class` is a scope.** A `def` never becomes a container for
//!   its own nested `def`s (unlike Rust, PLAT-845's own fix) — a function
//!   nested inside another function, or inside a method, qualifies under
//!   whatever *class* scope was already open (or bare, at module level),
//!   never under the enclosing function. This is the pre-port scanner's own
//!   behaviour, not a limitation this port introduces, and two nested
//!   helpers of the same name under the same class (or both bare at module
//!   level) still collide on identity exactly as they did before.
//! - **A qualified name never carries the module prefix.** Only class
//!   nesting joins with `.` (`Outer.Inner.method`); a module-level `def` or
//!   `class` is named bare. The *container* attribute is always
//!   `Some(...)` though — the enclosing class's qualified name, or the
//!   module name when there is none — matching `RawSymbol.container`'s
//!   meaning everywhere else (only the synthetic module symbol itself gets
//!   `container: None`).
//! - **Unittest `TestCase` base detection is bounded to a single-line,
//!   top-level class header** (#407): a base list that wraps across lines
//!   is never inferred, and neither is a nested class's base list, matching
//!   the pre-port scanner exactly — see [`is_unittest_class`] and the
//!   [`UnittestImports`] docs for why this adapter deliberately keeps that
//!   restriction rather than widening it (a widening is an identity change,
//!   PLAT-868's own non-negotiable rule, not a free fix).
//! - **Import/rebinding tracking stays module-top-level-only**, mirroring
//!   the pre-port scanner's `indent == 0` restriction: an import inside a
//!   function, an `if`, or a `try` block is invisible to the unittest
//!   classifier, matching FR-051-AC-3's own documented "local imports … not
//!   inferred" limitation.
//!
//! ## What changed, and why each delta is free
//!
//! Per PLAT-843's own cause-based rule — *free from parsing correctly → fix
//! here and name the delta; requires changing what a symbol **is** or how it
//! is **named** → a separate ticket* — these are all free (span attributes
//! are explicitly non-identity, `src/symbols/mod.rs`):
//!
//! - **A symbol embedded in a string is no longer readable as a
//!   declaration, structurally.** The pre-port scanner tracked triple-quoted
//!   string state by hand (`Quoting`/`scan_line`) and had three real defects
//!   in doing so (#274, pinned by the retired `tc1029`/`tc1030`/`tc1031`):
//!   a string opened mid-line was missed, a closing delimiter could be
//!   misread as an opener, and the scope stack could resume stale after an
//!   embedded string. tree-sitter's grammar makes the whole class
//!   structurally impossible — a `string` node's content is never a
//!   `class_definition`/`function_definition` node — so there is no longer
//!   a hand-rolled state machine to have a defect in. See `python_tests`
//!   below for this property's successors.
//! - **A multi-line (black-wrapped) decorator no longer truncates
//!   `leading_line`** (PLAT-234). The pre-port scanner walked physical
//!   *lines*, and a decorator argument list split across lines broke its
//!   contiguous-annotation-lines walk at the first continuation line, which
//!   does not itself start with `@`. [`leading_span`] walks preceding
//!   *sibling nodes* instead: `decorated_definition`'s own span already
//!   includes every decorator regardless of how any one of them wraps,
//!   tree-sitter having tokenized the whole call as one node.
//! - **A black-wrapped multi-line signature no longer truncates
//!   `end_line`** (CR-037). The pre-port scanner needed a bespoke
//!   parenthesis-depth walk ([`block_end`]/[`paren_delta`], now deleted) to
//!   avoid reading a signature's own dedented closing line as the end of the
//!   suite. `function_definition`/`class_definition`'s own node span already
//!   covers the full signature and body regardless of how the signature
//!   wraps, so no special case is needed.
//!
//! What did **not** change: the hand-written lexer subsystem this file used
//! to carry (`Quoting`, `scan_line`, `is_triple`, `single_quoted_end`,
//! `block_end`, `paren_delta`, `is_annotation`) is deleted outright as a
//! consequence of parsing correctly, not as a deliberate optimisation.
//! [`UnittestImports`] and its bounded, single-line text parsing
//! ([`declaration`]) are kept verbatim — see that struct's own docs for why
//! reusing text parsing there, rather than reading AST import fields, is the
//! safer identity-preserving choice for this port.

use std::collections::BTreeSet;

use quire_rust_extraction::tree_sitter::Node;
use quire_rust_extraction::{parse_file, Language};

use super::{RawSymbol, SymbolKind};

/// Parse `source` into raw symbols, or return a per-file reason to skip it.
///
/// **A declaration-structure failure fails loudly, naming the line** — the
/// same all-or-nothing per-file contract as before this port
/// (`Result<Vec<RawSymbol>, String>`, unchanged): `src/symbols/mod.rs`'s
/// `extend_with_file` pushes one [`super::SymbolDiagnostic`] on `Err` and
/// contributes zero symbols for that file (FR-051-AC-9, FR-051-CON-2). What
/// changed is the diagnostic itself: previously a scanner-specific reason
/// with no reliable location; now the tree-sitter diagnostic's own one-based
/// line and zero-based column, naming exactly where the declaration
/// structure — at *any* nesting depth, including a method two levels inside
/// a `class` — could not be trusted. See this module's own docs for why that
/// depth matters (the PLAT-14 trap).
pub(crate) fn parse(path: &str, source: &str) -> Result<Vec<RawSymbol>, String> {
    let parsed =
        parse_file(Language::Python, "<source>", source).map_err(|e| format!("parse: {e}"))?;
    if let Some(diagnostic) = parsed.diagnostic() {
        return Err(format!(
            "line {}: unresolvable declaration structure (column {})",
            diagnostic.line(),
            diagnostic.column()
        ));
    }

    let lines: Vec<&str> = source.lines().collect();
    let module = module_name(path);
    let mut out = vec![RawSymbol {
        qualified_name: module.clone(),
        kind: SymbolKind::Container,
        line: 1,
        leading_line: 1,
        end_line: lines.len().max(1),
        container: None,
    }];

    let mut walker = Walk::new(&lines, source, &module);
    walker.walk(
        parsed.root_node(),
        AtModuleLevel(true),
        DirectInClassBody(false),
    );
    out.extend(walker.out);
    Ok(out)
}

/// A class currently open on the walk: its own qualified name (what a member
/// qualifies under), and its test-class flavour.
struct ClassScope {
    qualified_name: String,
    test_class: TestClass,
}

/// A class scope's flavour, decided once when the class is entered.
enum TestClass {
    /// Neither convention matched: members are never test evidence by
    /// virtue of this class alone.
    None,
    /// `Test`-prefixed name (pytest convention): every `test_`-prefixed
    /// member is evidence, at any nesting depth inside the class.
    Pytest,
    /// A single-line, top-level class header with a direct `unittest`
    /// `TestCase` base (#407): only *direct* members of the class's own
    /// body are evidence — a `test_`-prefixed helper nested one level
    /// deeper (inside an `if`, inside a method) is not.
    Unittest,
}

/// Whether the walk is iterating the module node's own direct children —
/// never true for anything found by recursing into a body. A newtype, not a
/// bare `bool`: this sits in every call next to [`DirectInClassBody`], and
/// two adjacent same-typed positional bools compile silently transposed. A
/// transposition here would change which member reads as a container's
/// *direct* one — feeding `TestClass::Unittest`'s classification, which
/// feeds `kind`, which is an identity attribute (`Symbol::compute_id`), not
/// a diagnostic — so the type system is asked to rule it out rather than a
/// reviewer.
#[derive(Clone, Copy, PartialEq, Eq)]
struct AtModuleLevel(bool);

/// Whether the walk is iterating a class's own `body` block's direct
/// children — gates whether a member counts as *directly* declared for
/// [`TestClass::Unittest`] purposes (see that variant's own docs). See
/// [`AtModuleLevel`] for why this is a newtype rather than a second bare
/// `bool`.
#[derive(Clone, Copy, PartialEq, Eq)]
struct DirectInClassBody(bool);

/// The walk's threaded state for one file: the per-file inputs that never
/// change (`lines`/`source`/`module`) and the accumulators that do
/// (`class_stack`/`unittest`/`out`), held once on `self` instead of passed
/// as up to eight loose parameters through every recursive call — which is
/// what the two `#[allow(clippy::too_many_arguments)]` this replaces used to
/// cover.
struct Walk<'a> {
    lines: &'a [&'a str],
    source: &'a str,
    module: &'a str,
    class_stack: Vec<ClassScope>,
    unittest: UnittestImports,
    out: Vec<RawSymbol>,
}

impl<'a> Walk<'a> {
    fn new(lines: &'a [&'a str], source: &'a str, module: &'a str) -> Self {
        Walk {
            lines,
            source,
            module,
            class_stack: Vec::new(),
            unittest: UnittestImports::default(),
            out: Vec::new(),
        }
    }

    /// Feed one physical line to [`UnittestImports::observe_binding`] — the
    /// bounded, module-top-level-only rebinding tracking this adapter keeps
    /// text-based (see that struct's own docs).
    fn observe_line(&mut self, row: usize) {
        if let Some(line) = self.lines.get(row) {
            self.unittest.observe_binding(line);
        }
    }

    /// Walk `node`'s named children, minting a [`RawSymbol`] for each
    /// declaration and recursing to find every nested one — a `class`/`def`
    /// body, and anywhere else a declaration can legally appear (an `if`, a
    /// `try`, a `with`, a `for`), the same set the pre-port line-structural
    /// scanner saw regardless of statement context.
    ///
    /// `at_module_level` is true only while iterating the module node's own
    /// direct children — never for anything found by recursing into a body —
    /// matching the pre-port scanner's `indent == 0` restriction exactly: it
    /// gates both the unittest import/rebinding tracking and top-level
    /// `TestCase`-base detection. `direct_in_class_body` is true only while
    /// iterating a class's own `body` block's direct children — the AST
    /// equivalent of the pre-port scanner's recorded body-indent comparison,
    /// exact because Python requires uniform indentation within one suite.
    fn walk(
        &mut self,
        node: Node,
        at_module_level: AtModuleLevel,
        direct_in_class_body: DirectInClassBody,
    ) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "decorated_definition" => {
                    if at_module_level.0 {
                        for decorator in decorators_of(child) {
                            self.observe_line(decorator.start_position().row);
                        }
                    }
                    let Some(inner) = child.child_by_field_name("definition") else {
                        continue;
                    };
                    self.dispatch_definition(inner, child, at_module_level, direct_in_class_body);
                }
                "class_definition" | "function_definition" => {
                    self.dispatch_definition(child, child, at_module_level, direct_in_class_body);
                }
                _ => {
                    if at_module_level.0 {
                        self.observe_line(child.start_position().row);
                    }
                    self.walk(child, AtModuleLevel(false), DirectInClassBody(false));
                }
            }
        }
    }

    /// Mint a [`RawSymbol`] for a `class_definition` or `function_definition`
    /// (`def_node`), attributing its leading annotation span to `span_node` —
    /// the surrounding `decorated_definition` when decorated, `def_node`
    /// itself otherwise — and recurse into its body with the
    /// container/scope rules this module's own docs pin.
    fn dispatch_definition(
        &mut self,
        def_node: Node,
        span_node: Node,
        at_module_level: AtModuleLevel,
        direct_in_class_body: DirectInClassBody,
    ) {
        let Some(name) = field_text(def_node, "name", self.source) else {
            return;
        };
        let (container, qualified_name) = qualify(&self.class_stack, self.module, &name);

        match def_node.kind() {
            "class_definition" => {
                let header_row = def_node.start_position().row;
                let header_line = self.lines.get(header_row).copied().unwrap_or("").trim();
                let is_unittest = at_module_level.0 && self.unittest.is_test_case(header_line);
                if at_module_level.0 {
                    self.observe_line(header_row);
                }
                let test_class = if is_unittest {
                    TestClass::Unittest
                } else if name.starts_with("Test") {
                    TestClass::Pytest
                } else {
                    TestClass::None
                };
                self.out.push(RawSymbol {
                    qualified_name: qualified_name.clone(),
                    kind: SymbolKind::Container,
                    line: def_node.start_position().row + 1,
                    leading_line: leading_span(span_node, self.lines),
                    end_line: def_node.end_position().row + 1,
                    container: Some(container),
                });
                self.class_stack.push(ClassScope {
                    qualified_name,
                    test_class,
                });
                if let Some(body) = def_node.child_by_field_name("body") {
                    self.walk(body, AtModuleLevel(false), DirectInClassBody(true));
                }
                self.class_stack.pop();
            }
            "function_definition" => {
                if at_module_level.0 {
                    self.observe_line(def_node.start_position().row);
                }
                let in_test_class =
                    self.class_stack
                        .last()
                        .is_some_and(|scope| match scope.test_class {
                            TestClass::Pytest => true,
                            TestClass::Unittest => direct_in_class_body.0,
                            TestClass::None => false,
                        });
                let kind = if name.starts_with("test_")
                    && (self.class_stack.is_empty() || in_test_class)
                {
                    SymbolKind::TestFunction
                } else {
                    SymbolKind::Function
                };
                self.out.push(RawSymbol {
                    qualified_name,
                    kind,
                    line: def_node.start_position().row + 1,
                    leading_line: leading_span(span_node, self.lines),
                    end_line: def_node.end_position().row + 1,
                    container: Some(container),
                });
                // A `def` is never a container for its own nested `def`s (see
                // this module's own docs) — recurse with the *same* class
                // scope, not this function's own name, so a nested helper
                // qualifies under whatever class (or bare module level) was
                // already open, matching the pre-port scanner exactly.
                if let Some(body) = def_node.child_by_field_name("body") {
                    self.walk(body, AtModuleLevel(false), DirectInClassBody(false));
                }
            }
            _ => {}
        }
    }
}

/// The `decorator` children of a `decorated_definition` node, in source
/// order — not a named field (only `definition` is), so read as plain
/// children filtered by kind.
fn decorators_of(decorated: Node) -> Vec<Node> {
    let mut cursor = decorated.walk();
    decorated
        .named_children(&mut cursor)
        .filter(|n| n.kind() == "decorator")
        .collect()
}

/// `(container, qualified_name)` for a declaration named `name`, given the
/// currently open class scope: `Class.name` under the innermost open class,
/// or a bare `name` at module level — the module is never a name prefix,
/// only ever the *container* attribute when there is no class scope.
fn qualify(class_stack: &[ClassScope], module: &str, name: &str) -> (String, String) {
    match class_stack.last() {
        Some(scope) => (
            scope.qualified_name.clone(),
            format!("{}.{name}", scope.qualified_name),
        ),
        None => (module.to_string(), name.to_string()),
    }
}

/// `node`'s field named `name` as plain source text, or `None` when the
/// field is absent (the grammar guarantees it is present for
/// `class_definition`/`function_definition`; kept as a guard rather than a
/// panic, matching this crate's no-panic-in-production-code convention).
fn field_text(node: Node, field: &str, source: &str) -> Option<String> {
    node.child_by_field_name(field)
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .map(str::to_string)
}

/// The 1-based first line of `node`'s leading annotation block: `node`
/// itself already includes every decorator when it is a
/// `decorated_definition` (decorators are its own children, never its
/// siblings, in tree-sitter-python's grammar), so this only needs to walk
/// further back through contiguous preceding `comment` siblings — the same
/// two stopping conditions the pre-port scanner used (a non-annotation line,
/// or a blank-line gap), now checked between sibling nodes instead of
/// between lines, which is what fixes PLAT-234: a multi-line decorator
/// argument list is part of one `decorated_definition` node regardless of
/// how many lines it spans.
///
/// **A comment only extends the span when it starts its own line.**
/// tree-sitter emits a *trailing* comment (`x = 1  # note`) as a sibling
/// node exactly like a comment on its own line — `.kind() == "comment"`
/// alone cannot tell them apart, and the pre-port scanner's own
/// `is_annotation` required the *trimmed line* to start with `@`/`#`, which
/// a trailing comment never does. Without this check, `# note` above would
/// have been walked into the *next* declaration's leading span, pulling
/// whatever the comment says (a stray trace-shaped tag, a note about a
/// different function) into a span `trace.rs` binds from — a review finding
/// (PLAT-868 PR #479): a correct-looking span change that silently mints or
/// moves a binding. `lines` is used only for this same-line check, never to
/// re-derive what the tree already gives structurally.
///
/// A comment that stays indented at the *previous* declaration's own body
/// level, before the dedent back out of it, is never even a candidate here:
/// it is tree-sitter's own "extra"-token placement, not this function's own
/// stopping conditions, that attaches it as a trailing child of the
/// previous `block` rather than as a sibling of the next declaration — so
/// it cannot walk into the next span at all (measured directly; see
/// `an_indented_trailing_comment_stays_inside_the_previous_body_not_the_next_span`
/// in this module's own tests). Only a comment already dedented to the next
/// declaration's own level is its sibling, and reaches the walk below.
fn leading_span(node: Node, lines: &[&str]) -> usize {
    let mut boundary_row = node.start_position().row;
    let mut current = node;
    while let Some(prev) = current.prev_sibling() {
        if prev.kind() != "comment" {
            break;
        }
        if boundary_row.saturating_sub(prev.end_position().row) > 1 {
            break;
        }
        let starts_its_own_line = lines
            .get(prev.start_position().row)
            .and_then(|line| line.get(..prev.start_position().column))
            .is_some_and(|prefix| prefix.trim().is_empty());
        if !starts_its_own_line {
            break;
        }
        boundary_row = prev.start_position().row;
        current = prev;
    }
    boundary_row + 1
}

/// A `def`/`async def`/`class` declaration and whether it is a class — a
/// bounded, single-line text match used only inside [`UnittestImports`]'s
/// own rebinding tracking (a top-level `class`/`def` line shadows a
/// same-named import), never to locate a real declaration: that is the
/// tree's job everywhere else in this module.
fn declaration(trimmed: &str) -> Option<(String, bool)> {
    let (rest, is_class) = if let Some(r) = trimmed.strip_prefix("class ") {
        (r, true)
    } else if let Some(r) = trimmed.strip_prefix("def ") {
        (r, false)
    } else {
        (trimmed.strip_prefix("async def ")?, false)
    };
    let name: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some((name, is_class))
}

/// Bounded lexical identities, not dependency/type resolution. Only
/// preceding module-level imports authorize a unittest base; a lookalike
/// spelling or an explicitly rebound import must not create evidence (#407).
///
/// Kept as bounded, single-line text parsing over the reconstructed source
/// line — verbatim from the pre-port scanner — rather than rebuilt to read
/// AST import fields directly. `quire-code-parse` would make wrapped,
/// multi-line imports and bases trivial to recognize correctly, which is
/// tempting, but FR-051-AC-3's own CR note is explicit that "multi-line
/// import/base declarations are not inferred" — a bounded, *documented*
/// limitation of the shipped classifier, not an incidental gap this port is
/// free to close. Recognizing them now would change which methods classify
/// as [`SymbolKind::TestFunction`] versus [`SymbolKind::Function`] for any
/// corpus file using a wrapped import or base list — a `kind` change is an
/// identity change (`Symbol::compute_id` hashes it), which PLAT-868's own
/// non-negotiable rule reserves for a separate, deliberately-scoped ticket,
/// not a side effect of porting the structural walk to tree-sitter.
#[derive(Default)]
struct UnittestImports {
    modules: BTreeSet<String>,
    test_cases: BTreeSet<String>,
}

enum ImportKind {
    Modules,
    UnittestNames,
    OtherNames,
}

impl UnittestImports {
    fn forget(&mut self, name: &str) {
        self.modules.remove(name);
        self.test_cases.remove(name);
    }

    fn observe_binding(&mut self, line: &str) {
        let line = line.split('#').next().unwrap_or_default().trim();
        let from_import = line
            .strip_prefix("from ")
            .and_then(|rest| rest.split_once(" import "));
        let (imports, kind) = if let Some(imports) = line.strip_prefix("import ") {
            (imports, ImportKind::Modules)
        } else if let Some((module, imports)) = from_import {
            (
                imports,
                if module == "unittest" {
                    ImportKind::UnittestNames
                } else {
                    ImportKind::OtherNames
                },
            )
        } else {
            // An explicit assignment or a same-name declaration removes the
            // identity. More elaborate runtime rebinding is outside this
            // syntax-only adapter's declared import forms.
            if let Some((name, _)) = declaration(line) {
                self.forget(&name);
            } else if let Some((left, _)) = line.split_once('=') {
                let name = left.split(':').next().unwrap_or_default().trim();
                self.forget(name);
            } else if let Some(name) = line.strip_prefix("del ") {
                self.forget(name.trim());
            }
            return;
        };
        for import in imports.split(',') {
            let words: Vec<&str> = import.split_whitespace().collect();
            let (source, binding, module_identity) = match words.as_slice() {
                [source] => {
                    // `import helpers.sub` replaces `helpers`, not a binding
                    // called `helpers.sub`. Likewise `import unittest.mock`
                    // binds the genuine unittest root, not its submodule.
                    let root = source.split('.').next().unwrap_or(source);
                    (*source, root, root)
                }
                // An alias binds the full imported module: aliasing
                // unittest.mock does not authorize alias.TestCase.
                [source, "as", binding] => (*source, *binding, *source),
                _ => continue,
            };
            self.forget(binding);
            match kind {
                ImportKind::UnittestNames if source == "TestCase" => {
                    self.test_cases.insert(binding.to_string());
                }
                ImportKind::Modules if module_identity == "unittest" => {
                    self.modules.insert(binding.to_string());
                }
                _ => {}
            }
        }
    }

    fn is_test_case(&self, declaration: &str) -> bool {
        // The base list belongs to the header, not to a comment or a
        // same-line class suite. Those can mention TestCase as ordinary data.
        let header = declaration.split(['#', ':']).next().unwrap_or_default();
        let Some((_, after_open)) = header.split_once('(') else {
            return false;
        };
        let Some((bases, _)) = after_open.split_once(')') else {
            return false;
        };
        bases.split(',').map(str::trim).any(|base| {
            self.test_cases.contains(base)
                || base
                    .strip_suffix(".TestCase")
                    .is_some_and(|module| self.modules.contains(module))
        })
    }
}

/// Dotted module name from a repo-relative path: `pkg/mod.py` → `pkg.mod`,
/// `pkg/__init__.py` → `pkg`.
fn module_name(path: &str) -> String {
    let stem = path.strip_suffix(".py").unwrap_or(path);
    let stem = stem.strip_suffix("/__init__").unwrap_or(stem);
    stem.replace('/', ".")
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::*;

    /// Every declaration the adapter minted, qualified name first.
    fn names(symbols: &[RawSymbol]) -> Vec<&str> {
        symbols
            .iter()
            .map(|s| s.qualified_name.as_str())
            .collect::<Vec<_>>()
    }

    /// The one symbol with this qualified name, or a panic naming what was
    /// minted instead — an absent symbol is the failure this module is about.
    fn symbol<'s>(symbols: &'s [RawSymbol], qualified: &str) -> &'s RawSymbol {
        symbols
            .iter()
            .find(|s| s.qualified_name == qualified)
            .unwrap_or_else(|| panic!("no symbol {qualified}; minted {:?}", names(symbols)))
    }

    /// A minimal stand-in for the pre-port scanner's own naive approach —
    /// substring/prefix matching with no notion of a string literal at all —
    /// used only to demonstrate that the property the retired
    /// `tc1029`/`tc1030`/`tc1031` pinned is not trivially true of *any*
    /// implementation: a scanner that just looks for lines shaped like a
    /// declaration finds the phantom ones embedded in a string. `parse`
    /// itself must not, because tree-sitter never represents a string's
    /// content as a `class_definition`/`function_definition` node.
    fn naive_line_scan_finds_declaration(source: &str, name_fragment: &str) -> bool {
        source.lines().any(|line| {
            (line.contains("class ") || line.contains("def ")) && line.contains(name_fragment)
        })
    }

    /// TC-1029 successor (PLAT-868, retires `tc1029_a_triple_quote_is_tracked_wherever_it_opens`):
    /// a declaration embedded inside a triple-quoted string, in any
    /// delimiter, under any prefix, or opened-and-closed on one line, is
    /// **structurally** never read as a symbol — there is no longer a
    /// hand-rolled quoting state machine (`Quoting`/`scan_line`, deleted by
    /// this port) that could resume incorrectly, because a string's content
    /// is never a declaration node to begin with. Confirmed red first: the
    /// `naive_line_scan_finds_declaration` stand-in above — the shape of bug
    /// the retired tests pinned — reports every phantom present, proving the
    /// property is not vacuous; `parse` reports none of them.
    #[trace("TC-1029", "FR-051-AC-20")]
    #[test]
    fn tc1029_a_string_embedded_declaration_is_structurally_unreadable() {
        let source = concat!(
            "FIXTURE = \"\"\"\n",
            "class Phantom:\n",
            "    def method(self):\n",
            "        pass\n",
            "\"\"\"\n",
            "\n",
            "RAW = r'''\n",
            "def raw_phantom():\n",
            "    pass\n",
            "'''\n",
            "\n",
            "TEMPLATE = f\"\"\"\n",
            "def f_phantom():\n",
            "    pass\n",
            "\"\"\"\n",
            "\n",
            "BYTES = rb\"\"\"\n",
            "def rb_phantom():\n",
            "    pass\n",
            "\"\"\"\n",
            "\n",
            "ONE_LINE = \"\"\"def inline_phantom(): pass\"\"\"\n",
            "\n",
            "def test_after_every_string():\n",
            "    assert True\n",
        );
        let symbols = parse("t_test.py", source).expect("parses");

        for phantom in [
            "Phantom",
            "method",
            "raw_phantom",
            "f_phantom",
            "rb_phantom",
            "inline_phantom",
        ] {
            assert!(
                naive_line_scan_finds_declaration(source, phantom),
                "fixture premise: a naive scanner must see {phantom} as a declaration \
                 for this to be a meaningful property test",
            );
            assert!(
                !names(&symbols).iter().any(|n| n.contains(phantom)),
                "{phantom} is string content, not a declaration: {:?}",
                names(&symbols)
            );
        }
        assert_eq!(
            symbol(&symbols, "test_after_every_string").kind,
            SymbolKind::TestFunction
        );
    }

    /// TC-1030 successor (PLAT-868, retires
    /// `tc1030_a_delimiter_in_a_string_or_a_comment_does_not_toggle`): a
    /// triple-quote delimiter inside a single-quoted string, after a `#`, or
    /// escaped, and a `#` inside a triple-quoted string, never affect what
    /// the tree parses as a declaration — there is no scan state left to
    /// toggle incorrectly, because tree-sitter's own lexer (not this
    /// adapter's) resolves string and comment boundaries once, correctly, as
    /// part of producing the tree at all.
    #[trace("TC-1030", "FR-051-AC-20")]
    #[test]
    fn tc1030_a_delimiter_in_a_string_or_comment_never_affects_the_tree() {
        let source = concat!(
            "SINGLE = \"a ''' inside a double-quoted string\"\n",
            "OTHER = 'a \"\"\" inside a single-quoted string'\n",
            "ESCAPED = \"she said \\\"\\\"\\\" loudly\"\n",
            "# a bare \"\"\" in a comment\n",
            "TRAILING = 1  # and ''' after code\n",
            "DOC = \"\"\"\n",
            "# a comment marker inside the literal does not end it\n",
            "def phantom_in_doc():\n",
            "    pass\n",
            "\"\"\"\n",
            "\n",
            "def test_state_never_toggled():\n",
            "    assert True\n",
        );
        let symbols = parse("t_test.py", source).expect("parses");

        assert!(
            naive_line_scan_finds_declaration(source, "phantom_in_doc"),
            "fixture premise: a naive scanner sees the embedded def"
        );
        assert!(
            !names(&symbols).contains(&"phantom_in_doc"),
            "the literal's body is not code: {:?}",
            names(&symbols)
        );
        assert_eq!(
            symbol(&symbols, "test_state_never_toggled").kind,
            SymbolKind::TestFunction
        );
    }

    /// TC-1031 successor (PLAT-868, retires
    /// `tc1031_the_scope_stack_survives_an_embedded_string`): a declaration
    /// after an embedded string keeps its true container — the tree gives
    /// each `class`/`def` its real parent directly, so there is no scope
    /// *stack* that could resume stale the way the pre-port scanner's did
    /// (#274: it read 10 of 21 classes in a real file, attributing several
    /// methods to the wrong enclosing class after an embedded docstring
    /// fixture). Confirmed red first against a stand-in that flattens every
    /// `class`/`def` line to a single global scope regardless of nesting —
    /// the shape of bug #274 was — which cannot distinguish the two
    /// `test_*` methods' containers; `parse` does.
    #[trace("TC-1031", "FR-051-AC-20")]
    #[test]
    fn tc1031_the_true_container_survives_an_embedded_string() {
        let source = concat!(
            "class TestParsing:\n",
            "    def test_reads_a_fixture(self):\n",
            "        source = \"\"\"\n",
            "class Injected:\n",
            "    pass\n",
            "\"\"\"\n",
            "        assert source\n",
            "\n",
            "\n",
            "class TestModification:\n",
            "    def test_writes_a_fixture(self):\n",
            "        assert True\n",
        );
        let symbols = parse("t_test.py", source).expect("parses");

        assert!(
            !names(&symbols).contains(&"Injected"),
            "the embedded class is string content: {:?}",
            names(&symbols)
        );

        assert_eq!(
            symbol(&symbols, "TestModification").kind,
            SymbolKind::Container
        );
        let method = symbol(&symbols, "TestModification.test_writes_a_fixture");
        assert_eq!(method.kind, SymbolKind::TestFunction);
        assert_eq!(method.container.as_deref(), Some("TestModification"));

        let first = symbol(&symbols, "TestParsing.test_reads_a_fixture");
        assert_eq!(first.container.as_deref(), Some("TestParsing"));

        // A naive single-scope stand-in — a scope stack that never pops
        // once entered, the #274 defect shape — cannot distinguish these
        // two methods' containers: it always reports whichever `class` line
        // came *first* in the source. This is computed from the fixture
        // itself, not hand-typed, so renaming the fixture's first class
        // cannot make the property silently vacuous the way a bare string
        // literal here could (PLAT-868 PR #479 review, F8).
        let naive_single_scope =
            naive_never_popped_scope(source).expect("fixture premise: opens at least one class");
        assert_eq!(
            naive_single_scope, "TestParsing",
            "fixture premise: the first class in source is TestParsing"
        );
        assert_ne!(
            method.container.as_deref(),
            Some(naive_single_scope),
            "a stale scope would misattribute the second method to the first class"
        );
    }

    /// A minimal stand-in for a scope stack that never pops once entered —
    /// the #274 defect shape: every method after the first `class` line
    /// reports that same first class as its container, however many later
    /// classes close and reopen. Returns the first `class NAME` line's
    /// `NAME`, read from `source` itself (not hand-typed) so the comparison
    /// in `tc1031` stays tied to the fixture it checks.
    fn naive_never_popped_scope(source: &str) -> Option<&str> {
        source.lines().find_map(|line| {
            line.trim_start().strip_prefix("class ").map(|rest| {
                rest.split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or("")
            })
        })
    }

    /// TC-800, FR-051-AC-13 (CR-037): a signature black-wrapped over
    /// multiple lines still reaches its docstring — `function_definition`'s
    /// own node span already covers the full signature, so no bespoke
    /// paren-depth walk (the pre-port scanner's `block_end`/`paren_delta`,
    /// deleted by this port) is needed. This also backs `TC-1880`
    /// (`FR-051-AC-25`)'s "a `def` whose signature spans lines" clause:
    /// `test_multi_line`'s wrapped signature binds `TC-029` exactly as
    /// `test_single_line`'s single-line spelling binds `TC-028` — the
    /// single-line form is the control.
    #[trace("TC-1880", "FR-051-AC-25")]
    #[test]
    fn tc800_wrapped_signature_span_reaches_the_docstring() {
        let source = concat!(
            "def test_single_line(traceability: dict) -> None:\n",
            "    \"\"\"TC-028 (FR-004-AC-1): one line signature.\"\"\"\n",
            "    assert True\n",
            "\n",
            "\n",
            "def test_multi_line(\n",
            "    traceability: dict,\n",
            ") -> None:\n",
            "    \"\"\"TC-029 (FR-004-AC-2): signature split by black.\"\"\"\n",
            "    assert True\n",
        );
        let symbols = parse("t_test.py", source).expect("parses");

        for (name, tag) in [
            ("test_single_line", "TC-028"),
            ("test_multi_line", "TC-029"),
        ] {
            let symbol = symbols
                .iter()
                .find(|s| s.qualified_name == name)
                .unwrap_or_else(|| panic!("no symbol for {name}"));
            assert_eq!(symbol.kind, SymbolKind::TestFunction, "{name}");
            let span = source
                .lines()
                .skip(symbol.leading_line - 1)
                .take(symbol.end_line - symbol.leading_line + 1)
                .collect::<Vec<_>>()
                .join("\n");
            assert!(span.contains(tag), "{name} span misses {tag}:\n{span}");
        }
    }

    /// PLAT-234, `TC-1880` (`FR-051-AC-25`): a black-wrapped, multi-line
    /// `@pytest.mark.trace(...)` decorator still reaches `leading_line` —
    /// this is the exact ticket this port closes. The pre-port scanner's
    /// physical-line walk broke on the first continuation line (it does not
    /// start with `@`); tree-sitter tokenizes the whole call as one
    /// `decorator` node inside `decorated_definition`, whose own span
    /// already starts there. The single-line spelling below is the control:
    /// both bind `TC-028` from their span (reusing `tc800`'s own real id
    /// rather than inventing a new one — an invented id here would be an
    /// id-shaped literal with no declared row, landing in this repo's own
    /// `unmatched_tags` population, PLAT-868 PR #479 review, F5).
    #[trace("TC-1880", "FR-051-AC-25")]
    #[test]
    fn plat234_a_black_wrapped_multiline_decorator_reaches_leading_line() {
        let wrapped = concat!(
            "@pytest.mark.trace(\n",
            "    \"TC-028\",\n",
            "    \"FR-051-AC-1\",\n",
            ")\n",
            "def test_wrapped_decorator():\n",
            "    assert True\n",
        );
        let control = concat!(
            "@pytest.mark.trace(\"TC-028\", \"FR-051-AC-1\")\n",
            "def test_wrapped_decorator():\n",
            "    assert True\n",
        );
        for (source, label) in [(wrapped, "wrapped"), (control, "single-line control")] {
            let symbols = parse("t_test.py", source).expect("parses");
            let symbol = symbols
                .iter()
                .find(|s| s.qualified_name == "test_wrapped_decorator")
                .unwrap_or_else(|| panic!("{label}: the decorated test is a symbol"));
            assert_eq!(symbol.kind, SymbolKind::TestFunction, "{label}");
            assert_eq!(
                symbol.leading_line, 1,
                "{label}: the decorator's own first line"
            );
            let span = source
                .lines()
                .skip(symbol.leading_line - 1)
                .take(symbol.end_line - symbol.leading_line + 1)
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                span.contains("TC-028"),
                "{label}: span misses the wrapped tag:\n{span}"
            );
        }
    }

    /// `TC-1880` (`FR-051-AC-25`): a wrapped `@pytest.mark.trace(...)`
    /// separated from its `def` by a second, also-wrapped decorator (e.g.
    /// `@pytest.mark.parametrize(...)`) still binds from `leading_line` —
    /// `decorated_definition`'s span starts at its first `decorator` child
    /// regardless of how many decorators follow it or how each one wraps.
    /// The single-line spelling of both decorators is the control: both
    /// bind `TC-029` from their span (see `plat234`'s own doc comment above
    /// for why this reuses a real id rather than inventing one).
    #[trace("TC-1880", "FR-051-AC-25")]
    #[test]
    fn a_second_wrapped_decorator_between_the_tag_and_def_does_not_move_leading_line() {
        let wrapped = concat!(
            "@pytest.mark.trace(\n",
            "    \"TC-029\",\n",
            "    \"FR-051-AC-1\",\n",
            ")\n",
            "@pytest.mark.parametrize(\n",
            "    \"x\",\n",
            "    [1, 2],\n",
            ")\n",
            "def test_two_wrapped_decorators(x):\n",
            "    assert True\n",
        );
        let control = concat!(
            "@pytest.mark.trace(\"TC-029\", \"FR-051-AC-1\")\n",
            "@pytest.mark.parametrize(\"x\", [1, 2])\n",
            "def test_two_wrapped_decorators(x):\n",
            "    assert True\n",
        );
        for (source, label) in [(wrapped, "wrapped"), (control, "single-line control")] {
            let symbols = parse("t_test.py", source).expect("parses");
            let symbol = symbols
                .iter()
                .find(|s| s.qualified_name == "test_two_wrapped_decorators")
                .unwrap_or_else(|| panic!("{label}: the decorated test is a symbol"));
            assert_eq!(symbol.kind, SymbolKind::TestFunction, "{label}");
            assert_eq!(
                symbol.leading_line, 1,
                "{label}: the first decorator's own first line"
            );
            let span = source
                .lines()
                .skip(symbol.leading_line - 1)
                .take(symbol.end_line - symbol.leading_line + 1)
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                span.contains("TC-029"),
                "{label}: span misses the wrapped tag:\n{span}"
            );
        }
    }

    /// PLAT-868 PR #479 review, F4: a trailing comment sharing the
    /// *previous* statement's own line must not be pulled into the next
    /// declaration's leading span — it is not an annotation of anything
    /// below it. Confirmed as a real regression against the pre-port
    /// scanner's behaviour before this fix: `leading_span` originally
    /// treated any preceding `comment`-kind sibling as leading, and
    /// tree-sitter emits a trailing comment as a sibling exactly like an
    /// own-line one, so `# note` here would have been walked into `f`'s
    /// span — the same hazard as PLAT-234, but silently *creating* a
    /// binding rather than losing one, had that comment carried a
    /// trace-shaped tag.
    #[test]
    fn a_trailing_comment_on_the_previous_line_is_not_a_leading_annotation() {
        let source = concat!("x = 1  # note\n", "def f():\n", "    pass\n",);
        let symbols = parse("t.py", source).expect("parses");
        let f = symbol(&symbols, "f");
        assert_eq!(
            f.leading_line, 2,
            "a same-line trailing comment must not be pulled into the leading span"
        );
    }

    /// PLAT-868 PR #479 review, F4(b): what a comment's *indentation*
    /// contributes, beyond the own-line check `leading_span` itself makes.
    /// tree-sitter's `comment` is an "extra" token attached to wherever it
    /// falls in the token stream relative to Python's own INDENT/DEDENT —
    /// not to `leading_span`'s own logic, which only ever looks at sibling
    /// nodes.
    ///
    /// A comment still indented at the *previous* declaration's body level,
    /// appearing before the dedent back to module level, is swallowed as a
    /// trailing child of that previous `block` — never a sibling of the
    /// next declaration at all, so it cannot extend that declaration's
    /// span (`h.leading_line` stays on `h`'s own line, unaffected by `g`'s
    /// trailing comment). A comment already dedented to the next
    /// declaration's own level *is* that declaration's sibling, and does
    /// extend its span, the ordinary case every other `leading_span` test
    /// here already exercises. Documented because the review that raised
    /// this (F4) asked what the indented case does: it does not move
    /// `leading_line` relative to `h`'s own line; against the pre-port
    /// scanner it shifts 3→4, a favourable direction (the comment is `g`'s)
    /// with zero corpus occurrences. The pre-port `leading_block` trimmed
    /// each preceding line and accepted any that began with `#`, so it
    /// walked this comment into `h`'s span; the indentation-sensitive
    /// grammar keeps it contained in the block it visually belongs to.
    #[test]
    fn an_indented_trailing_comment_stays_inside_the_previous_body_not_the_next_span() {
        let indented_before_dedent = concat!(
            "def g():\n",
            "    pass\n",
            "    # still indented like g's body, but g is done\n",
            "def h():\n",
            "    pass\n",
        );
        let symbols = parse("t.py", indented_before_dedent).expect("parses");
        let h = symbol(&symbols, "h");
        assert_eq!(
            h.leading_line, 4,
            "a comment still indented at g's body level is g's, not h's"
        );

        let dedented_to_module_level = concat!(
            "def g():\n",
            "    pass\n",
            "# module level comment\n",
            "def h():\n",
            "    pass\n",
        );
        let symbols2 = parse("t.py", dedented_to_module_level).expect("parses");
        let h2 = symbol(&symbols2, "h");
        assert_eq!(
            h2.leading_line, 3,
            "a comment already dedented to h's own level is h's leading annotation"
        );
    }

    /// FR-051-AC-1/AC-2: a `def` nested inside another `def` is never its
    /// own container — matches the pre-port scanner exactly (only a `class`
    /// pushes a scope), so two nested helpers of the same name, under two
    /// differently-named outer functions with no enclosing class, collide on
    /// identity exactly as they did before this port. This is a
    /// byte-for-byte preserved quirk, not something this port is free to
    /// fix (PLAT-845 made the opposite call for Rust; Python's own identity
    /// surface is pinned separately by PLAT-868).
    #[test]
    fn a_nested_def_is_never_a_container_for_its_own_nested_defs() {
        let source = concat!(
            "def outer_one():\n",
            "    def helper():\n",
            "        pass\n",
            "\n",
            "\n",
            "def outer_two():\n",
            "    def helper():\n",
            "        pass\n",
        );
        let symbols = parse("t_test.py", source).expect("parses");
        let helpers: Vec<&RawSymbol> = symbols
            .iter()
            .filter(|s| s.qualified_name == "helper")
            .collect();
        assert_eq!(
            helpers.len(),
            2,
            "both nested helpers are extracted, bare-named: {symbols:?}"
        );
        assert!(
            helpers
                .iter()
                .all(|s| s.container.as_deref() == Some("t_test")),
            "neither nests under its own enclosing function: {helpers:?}"
        );
    }

    /// FR-051-AC-3, #407: the bounded unittest `TestCase` base detection —
    /// a direct import, an aliased import, a comma-separated base list, and
    /// the documented non-inference cases (a rebound import, a non-`unittest`
    /// module, and a multi-line base list).
    #[test]
    fn unittest_test_case_base_is_bounded_and_import_gated() {
        let source = concat!(
            "import unittest\n",
            "\n",
            "class NativeCase(unittest.TestCase):\n",
            "    def test_direct(self):\n",
            "        assert True\n",
            "\n",
            "    def helper(self):\n",
            "        return 1\n",
            "\n",
            "\n",
            "from unittest import TestCase as TC\n",
            "\n",
            "\n",
            "class AliasedCase(TC):\n",
            "    def test_aliased(self):\n",
            "        assert True\n",
            "\n",
            "\n",
            "class OrdinaryHelper(unittest.TestCase):\n",
            "    def not_test_named(self):\n",
            "        return 2\n",
        );
        let symbols = parse("t_test.py", source).expect("parses");
        assert_eq!(
            symbol(&symbols, "NativeCase.test_direct").kind,
            SymbolKind::TestFunction,
            "a direct unittest.TestCase base makes a test_-prefixed method evidence \
             with no Test-prefixed class name needed"
        );
        assert_eq!(
            symbol(&symbols, "NativeCase.helper").kind,
            SymbolKind::Function,
            "a non-test_-prefixed method on the same class is not evidence"
        );
        assert_eq!(
            symbol(&symbols, "AliasedCase.test_aliased").kind,
            SymbolKind::TestFunction,
            "an aliased `from unittest import TestCase as TC` base is recognized"
        );
        assert_eq!(
            symbol(&symbols, "OrdinaryHelper.not_test_named").kind,
            SymbolKind::Function,
            "a method not named test_* is never evidence, unittest base or not"
        );
    }

    /// #407: an explicit rebinding removes a tracked identity — a same-name
    /// top-level assignment after the import invalidates the unittest base
    /// for any class declared afterward.
    #[test]
    fn a_rebound_import_removes_the_unittest_identity() {
        let source = concat!(
            "from unittest import TestCase\n",
            "TestCase = object\n",
            "\n",
            "class NowOrdinary(TestCase):\n",
            "    def test_not_evidence(self):\n",
            "        assert True\n",
        );
        let symbols = parse("t_test.py", source).expect("parses");
        assert_eq!(
            symbol(&symbols, "NowOrdinary.test_not_evidence").kind,
            SymbolKind::Function,
            "the rebinding must invalidate the TestCase identity before the class"
        );
    }

    /// #407: a nested class's base list is never inferred, matching the
    /// pre-port scanner's `indent == 0` restriction — only a *top-level*
    /// class header is checked.
    #[test]
    fn a_nested_class_base_is_never_inferred_as_unittest() {
        let source = concat!(
            "import unittest\n",
            "\n",
            "class Outer:\n",
            "    class Nested(unittest.TestCase):\n",
            "        def test_nested(self):\n",
            "            assert True\n",
        );
        let symbols = parse("t_test.py", source).expect("parses");
        assert_eq!(
            symbol(&symbols, "Outer.Nested.test_nested").kind,
            SymbolKind::Function,
            "a nested class's own base list is never checked for a unittest identity"
        );
    }

    /// A paren inside a string default is unaffected — a `function_definition`
    /// node's own span already resolves this; this asserts the *outcome*
    /// (line attributes) rather than a deleted `paren_delta` helper's return
    /// value.
    #[test]
    fn default_argument_parens_do_not_move_the_span() {
        let source = "def f(a: str = \"(\") -> None:\n    return a\n";
        let symbols = parse("t_test.py", source).expect("parses");
        let f = symbol(&symbols, "f");
        assert_eq!(f.line, 1);
        assert_eq!(f.end_line, 2);
    }
}
