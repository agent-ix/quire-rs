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

use std::sync::OnceLock;

use regex::Regex;

use super::{leading_block, RawSymbol, SymbolKind};

/// Parse `source` into raw symbols, or return a per-file reason to skip it.
pub(crate) fn parse(path: &str, source: &str) -> Result<Vec<RawSymbol>, String> {
    let lines: Vec<&str> = source.lines().collect();
    check_balanced(&lines)?;

    let module = module_name(path);
    let mut out = vec![RawSymbol {
        qualified_name: module.clone(),
        kind: SymbolKind::Container,
        line: 1,
        leading_line: 1,
        end_line: lines.len().max(1),
        container: None,
    }];

    let mut scopes: Vec<(String, i64)> = Vec::new();
    let mut depth: i64 = 0;
    let mut in_block_comment = false;

    for (idx, raw_line) in lines.iter().enumerate() {
        let line = strip_comment(raw_line, &mut in_block_comment);
        let trimmed = line.trim();

        if let Some(title) = registration(trimmed) {
            push(
                &mut out,
                &lines,
                idx,
                title,
                SymbolKind::TestFunction,
                scope_container(&scopes, &module),
            );
        } else if let Some(name) = class_declaration(trimmed) {
            let qualified = qualify(&scopes, &name);
            push(
                &mut out,
                &lines,
                idx,
                qualified.clone(),
                SymbolKind::Container,
                scope_container(&scopes, &module),
            );
            if trimmed.contains('{') {
                scopes.push((qualified, depth));
            }
        } else if let Some(name) = function_declaration(trimmed) {
            push(
                &mut out,
                &lines,
                idx,
                qualify(&scopes, &name),
                SymbolKind::Function,
                scope_container(&scopes, &module),
            );
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

fn push(
    out: &mut Vec<RawSymbol>,
    lines: &[&str],
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
        end_line: block_end(lines, idx),
        container,
    });
}

fn qualify(scopes: &[(String, i64)], name: &str) -> String {
    match scopes.last() {
        Some((prefix, _)) => format!("{prefix}.{name}"),
        None => name.to_string(),
    }
}

fn scope_container(scopes: &[(String, i64)], module: &str) -> Option<String> {
    scopes
        .last()
        .map(|(prefix, _)| prefix.clone())
        .or_else(|| Some(module.to_string()))
}

/// A `test('title', …)` / `it("title", …)` registration — the title is the
/// symbol name. `describe(…)` blocks group tests but register none themselves.
fn registration(trimmed: &str) -> Option<String> {
    let caps = re_registration().captures(trimmed)?;
    // Groups 2/3/4 are the single-, double-, and backtick-quoted title forms.
    (2..=4)
        .find_map(|g| caps.get(g))
        .map(|m| m.as_str().to_string())
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

fn block_end(lines: &[&str], decl_idx: usize) -> usize {
    let mut depth = 0i64;
    let mut seen_open = false;
    let mut in_block_comment = false;
    for (offset, raw) in lines[decl_idx..].iter().enumerate() {
        let line = strip_comment(raw, &mut in_block_comment);
        if line.contains('{') {
            seen_open = true;
        }
        depth += brace_delta(&line);
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
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for c in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match (quote, c) {
            (Some(_), '\\') => escaped = true,
            (Some(q), ch) if ch == q => quote = None,
            (Some(_), _) => {}
            (None, '"') | (None, '\'') | (None, '`') => quote = Some(c),
            (None, '{') => delta += 1,
            (None, '}') => delta -= 1,
            _ => {}
        }
    }
    delta
}

fn strip_comment(line: &str, in_block_comment: &mut bool) -> String {
    let mut out = String::with_capacity(line.len());
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if *in_block_comment {
            if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                *in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if chars[i] == '/' && chars.get(i + 1) == Some(&'/') {
            break;
        }
        if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
            *in_block_comment = true;
            i += 2;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn check_balanced(lines: &[&str]) -> Result<(), String> {
    let mut depth = 0i64;
    let mut in_block_comment = false;
    for line in lines {
        depth += brace_delta(&strip_comment(line, &mut in_block_comment));
        if depth < 0 {
            return Err("unbalanced braces: a `}` closes no block".to_string());
        }
    }
    if depth != 0 {
        return Err(format!("unbalanced braces: {depth} block(s) left open"));
    }
    Ok(())
}

fn re_registration() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r#"^(?:await\s+)?(test|it)(?:\.\w+)?\(\s*(?:'([^']*)'|"([^"]*)"|`([^`]*)`)"#)
            .expect("registration regex")
    })
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
