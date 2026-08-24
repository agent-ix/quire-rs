//! TypeScript source adapter (FR-051).
//!
//! Line-structural like the Rust adapter: brace depth for scopes, leading
//! keywords for declarations. The file itself is a container symbol (the
//! module), named by its extension-less path.
//!
//! Test classification follows the vitest/jest convention: a `test(...)` or
//! `it(...)` registration is a test symbol, and **its registered title is the
//! qualified name** (FR-051) — that is the identity a report or a trace marker
//! refers to, not an anonymous arrow function.
//!
//! A `describe(...)` / `suite(...)` registration is a **container** symbol
//! (CR-119): it groups tests rather than being one, so it is a scope the
//! registrations inside it are parented to — never a `TestFunction`, which
//! would make the suite evidence in its own right.

use std::sync::OnceLock;

use regex::Regex;

use super::{leading_block, RawSymbol, SymbolKind};

/// Parse `source` into raw symbols, or return a per-file reason to skip it.
pub(crate) fn parse(path: &str, source: &str) -> Result<Vec<RawSymbol>, String> {
    let lines: Vec<&str> = source.lines().collect();
    // One lexer pass for the whole file (CR-039). Everything downstream reads
    // it rather than re-deriving comment/string/template state.
    let lexed = lex(&lines);
    check_balanced(&lexed)?;

    let module = module_name(path);
    let mut out = vec![RawSymbol {
        qualified_name: module.clone(),
        kind: SymbolKind::Container,
        line: 1,
        leading_line: 1,
        end_line: lines.len().max(1),
        container: None,
    }];

    let mut scopes: Vec<Scope> = Vec::new();
    let mut depth: i64 = 0;

    for (idx, lexed_line) in lexed.iter().enumerate() {
        let trimmed = lexed_line.code.trim();

        if let Some(title) = registration(&lexed, idx, TEST_NAMES) {
            push(
                &mut out,
                &lines,
                &lexed,
                idx,
                title,
                SymbolKind::TestFunction,
                scope_container(&scopes, &module),
            );
        } else if let Some(title) = registration(&lexed, idx, SUITE_NAMES) {
            push(
                &mut out,
                &lines,
                &lexed,
                idx,
                title.clone(),
                SymbolKind::Container,
                scope_container(&scopes, &module),
            );
            if trimmed.contains('{') {
                scopes.push(Scope {
                    name: title,
                    depth,
                    qualifies: false,
                });
            }
        } else if let Some(name) = class_declaration(trimmed) {
            let qualified = qualify(&scopes, &name);
            push(
                &mut out,
                &lines,
                &lexed,
                idx,
                qualified.clone(),
                SymbolKind::Container,
                scope_container(&scopes, &module),
            );
            if trimmed.contains('{') {
                scopes.push(Scope {
                    name: qualified,
                    depth,
                    qualifies: true,
                });
            }
        } else if let Some(name) = function_declaration(trimmed) {
            push(
                &mut out,
                &lines,
                &lexed,
                idx,
                qualify(&scopes, &name),
                SymbolKind::Function,
                scope_container(&scopes, &module),
            );
        }

        depth += lexed_line.delta;
        while let Some(scope) = scopes.last() {
            if depth <= scope.depth {
                scopes.pop();
            } else {
                break;
            }
        }
    }
    Ok(out)
}

fn push(
    out: &mut Vec<RawSymbol>,
    lines: &[&str],
    lexed: &[LexedLine],
    idx: usize,
    qualified_name: String,
    kind: SymbolKind,
    container: Option<String>,
) {
    out.push(RawSymbol {
        qualified_name,
        kind,
        line: idx + 1,
        leading_line: leading_block(lines, idx, is_annotation),
        end_line: block_end(lexed, idx),
        container,
    });
}

/// One open lexical scope: what it is called, the brace depth it opened at,
/// and whether declarations inside it are *named through* it.
///
/// The two axes are separate because a suite is a scope that does not rename
/// its members (CR-119). A class qualifies — `Harness.ready` — but a
/// registration's qualified name **is its registered title** (FR-051), so
/// prefixing it with the enclosing `describe(…)` title would change the
/// identity of every test in the ecosystem that sits inside one.
struct Scope {
    name: String,
    depth: i64,
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
/// A suite counts here even though it does not qualify: `contains` is what
/// records that a `describe(…)` groups the registrations written inside it,
/// and a container that contains nothing is not a grouping (FR-051-AC-8).
fn scope_container(scopes: &[Scope], module: &str) -> Option<String> {
    scopes
        .last()
        .map(|scope| scope.name.clone())
        .or_else(|| Some(module.to_string()))
}

/// How far past the opening line a registration's title may begin. Vitest and
/// prettier both wrap a long curried call, and the title lands on the very next
/// line; three is slack for a formatted argument list without letting a string
/// several statements away become a test name.
const TITLE_LOOKAHEAD_LINES: usize = 3;

/// The callees that open a **test** registration. Its title is the test
/// symbol's qualified name (FR-051).
const TEST_NAMES: &[&str] = &["test", "it"];

/// The callees that open a **suite** registration — a container, not evidence
/// (CR-119).
///
/// `context` is deliberately absent, and the reason is measured rather than
/// stylistic. It is mocha's TDD-interface alias, which nothing in this
/// ecosystem uses: across `~/dev` there are **1,699** `describe(` registrations
/// with a quoted title in 103 repository roots, **zero** `suite(`, and **zero**
/// `context(` — while five lines do begin `context.` (`context.setTransform(`,
/// `context.client.putSettings(`), method calls on an object that happens to
/// be named `context`. Admitting the name would put a grammar that reads a
/// `.modifier` chain in front of exactly one shape it cannot tell from a suite,
/// for no occurrence it would recover. `suite` is admitted despite measuring
/// zero because it is the same construct under another harness's spelling and
/// carries no such collision.
const SUITE_NAMES: &[&str] = &["describe", "suite"];

/// A registration by one of `names`, with its quoted title — a
/// `test('title', …)` / `it("title", …)` test, or a `describe("title", …)`
/// suite. The title is the symbol name.
///
/// Reads the whole-file lex rather than one line (CR-084), which buys the two
/// shapes the single-line regex could not see:
///
/// * **Curried modifiers.** `it.skipIf(cond)(…)` and `it.each([…])(…)` are the
///   conditional and parametrised forms both vitest and jest ship. The first
///   `(` holds the condition, not the title.
/// * **A title on a later line**, which is simply how either of the above is
///   formatted once it exceeds the line width.
///
/// Neither registered a symbol at all before, and that is worse than a missed
/// tag: with no symbol, a legacy comment id and a canonical `trace(…)` call
/// inside the body have nothing to attach to, so the test runs, passes and binds
/// nothing, silently and always in the direction that loses coverage.
fn registration(lexed: &[LexedLine], idx: usize, names: &[&str]) -> Option<String> {
    let first = lexed.get(idx)?.code.trim_start();
    let open = registration_open(first, names)?;
    read_title(lexed, idx, first, open)
}

/// Byte offset just past the `(` that opens the registration call, or `None`
/// when the line does not open one.
fn registration_open(line: &str, names: &[&str]) -> Option<usize> {
    let bytes = line.as_bytes();
    let ident = |c: &u8| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'$';
    let mut i = 0;

    if line.starts_with("await") && bytes.get(5).is_some_and(u8::is_ascii_whitespace) {
        i = 5;
        while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
    }

    // One of `names`, and nothing longer — `iterate(` registers nothing.
    let name = names.iter().find(|n| line[i..].starts_with(**n))?;
    i += name.len();
    if bytes.get(i).is_some_and(ident) {
        return None;
    }

    loop {
        // A `.modifier` chain: `.skip`, `.each`, `.concurrent.skip`, …
        while bytes.get(i) == Some(&b'.') {
            i += 1;
            let start = i;
            while bytes.get(i).is_some_and(ident) {
                i += 1;
            }
            if i == start {
                return None;
            }
        }
        while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
            i += 1;
        }
        if bytes.get(i) != Some(&b'(') {
            return None;
        }
        let open = i + 1;
        match close_paren(bytes, i) {
            // The group closed on this line and another `(` follows, so what
            // closed was a curried modifier's arguments and the registration
            // call is the next one along.
            Some(close) => {
                let mut j = close + 1;
                while bytes.get(j).is_some_and(u8::is_ascii_whitespace) {
                    j += 1;
                }
                if bytes.get(j) == Some(&b'(') {
                    i = j;
                    continue;
                }
                return Some(open);
            }
            // Unclosed on this line: the callback body spans lines, so this
            // group *is* the registration call.
            None => return Some(open),
        }
    }
}

/// Offset of the `)` matching the `(` at `open`, searching this line only.
/// Quoted spans are skipped, so a paren inside a string literal cannot unbalance
/// the count. `None` when the group does not close on this line.
fn close_paren(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut i = open;
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let c = bytes[i];
        match quote {
            Some(q) => {
                if c == b'\\' {
                    i += 2;
                    continue;
                }
                if c == q {
                    quote = None;
                }
            }
            None => match c {
                b'\'' | b'"' | b'`' => quote = Some(c),
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            },
        }
        i += 1;
    }
    None
}

/// The quoted title at or after `open`, following the line break into at most
/// [`TITLE_LOOKAHEAD_LINES`] further lines.
///
/// Stops at the first non-blank text rather than hunting for a quote: in
/// `it(name, …)` the title is a variable, and scanning past it would name the
/// test after some unrelated string literal further down the argument list.
///
/// **Known limitation, by design.** `lex_line` drops content carried in from an
/// unterminated template literal, so a title written inside a multi-line
/// template yields no quote here and registers nothing — the same outcome as
/// before this change, and preferable to registering a wrong name.
fn read_title(lexed: &[LexedLine], idx: usize, first: &str, open: usize) -> Option<String> {
    for offset in 0..=TITLE_LOOKAHEAD_LINES {
        let text = if offset == 0 {
            first.get(open..)?
        } else {
            lexed.get(idx + offset)?.code.as_str()
        };
        let text = text.trim_start();
        if text.is_empty() {
            continue;
        }
        return quoted(text);
    }
    None
}

/// Contents of a leading quoted literal. Escapes are not interpreted, matching
/// the `[^']*` the single-line regex used before CR-084.
fn quoted(text: &str) -> Option<String> {
    let quote = *text.as_bytes().first()?;
    if !matches!(quote, b'\'' | b'"' | b'`') {
        return None;
    }
    let rest = text.get(1..)?;
    let end = rest.find(quote as char)?;
    Some(rest[..end].to_string())
}

fn class_declaration(trimmed: &str) -> Option<String> {
    let rest = trimmed
        .strip_prefix("export default ")
        .or_else(|| trimmed.strip_prefix("export "))
        .unwrap_or(trimmed);
    let rest = rest.strip_prefix("abstract ").unwrap_or(rest);
    ident(rest.strip_prefix("class ")?)
}

/// `function f(…)`, `const f = (…) =>`, and class methods (`name(…) {`).
fn function_declaration(trimmed: &str) -> Option<String> {
    let rest = trimmed
        .strip_prefix("export default ")
        .or_else(|| trimmed.strip_prefix("export "))
        .unwrap_or(trimmed);
    let rest = rest.strip_prefix("async ").unwrap_or(rest);
    if let Some(after) = rest.strip_prefix("function ") {
        return ident(after);
    }
    if let Some(caps) = re_arrow_const().captures(rest) {
        return Some(caps[1].to_string());
    }
    re_method()
        .captures(rest)
        .map(|c| c[1].to_string())
        .filter(|name| !RESERVED.contains(&name.as_str()))
}

/// Keywords whose `keyword (…) {` shape looks like a method declaration.
const RESERVED: &[&str] = &[
    "if",
    "for",
    "while",
    "switch",
    "catch",
    "return",
    "function",
    "constructor",
    "do",
    "else",
];

fn ident(s: &str) -> Option<String> {
    let name: String = s
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
        .collect();
    (!name.is_empty()).then_some(name)
}

fn is_annotation(line: &str) -> bool {
    line.starts_with("//") || line.starts_with('@') || line.starts_with('*')
}

fn module_name(path: &str) -> String {
    let stem = path
        .strip_suffix(".tsx")
        .or_else(|| path.strip_suffix(".ts"))
        .unwrap_or(path);
    stem.to_string()
}

/// Where a declaration's block ends, read off the file-wide lex (CR-039).
///
/// It used to restart a `ScanState` at the declaration index, so a declaration
/// following an unclosed block comment or template literal had its span
/// computed as if the file began there. `lexed` already carries the state the
/// declaration is actually in.
fn block_end(lexed: &[LexedLine], decl_idx: usize) -> usize {
    let mut depth = 0i64;
    let mut seen_open = false;
    for (offset, line) in lexed[decl_idx..].iter().enumerate() {
        if line.code.contains('{') {
            seen_open = true;
        }
        depth += line.delta;
        if seen_open && depth <= 0 {
            return decl_idx + offset + 1;
        }
        if !seen_open && line.code.trim_end().ends_with(';') {
            return decl_idx + offset + 1;
        }
    }
    lexed.len().max(decl_idx + 1)
}

/// Comment/literal state that must survive from one line to the next.
///
/// A block comment spans lines, and so does a **template literal** — the only
/// string form in TS/JS that can. Both have to be carried, or the scanner
/// re-enters each line believing it is in code.
#[derive(Debug, Default, Clone, Copy)]
struct ScanState {
    in_block_comment: bool,
    in_template: bool,
}

/// One lexed line: the code with comments and carried literal content removed,
/// and the brace delta counted **in the same pass** (CR-039).
///
/// The delta used to be recomputed by a second function that re-derived quote
/// state from the stripped text. The two agreed, but only incidentally — three
/// functions each deriving "am I in a string?" by slightly different rules is
/// how CR-036 and CR-037 both happened. There is now one derivation.
#[derive(Debug, Default, Clone)]
struct LexedLine {
    code: String,
    delta: i64,
}

/// Lex the file once, carrying comment and template state line to line.
fn lex(lines: &[&str]) -> Vec<LexedLine> {
    let mut state = ScanState::default();
    lines
        .iter()
        .map(|line| lex_line(line, &mut state))
        .collect()
}

/// Drop comments from one line, leaving string literals intact, and count the
/// braces that are actually code.
///
/// String-aware by necessity (CR-036): a `/*` inside a literal is not a comment
/// opener. A git refspec in a template literal —
/// `` `fetch = +refs/heads/*:refs/remotes/origin/*` `` — used to open a block
/// comment that never closed, so every following line was stripped, the braces
/// could not balance, and [`check_balanced`] rejected the whole file. A file
/// rejected that way yields **zero** symbols, so every trace tag in it binds to
/// nothing — silently, since the file is otherwise perfectly valid TypeScript.
///
/// Template state is carried across lines for the same reason: the corpus form
/// that triggered this writes the refspec on a *continuation* line, where a
/// per-line scanner has already forgotten it is inside a literal.
fn lex_line(line: &str, state: &mut ScanState) -> LexedLine {
    let mut out = String::with_capacity(line.len());
    let mut delta = 0i64;
    let chars: Vec<char> = line.chars().collect();
    // Only a template literal carries in; `'` and `"` cannot span a line.
    //
    // A carried-in literal's content is **dropped** rather than copied: a
    // continuation line is never a declaration, and a `${…}` interpolation is
    // balanced, so removing it leaves the depth intact. A literal that opens
    // and closes on one line is still copied, since a backtick-quoted
    // `test(`title`, …)` title has to survive for [`registration`] to read it.
    let mut quote: Option<char> = None;
    let mut i = 0;
    if state.in_template {
        let mut closed = false;
        while i < chars.len() {
            if chars[i] == '\\' {
                i += 2;
                continue;
            }
            if chars[i] == '`' {
                closed = true;
                i += 1;
                break;
            }
            i += 1;
        }
        if !closed {
            // The whole line is literal content and the literal continues, so
            // the carried flag must survive the early return below.
            return LexedLine { code: out, delta };
        }
        state.in_template = false;
    }
    while i < chars.len() {
        if state.in_block_comment {
            if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                state.in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        // Inside a literal nothing is a comment; the escape is copied with the
        // character it escapes so a trailing `\` cannot swallow the closer.
        if let Some(q) = quote {
            if chars[i] == '\\' {
                out.push(chars[i]);
                if let Some(&next) = chars.get(i + 1) {
                    out.push(next);
                }
                i += 2;
                continue;
            }
            if chars[i] == q {
                quote = None;
            }
            out.push(chars[i]);
            i += 1;
            continue;
        }
        if matches!(chars[i], '"' | '\'' | '`') {
            quote = Some(chars[i]);
            out.push(chars[i]);
            i += 1;
            continue;
        }
        if chars[i] == '/' && chars.get(i + 1) == Some(&'/') {
            break;
        }
        if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
            state.in_block_comment = true;
            i += 2;
            continue;
        }
        // Only braces reached here are code: not in a comment, not in a
        // literal, not in carried template content.
        match chars[i] {
            '{' => delta += 1,
            '}' => delta -= 1,
            _ => {}
        }
        out.push(chars[i]);
        i += 1;
    }
    // A `'`/`"` left open at end of line is a malformed line, not a carried
    // literal — only a backtick continues.
    state.in_template = quote == Some('`');
    LexedLine { code: out, delta }
}

fn check_balanced(lexed: &[LexedLine]) -> Result<(), String> {
    let mut depth = 0i64;
    for line in lexed {
        depth += line.delta;
        if depth < 0 {
            return Err("unbalanced braces: a `}` closes no block".to_string());
        }
    }
    if depth != 0 {
        return Err(format!("unbalanced braces: {depth} block(s) left open"));
    }
    Ok(())
}

fn re_arrow_const() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*(?::[^=]+)?=\s*(?:async\s+)?\(")
            .expect("arrow-const regex")
    })
}

fn re_method() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^(?:public\s+|private\s+|protected\s+|static\s+)*([A-Za-z_$][\w$]*)\s*\([^;]*\)\s*(?::\s*[^{]+)?\{")
            .expect("method regex")
    })
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
        // A variable title. The scan must stop at `name`, not walk on to the
        // string two lines down.
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

        // Beyond the lookahead window: a title four lines down is not this
        // registration's.
        let far = concat!(
            "it(\n",
            "\n",
            "\n",
            "\n",
            "  \"TC-502 too far to be ours\",\n",
            ");\n",
        );
        assert!(
            parse("c.test.ts", far)
                .expect("parses")
                .iter()
                .all(|s| s.kind != SymbolKind::TestFunction),
            "the lookahead window is bounded",
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

        // A real comment still strips: a `//` outside a literal hides its line.
        let mut state = ScanState::default();
        let lexed = lex_line("const a = 1; // { unbalanced", &mut state);
        assert_eq!(lexed.code, "const a = 1; ");
        // The `{` was inside the comment, so it is not a block open either.
        assert_eq!(lexed.delta, 0);
        assert!(!state.in_block_comment);
    }

    /// TC-799, FR-051-AC-12 (CR-036): the same `/*`, on a **continuation** line
    /// of a multi-line template literal.
    ///
    /// A per-line scanner re-enters each line believing it is in code, so it
    /// re-opened the block comment one line later and the file was rejected
    /// exactly as before. The first fix handled only the single-line form; this
    /// is the form the corpus actually writes, and it is why template state is
    /// carried across lines rather than reset.
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

        // The literal opens and does not close on its own line, and closes on a
        // later one — the state has to say so at each boundary.
        let mut state = ScanState::default();
        lex_line("const cfg = `", &mut state);
        assert!(
            state.in_template,
            "an unclosed backtick carries into the next line"
        );
        lex_line("fetch = +refs/heads/*:refs/remotes/origin/*", &mut state);
        assert!(
            !state.in_block_comment,
            "`/*` inside the literal is content"
        );
        assert!(state.in_template);
        lex_line("`;", &mut state);
        assert!(!state.in_template, "the closing backtick ends it");
    }

    /// TC-803, FR-051-AC-14 (CR-039): every consumer reads one lex, so the
    /// three derivations that used to disagree cannot.
    ///
    /// The file carries all three constructs that defeated a per-line scanner:
    /// a block comment holding an unbalanced brace, a multi-line template
    /// holding another, and an unterminated quote followed by what looks like a
    /// comment. Under the old shape each of `check_balanced`, `brace_delta` and
    /// `block_end` derived string/comment state its own way; they agreed here,
    /// but incidentally. The assertion is now the agreement itself: the deltas
    /// the balance check sums are the same deltas the spans are cut from.
    #[test]
    fn tc803_one_lex_serves_every_consumer() {
        let source = concat!(
            "/* a comment holding a brace {\n",
            "   and closing here */\n",
            "const cfg = `\n",
            "a { bare brace in a literal\n",
            "`;\n",
            "const re = /['\"]/; // a comment after an unterminated quote {\n",
            "test(\"holds\", () => {\n",
            "  expect(1).toBe(1);\n",
            "});\n",
        );
        let lines: Vec<&str> = source.lines().collect();
        let lexed = lex(&lines);

        // None of the three decoy braces is code, so the file balances on the
        // one real block alone.
        check_balanced(&lexed).expect("the file balances");
        assert_eq!(lexed.iter().map(|l| l.delta).sum::<i64>(), 0);
        assert_eq!(
            lexed[0].delta, 0,
            "a brace inside a block comment is not a block open"
        );
        assert_eq!(
            lexed[3].delta, 0,
            "a brace inside a carried template literal is not a block open"
        );
        assert_eq!(
            lexed[5].delta, 0,
            "a brace after an unterminated quote is not a block open"
        );

        // And the span cut from those same deltas is the declaration's own.
        let symbols = parse("a.test.ts", source).expect("a valid file must parse");
        let test_symbol = symbols
            .iter()
            .find(|s| s.qualified_name.ends_with("holds"))
            .expect("the registration is a test symbol");
        assert_eq!(test_symbol.line, 7);
        assert_eq!(test_symbol.end_line, 9);
    }

    /// A bare `{` inside a multi-line template literal must not count as a
    /// block open: the carried-in content is dropped rather than copied, and
    /// since CR-039 the delta is counted in the same pass that drops it.
    /// Otherwise the literal unbalances the file and `check_balanced` rejects
    /// it, which is the same zero-symbol outcome by a different route.
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

        // And a single-line backtick title still survives stripping, or the
        // registration regex has nothing to read.
        let mut state = ScanState::default();
        let lexed = lex_line("test(`a title`, () => {", &mut state);
        assert!(lexed.code.contains("a title"));
        assert_eq!(lexed.delta, 1, "the trailing brace is code");
        assert!(!state.in_template);
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

        // Outside: whitespace before the `.` splits the chain off `it`.
        assert!(!registers("it .skip('split chain', () => {});"));
        // Outside: an empty modifier name.
        assert!(!registers("it.('empty modifier', () => {});"));
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
        // Verbatim shapes from `~/dev`. Neither is a registration, and the
        // second would be admitted by the `.modifier` chain if `context` were.
        assert!(containers("context.setTransform(dpr, 0, 0, dpr, 0, 0);\n").is_empty());
        assert!(containers("await context.client.putSettings(key, newData);\n").is_empty());

        // An identifier continuing past a suite name is not one, and a title
        // held in a variable registers nothing rather than something wrong —
        // the same boundary TC-948 pins for `it`/`test`.
        assert!(containers("describeAll(\"not a registration\", () => {});\n").is_empty());
        assert!(containers("describe(title, () => {});\n").is_empty());

        // What IS admitted, so the negative cases above are not vacuous.
        assert_eq!(
            containers("describe.each([1, 2])(\"a parametrised suite %i\", () => {});\n"),
            vec!["a parametrised suite %i".to_string()],
        );
    }
}
