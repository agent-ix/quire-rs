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
    check_balanced(&lines)?;

    let mut out: Vec<RawSymbol> = Vec::new();
    // Scope stack: (qualified prefix, brace depth at which the scope opened).
    let mut scopes: Vec<(String, i64)> = Vec::new();
    let mut depth: i64 = 0;
    let mut in_block_comment = false;

    for (idx, raw_line) in lines.iter().enumerate() {
        let line = strip_comment(raw_line, &mut in_block_comment);
        let trimmed = line.trim();
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
            let end_line = block_end(&lines, idx);
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

        depth += brace_delta(&line);
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
fn block_end(lines: &[&str], decl_idx: usize) -> usize {
    let mut depth = 0i64;
    let mut seen_open = false;
    let mut in_block_comment = false;
    for (offset, raw) in lines[decl_idx..].iter().enumerate() {
        let line = strip_comment(raw, &mut in_block_comment);
        let delta = brace_delta(&line);
        if line.contains('{') {
            seen_open = true;
        }
        depth += delta;
        if seen_open && depth <= 0 {
            return decl_idx + offset + 1;
        }
        if !seen_open && line.trim_end().ends_with(';') {
            return decl_idx + offset + 1;
        }
    }
    lines.len().max(decl_idx + 1)
}

fn brace_delta(line: &str) -> i64 {
    let mut delta = 0i64;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    for c in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' if in_string || in_char => escaped = true,
            '"' if !in_char => in_string = !in_string,
            '\'' if !in_string => in_char = !in_char,
            '{' if !in_string && !in_char => delta += 1,
            '}' if !in_string && !in_char => delta -= 1,
            _ => {}
        }
    }
    delta
}

/// Strip line and block comments so braces inside them never move the depth.
fn strip_comment(line: &str, in_block_comment: &mut bool) -> String {
    let mut out = String::with_capacity(line.len());
    let bytes: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if *in_block_comment {
            if bytes[i] == '*' && bytes.get(i + 1) == Some(&'/') {
                *in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if bytes[i] == '/' && bytes.get(i + 1) == Some(&'/') {
            break;
        }
        if bytes[i] == '/' && bytes.get(i + 1) == Some(&'*') {
            *in_block_comment = true;
            i += 2;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

/// Reject a file whose braces do not balance — the adapter's notion of
/// "cannot parse" (FR-051-AC-9). Comments and string literals are excluded.
fn check_balanced(lines: &[&str]) -> Result<(), String> {
    let mut depth = 0i64;
    let mut in_block_comment = false;
    for line in lines {
        let stripped = strip_comment(line, &mut in_block_comment);
        depth += brace_delta(&stripped);
        if depth < 0 {
            return Err("unbalanced braces: a `}` closes no block".to_string());
        }
    }
    if depth != 0 {
        return Err(format!("unbalanced braces: {depth} block(s) left open"));
    }
    Ok(())
}
