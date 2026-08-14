//! Rust source adapter (FR-051).
//!
//! Line-structural: it tracks `mod`/`impl`/type scopes by brace depth and
//! recognises declarations by their leading keyword. That is enough for the
//! symbol set FR-051 specifies (functions, test functions, containers) without
//! a parser dependency, a build, or any type resolution.
//!
//! Test classification is the `#[test]` **family**: any attribute in the
//! declaration's annotation block whose path ends in `test` — `#[test]`,
//! `#[tokio::test]`, `#[rstest]`, `#[wasm_bindgen_test]`.

use super::{leading_block, RawSymbol, SymbolKind};

/// Parse `source` into raw symbols, or return a per-file reason to skip it.
pub(crate) fn parse(source: &str) -> Result<Vec<RawSymbol>, String> {
    let lines: Vec<&str> = source.lines().collect();
    // One lexer pass for the whole file (CR-040); every consumer reads it.
    let lexed = lex(&lines);
    check_balanced(&lexed)?;

    let mut out: Vec<RawSymbol> = Vec::new();
    // Scope stack: (qualified prefix, brace depth at which the scope opened).
    let mut scopes: Vec<(String, i64)> = Vec::new();
    let mut depth: i64 = 0;

    for (idx, lexed_line) in lexed.iter().enumerate() {
        let trimmed = lexed_line.code.trim();
        let opens = trimmed.contains('{');

        if let Some((name, kind)) = declaration(trimmed) {
            let container = scopes.last().map(|(prefix, _)| prefix.clone());
            let qualified_name = match &container {
                Some(prefix) => format!("{prefix}::{name}"),
                None => name.clone(),
            };
            let kind = match kind {
                DeclKind::Function => {
                    if is_test(&lines, idx) {
                        SymbolKind::TestFunction
                    } else {
                        SymbolKind::Function
                    }
                }
                DeclKind::Container => SymbolKind::Container,
                DeclKind::Scope => SymbolKind::Container,
            };
            let leading_line = leading_block(&lines, idx, is_annotation);
            let end_line = block_end(&lexed, idx);
            // `impl` blocks are scopes, not symbols: their methods qualify
            // under the type name, but the block itself declares nothing new.
            if !trimmed.starts_with("impl") {
                out.push(RawSymbol {
                    qualified_name: qualified_name.clone(),
                    kind,
                    line: idx + 1,
                    leading_line,
                    end_line,
                    container,
                });
            }
            // A declaration that opens a brace becomes the enclosing scope for
            // what follows (`mod`, `impl`, `trait`, and inherent `struct`
            // bodies all nest this way).
            if opens && matches!(kind, SymbolKind::Container) {
                scopes.push((qualified_name, depth));
            }
        }

        depth += lexed_line.delta;
        while let Some((_, opened_at)) = scopes.last() {
            if depth <= *opened_at {
                scopes.pop();
            } else {
                break;
            }
        }
    }
    Ok(out)
}

enum DeclKind {
    Function,
    Container,
    Scope,
}

/// Recognise a declaration line: `fn`, `mod`, `struct`, `enum`, `trait`, or an
/// `impl` scope. Visibility, `async`, `unsafe`, `const`, and generics are
/// skipped over rather than parsed.
fn declaration(trimmed: &str) -> Option<(String, DeclKind)> {
    if trimmed.starts_with("impl") {
        // `impl Foo`, `impl<T> Foo<T>`, `impl Trait for Foo`.
        let rest = trimmed.trim_start_matches("impl").trim_start();
        let rest = strip_generics(rest);
        let target = match rest.split(" for ").nth(1) {
            Some(after_for) => after_for,
            None => rest,
        };
        let name = ident(strip_generics(target.trim()))?;
        return Some((name, DeclKind::Scope));
    }
    let mut rest = trimmed;
    for modifier in [
        "pub(crate) ",
        "pub(super) ",
        "pub ",
        "default ",
        "async ",
        "unsafe ",
        "const ",
        "extern ",
    ] {
        while let Some(stripped) = rest.strip_prefix(modifier) {
            rest = stripped.trim_start();
        }
    }
    for (keyword, kind) in [
        ("fn ", DeclKind::Function),
        ("mod ", DeclKind::Container),
        ("struct ", DeclKind::Container),
        ("enum ", DeclKind::Container),
        ("trait ", DeclKind::Container),
    ] {
        if let Some(after) = rest.strip_prefix(keyword) {
            let name = ident(after.trim_start())?;
            return Some((name, kind));
        }
    }
    None
}

/// The leading identifier of `s`, or `None` when it does not start with one.
fn ident(s: &str) -> Option<String> {
    let name: String = s
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// Drop a leading generic parameter list (`<T: Bound>`).
fn strip_generics(s: &str) -> &str {
    let s = s.trim_start();
    if !s.starts_with('<') {
        return s;
    }
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return s[i + 1..].trim_start();
                }
            }
            _ => {}
        }
    }
    s
}

/// True when `line` is part of an annotation block: an attribute or a comment.
fn is_annotation(line: &str) -> bool {
    line.starts_with("#[") || line.starts_with("#![") || line.starts_with("//")
}

/// The `#[test]` family: any attribute whose path segment ends in `test`.
fn is_test(lines: &[&str], decl_idx: usize) -> bool {
    let start = leading_block(lines, decl_idx, is_annotation).saturating_sub(1);
    lines[start..decl_idx].iter().any(|l| {
        let t = l.trim();
        let Some(inner) = t.strip_prefix("#[").and_then(|s| s.strip_suffix(']')) else {
            return false;
        };
        let path = inner.split(['(', ',']).next().unwrap_or("").trim();
        path.rsplit("::").next().is_some_and(|seg| seg == "test")
    })
}

/// The 1-based last line of the block opened at `decl_idx`. A declaration with
/// no brace (a unit struct, a trait method signature) ends on its own line.
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

/// One lexed line: the code with comments and carried string content removed,
/// and the brace delta counted **in the same pass** (CR-040).
///
/// The three derivations this replaces each had their own idea of what a string
/// was, and Rust gives that idea four ways to be wrong: raw strings (`r#"…"#`,
/// where a `"` is content and a `\` escapes nothing), lifetimes (`&'a str`,
/// where `'` opens no char literal), multi-line strings, and a `//` or `/*`
/// inside any of them.
#[derive(Debug, Default, Clone)]
struct LexedLine {
    code: String,
    delta: i64,
}

/// Comment and string state that survives from one line to the next.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct ScanState {
    /// Rust block comments **nest**, so this is a depth, not a flag.
    block_comment_depth: u32,
    in_string: bool,
    /// `Some(n)` inside a raw string opened with `n` hashes (`r##"` → 2).
    raw_hashes: Option<usize>,
}

/// Lex the file once, carrying comment and string state line to line.
fn lex(lines: &[&str]) -> Vec<LexedLine> {
    let mut state = ScanState::default();
    lines
        .iter()
        .map(|line| lex_line(line, &mut state))
        .collect()
}

/// True when `'` at `i` opens a character literal rather than a lifetime.
///
/// `'a'`, `'\n'` and `'\''` are literals; `&'a str`, `'static` and `Foo<'_>`
/// are not. The discriminator is a closing quote in the only two positions a
/// literal can put one — which is decidable without parsing, because a lifetime
/// is never followed by a quote that soon.
fn opens_char_literal(chars: &[char], i: usize) -> bool {
    match chars.get(i + 1) {
        Some('\\') => {
            // `'\n'`, `'\''`, `'\u{1F600}'` — scan to the closing quote.
            let mut j = i + 2;
            while j < chars.len() && j <= i + 12 {
                if chars[j] == '\'' {
                    return true;
                }
                j += 1;
            }
            false
        }
        Some(_) => chars.get(i + 2) == Some(&'\''),
        None => false,
    }
}

/// Drop comments and string content from one line, and count the braces that
/// are actually code.
///
/// String **content is dropped rather than copied**: unlike the TypeScript
/// adapter, no declaration form here reads a literal's text, so keeping it
/// would only risk a brace inside it being counted twice.
fn lex_line(line: &str, state: &mut ScanState) -> LexedLine {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut delta = 0i64;
    let mut i = 0;

    while i < chars.len() {
        // ── carried raw string ──
        if let Some(hashes) = state.raw_hashes {
            if chars[i] == '"' && closes_raw(&chars, i + 1, hashes) {
                state.raw_hashes = None;
                i += 1 + hashes;
            } else {
                i += 1;
            }
            continue;
        }
        // ── carried normal string ──
        if state.in_string {
            if chars[i] == '\\' {
                i += 2;
                continue;
            }
            if chars[i] == '"' {
                state.in_string = false;
            }
            i += 1;
            continue;
        }
        // ── carried block comment (nesting) ──
        if state.block_comment_depth > 0 {
            if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                state.block_comment_depth += 1;
                i += 2;
                continue;
            }
            if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                state.block_comment_depth -= 1;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        // ── code ──
        if chars[i] == '/' && chars.get(i + 1) == Some(&'/') {
            break;
        }
        if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
            state.block_comment_depth = 1;
            i += 2;
            continue;
        }
        // `r"…"`, `r#"…"#`, `br#"…"#` — a `\` escapes nothing inside.
        if let Some(hashes) = opens_raw_string(&chars, i) {
            let quote = i + raw_prefix_len(&chars, i);
            state.raw_hashes = Some(hashes);
            i = quote + 1;
            continue;
        }
        if chars[i] == '"' {
            state.in_string = true;
            i += 1;
            continue;
        }
        if chars[i] == '\'' {
            if opens_char_literal(&chars, i) {
                // Skip the whole literal; a `{` inside it is content.
                let mut j = i + 1;
                while j < chars.len() {
                    if chars[j] == '\\' {
                        j += 2;
                        continue;
                    }
                    if chars[j] == '\'' {
                        break;
                    }
                    j += 1;
                }
                i = j + 1;
                continue;
            }
            // A lifetime: ordinary code, and it opens nothing.
            out.push(chars[i]);
            i += 1;
            continue;
        }
        match chars[i] {
            '{' => delta += 1,
            '}' => delta -= 1,
            _ => {}
        }
        out.push(chars[i]);
        i += 1;
    }
    LexedLine { code: out, delta }
}

/// Hash count when a raw-string prefix (`r`, `br`, `r#`, `br##`, …) starts at
/// `i`, or `None`.
fn opens_raw_string(chars: &[char], i: usize) -> Option<usize> {
    let mut j = i;
    if chars.get(j) == Some(&'b') {
        j += 1;
    }
    if chars.get(j) != Some(&'r') {
        return None;
    }
    // A raw-string prefix is a token: `for` must not read as `r` + `#`/`"`.
    if i > 0 {
        let prev = chars[i - 1];
        if prev.is_alphanumeric() || prev == '_' {
            return None;
        }
    }
    j += 1;
    let mut hashes = 0;
    while chars.get(j) == Some(&'#') {
        hashes += 1;
        j += 1;
    }
    (chars.get(j) == Some(&'"')).then_some(hashes)
}

/// Character offset from the prefix start to its opening quote.
fn raw_prefix_len(chars: &[char], i: usize) -> usize {
    let mut j = i;
    if chars.get(j) == Some(&'b') {
        j += 1;
    }
    j += 1; // `r`
    while chars.get(j) == Some(&'#') {
        j += 1;
    }
    j - i
}

/// True when `hashes` `#` characters follow the closing quote at `from`.
fn closes_raw(chars: &[char], from: usize, hashes: usize) -> bool {
    (0..hashes).all(|k| chars.get(from + k) == Some(&'#'))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-804 (FR-051-AC-15, CR-040): a file is not rejected for braces that
    /// live in a raw string, a lifetime, a char literal or a nested comment.
    ///
    /// Measured before the fix: **33 of quire-rs's own source files** — every
    /// one holding a `r#"…"#` JSON fixture — were rejected as unbalanced and
    /// yielded zero symbols, so every trace tag in them bound to nothing. That
    /// alone accounted for 78 of the repo's 140 reported "status lies": the
    /// matrix was not overclaiming, the adapter could not see the tests.
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

    /// The lexer's own discriminations, asserted directly.
    #[test]
    fn tc804_lexer_counts_only_code_braces() {
        let cases: &[(&str, i64)] = &[
            ("fn f() {", 1),
            ("}", -1),
            // A brace inside a raw string is content, and `\` escapes nothing.
            ("let s = r#\"{ \\ }\"#;", 0),
            ("let s = r\"{}\";", 0),
            ("let s = b r\"x\";", 0),
            // …and inside an ordinary string.
            ("let s = \"{ }\";", 0),
            // A char literal holding a brace.
            ("let c = '{';", 0),
            // A lifetime is not a char literal: the braces around it are code.
            ("impl<'a> Foo<'a> { fn g(&'a self) {} }", 0),
            ("fn h<'a>(x: &'a str) -> &'a str {", 1),
            // Comments, including the nesting Rust allows.
            ("let x = 1; // {", 0),
            ("/* { */ fn k() {", 1),
            ("/* /* { */ */", 0),
        ];
        for (line, want) in cases {
            let mut state = ScanState::default();
            let lexed = lex_line(line, &mut state);
            assert_eq!(lexed.delta, *want, "delta for {line:?}: {lexed:?}");
        }
    }

    /// A string or raw string left open carries to the next line, and a `//`
    /// inside it is content rather than a comment opener.
    #[test]
    fn tc804_string_state_carries_across_lines() {
        let mut state = ScanState::default();
        lex_line("let s = r#\"", &mut state);
        assert_eq!(state.raw_hashes, Some(1), "the raw string carries");
        let inside = lex_line("a url // not a comment, and a brace {", &mut state);
        assert_eq!(inside.delta, 0, "everything here is literal content");
        lex_line("\"#;", &mut state);
        assert_eq!(state.raw_hashes, None, "the matching hash count closes it");

        let mut state = ScanState::default();
        lex_line("let s = \"open", &mut state);
        assert!(state.in_string);
        lex_line("still open {", &mut state);
        assert!(state.in_string);
        let closed = lex_line("done\"; fn f() {", &mut state);
        assert!(!state.in_string);
        assert_eq!(closed.delta, 1, "code after the closer counts again");
    }
}
