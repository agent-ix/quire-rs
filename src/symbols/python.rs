//! Python source adapter (FR-051).
//!
//! Indentation-structural: `class`/`def` declarations nest by column, which is
//! Python's actual block grammar, so no parser dependency is needed for the
//! symbol set FR-051 specifies. The file itself is a container symbol (the
//! module), named by its dotted path.
//!
//! Test classification is the pytest convention: a `test_`-prefixed function,
//! or a method of a `Test`-prefixed class whose own name is `test_`-prefixed.

use super::{leading_block, RawSymbol, SymbolKind};

/// Parse `source` into raw symbols, or return a per-file reason to skip it.
pub(crate) fn parse(path: &str, source: &str) -> Result<Vec<RawSymbol>, String> {
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

    // Scope stack of (qualified name, indent column of the declaration).
    let mut scopes: Vec<(String, usize, bool)> = Vec::new();
    let mut in_docstring: Option<&str> = None;

    for (idx, raw_line) in lines.iter().enumerate() {
        let trimmed = raw_line.trim();
        // Skip docstring/triple-quoted bodies so `def` inside them is not a
        // declaration.
        if let Some(delim) = in_docstring {
            if trimmed.contains(delim) {
                in_docstring = None;
            }
            continue;
        }
        for delim in ["\"\"\"", "'''"] {
            if trimmed.starts_with(delim) && trimmed[delim.len()..].matches(delim).count() == 0 {
                in_docstring = Some(delim);
                break;
            }
        }
        if in_docstring.is_some() {
            continue;
        }

        let Some((name, is_class)) = declaration(trimmed) else {
            continue;
        };
        let indent = raw_line.len() - raw_line.trim_start().len();
        while scopes.last().is_some_and(|(_, col, _)| indent <= *col) {
            scopes.pop();
        }
        let container = scopes
            .last()
            .map(|(qualified, _, _)| qualified.clone())
            .or_else(|| Some(module.clone()));
        let qualified_name = match scopes.last() {
            Some((prefix, _, _)) => format!("{prefix}.{name}"),
            None => name.clone(),
        };
        let in_test_class = scopes.last().is_some_and(|(_, _, is_test)| *is_test);
        let kind = if is_class {
            SymbolKind::Container
        } else if name.starts_with("test_") && (scopes.is_empty() || in_test_class) {
            // Module-level `test_*` functions, and `test_*` methods of a
            // `Test*` class — the pytest collection convention. A `test_*`
            // helper on a non-test class is not collected, so it is not a test.
            SymbolKind::TestFunction
        } else {
            SymbolKind::Function
        };
        out.push(RawSymbol {
            qualified_name: qualified_name.clone(),
            kind,
            line: idx + 1,
            leading_line: leading_block(&lines, idx, is_annotation),
            end_line: block_end(&lines, idx, indent),
            container,
        });
        if is_class {
            scopes.push((qualified_name, indent, name.starts_with("Test")));
        }
    }
    Ok(out)
}

/// A `def`/`async def`/`class` declaration and whether it is a class.
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

/// Decorators and comments form the annotation block.
fn is_annotation(line: &str) -> bool {
    line.starts_with('@') || line.starts_with('#')
}

/// The 1-based last line of a suite: the final line indented deeper than the
/// declaration (blank lines inside the suite are absorbed).
fn block_end(lines: &[&str], decl_idx: usize, indent: usize) -> usize {
    // A signature can span lines — black wraps any `def` over the line limit —
    // and its closing `) -> None:` dedents back to the declaration's own column.
    // The indent rule below reads that as the end of the block, so the span
    // stopped one line short of the docstring, which is exactly where the trace
    // tag lives (CR-037). Consume the signature first, by parenthesis depth.
    let mut head = decl_idx;
    let mut depth = paren_delta(lines[decl_idx]);
    while depth > 0 && head + 1 < lines.len() {
        head += 1;
        depth += paren_delta(lines[head]);
    }

    let mut end = head + 1;
    for (offset, line) in lines[head + 1..].iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let col = line.len() - line.trim_start().len();
        if col <= indent {
            break;
        }
        end = head + offset + 2;
    }
    end
}

/// Net parenthesis depth contributed by one line, ignoring quoted spans and a
/// trailing `#` comment — a default argument may hold either.
fn paren_delta(line: &str) -> i64 {
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
            (None, '"') | (None, '\'') => quote = Some(c),
            (None, '#') => break,
            (None, '(') => delta += 1,
            (None, ')') => delta -= 1,
            _ => {}
        }
    }
    delta
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
    use super::*;

    /// TC-800, FR-051-AC-13 (CR-037): a signature black wrapped still reaches
    /// its docstring.
    ///
    /// `def f(\n  arg,\n) -> None:` closes at the declaration's own column, and
    /// the indent rule read that dedent as the end of the suite — so the span
    /// stopped one line short of the docstring, which is exactly where the trace
    /// tag lives. Two identical tests differing only in signature wrapping bound
    /// differently, which is not a distinction any author intends.
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

    /// A paren inside a string default must not be counted, or the signature
    /// scan runs past the suite it belongs to.
    #[test]
    fn paren_depth_ignores_quotes_and_comments() {
        assert_eq!(paren_delta("def f(a: str = \"(\") -> None:"), 0);
        assert_eq!(paren_delta("def f(  # note (unbalanced"), 1);
        assert_eq!(paren_delta(") -> None:"), -1);
    }
}
