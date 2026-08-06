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
    let mut end = decl_idx + 1;
    for (offset, line) in lines[decl_idx + 1..].iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let col = line.len() - line.trim_start().len();
        if col <= indent {
            break;
        }
        end = decl_idx + offset + 2;
    }
    end
}

/// Dotted module name from a repo-relative path: `pkg/mod.py` → `pkg.mod`,
/// `pkg/__init__.py` → `pkg`.
fn module_name(path: &str) -> String {
    let stem = path.strip_suffix(".py").unwrap_or(path);
    let stem = stem.strip_suffix("/__init__").unwrap_or(stem);
    stem.replace('/', ".")
}
