//! Python source adapter (FR-051).
//!
//! Indentation-structural: `class`/`def` declarations nest by column, which is
//! Python's actual block grammar, so no parser dependency is needed for the
//! symbol set FR-051 specifies. The file itself is a container symbol (the
//! module), named by its dotted path.
//!
//! Test classification is the pytest convention: a `test_`-prefixed function,
//! or a method of a `Test`-prefixed class whose own name is `test_`-prefixed.
//! Direct imported unittest.TestCase bases additionally identify test classes
//! without a naming convention (FR-051-AC-3, #407).

use std::collections::BTreeSet;

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
    let mut scopes: Vec<(String, usize, TestClass)> = Vec::new();
    let mut quoting = Quoting::Code;
    let mut unittest = UnittestImports::default();

    for (idx, raw_line) in lines.iter().enumerate() {
        // A line that *starts* inside a triple-quoted string is string body, not
        // code: whatever `class`/`def` it holds belongs to the literal. Advance
        // the state across the whole line first — the closer may sit anywhere on
        // it, and a further opener may follow the closer (CR-115).
        let was_string = quoting.is_string();
        quoting = scan_line(raw_line, quoting);
        if was_string {
            continue;
        }

        let trimmed = raw_line.trim();
        let indent = raw_line.len() - raw_line.trim_start().len();
        let is_unittest_class =
            indent == 0 && trimmed.starts_with("class ") && unittest.is_test_case(trimmed);
        if indent == 0 {
            unittest.observe_binding(trimmed);
        }
        let Some((name, is_class)) = declaration(trimmed) else {
            continue;
        };
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
        let in_test_class = scopes.last().is_some_and(|(_, _, class)| match class {
            TestClass::Pytest => true,
            TestClass::Unittest(body_indent) => *body_indent == Some(indent),
            TestClass::None => false,
        });
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
            let class = if is_unittest_class {
                // Only direct methods are unittest evidence, never a local
                // test_-named helper nested inside a method's body.
                let body_indent = lines[idx + 1..]
                    .iter()
                    .find(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
                    .map(|line| line.len() - line.trim_start().len());
                TestClass::Unittest(body_indent)
            } else if name.starts_with("Test") {
                TestClass::Pytest
            } else {
                TestClass::None
            };
            scopes.push((qualified_name, indent, class));
        }
    }
    Ok(out)
}

enum TestClass {
    None,
    Pytest,
    Unittest(Option<usize>),
}

/// Bounded lexical identities, not dependency/type resolution. Only preceding
/// module-level imports authorize a unittest base; a lookalike spelling or an
/// explicitly rebound import must not create evidence (#407).
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

/// Whether a physical line boundary falls inside a triple-quoted string, and if
/// so which delimiter closes it.
///
/// Only the *triple* forms carry across lines, which is why a single-quoted span
/// is not a variant: it is consumed within [`scan_line`] and never observed at a
/// line boundary.
///
/// A single-quoted string continued with a trailing backslash is legal Python
/// and IS observed at a line boundary — `x = 'abc\` … `'`. The scan drops back
/// to [`Quoting::Code`] there and reads the continuation as code, which can
/// mint a declaration from string body. That is a known residual, not a claim
/// that the case cannot arise: measured with `tokenize` over 3,689 real `.py`
/// files, **zero** carry a multi-line non-triple string. The old scanner had
/// the identical behaviour, so this is neither introduced nor fixed here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Quoting {
    Code,
    /// Inside `"""…"""` (`b'"'`) or `'''…'''` (`b'\''`).
    Triple(u8),
}

impl Quoting {
    const fn is_string(self) -> bool {
        matches!(self, Self::Triple(_))
    }
}

/// Advance the quoting state across one physical line, returning the state that
/// leaves it.
///
/// One left-to-right pass, no allocation, no lookbehind — this runs on every
/// line of every Python file in the walk, so it stays in the same complexity
/// class as the `starts_with` test it replaces (NFR-001/NFR-015).
///
/// Byte indexing is safe for the same reason it is fast: every marker consulted
/// (`"`, `'`, `#`, `\`) is ASCII, and a UTF-8 continuation byte is never < 0x80,
/// so a multi-byte character can neither match a marker nor be split — the scan
/// only ever reports a state, it never slices.
fn scan_line(line: &str, entering: Quoting) -> Quoting {
    let bytes = line.as_bytes();
    let mut state = entering;
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        match state {
            Quoting::Triple(delim) => {
                // A backslash escapes the next byte even in a raw string: `r"\""`
                // is a two-character value, not a terminated one. Skipping the
                // pair is therefore right for both prefixes.
                if c == b'\\' {
                    i += 2;
                } else if c == delim && is_triple(bytes, i) {
                    state = Quoting::Code;
                    i += 3;
                } else {
                    i += 1;
                }
            }
            Quoting::Code => match c {
                // A `#` outside a string starts a comment: nothing after it on
                // this line can open or close anything.
                b'#' => return Quoting::Code,
                b'"' | b'\'' if is_triple(bytes, i) => {
                    state = Quoting::Triple(c);
                    i += 3;
                }
                // A prefix (`f`, `r`, `rb`, `u`, …) needs no special case: it is
                // ordinary identifier text the scan walks straight past, and the
                // quote that follows is what opens the string.
                b'"' | b'\'' => i = single_quoted_end(bytes, i),
                _ => i += 1,
            },
        }
    }
    state
}

/// Whether three identical quote bytes start at `i`.
fn is_triple(bytes: &[u8], i: usize) -> bool {
    bytes.len() - i >= 3 && bytes[i + 1] == bytes[i] && bytes[i + 2] == bytes[i]
}

/// The index just past a single-quoted string opening at `i`, or the end of the
/// line if it is not closed on it.
fn single_quoted_end(bytes: &[u8], i: usize) -> usize {
    let delim = bytes[i];
    let mut j = i + 1;
    while j < bytes.len() {
        match bytes[j] {
            b'\\' => j += 2,
            c if c == delim => return j + 1,
            _ => j += 1,
        }
    }
    bytes.len()
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

    #[trace("TC-1029", "FR-051-AC-20")]
    // a triple-quoted string is tracked wherever it opens,
    // in either delimiter, under any prefix, and open-and-closed on one line.
    //
    // The adapter entered string state only when the delimiter STARTED the
    // trimmed line, so `FIXTURE = """` never entered it and the string body was
    // read as code — and then the CLOSING `"""`, which does start its line, was
    // read as an OPENER, swallowing the rest of the file (#274).
    #[test]
    fn tc1029_a_triple_quote_is_tracked_wherever_it_opens() {
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

        // Nothing inside a literal is a declaration.
        for phantom in [
            "Phantom",
            "Phantom.method",
            "raw_phantom",
            "f_phantom",
            "rb_phantom",
            "inline_phantom",
        ] {
            assert!(
                !names(&symbols).contains(&phantom),
                "{phantom} is string content, not a declaration: {:?}",
                names(&symbols)
            );
        }

        // And every closer closed rather than re-opened, so the real test after
        // them all is still seen. `ONE_LINE` is the same-line open-and-close: had
        // it toggled the state, this declaration would have been swallowed.
        assert_eq!(
            symbol(&symbols, "test_after_every_string").kind,
            SymbolKind::TestFunction
        );
    }

    #[trace("TC-1030", "FR-051-AC-20")]
    // a triple delimiter inside a single-quoted string, after
    // a `#`, or escaped, toggles nothing — and a `#` inside a triple-quoted
    // string does not end it.
    //
    // The discriminator is the declaration at the end: any spurious toggle
    // leaves an unterminated string that swallows it.
    #[test]
    fn tc1030_a_delimiter_in_a_string_or_a_comment_does_not_toggle() {
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
            !names(&symbols).contains(&"phantom_in_doc"),
            "the literal's body is not code: {:?}",
            names(&symbols)
        );
        assert_eq!(
            symbol(&symbols, "test_state_never_toggled").kind,
            SymbolKind::TestFunction
        );

        // The state machine itself, so a passing assertion above cannot be an
        // accident of two errors cancelling.
        for line in [
            "SINGLE = \"a ''' inside a double-quoted string\"",
            "OTHER = 'a \"\"\" inside a single-quoted string'",
            "ESCAPED = \"she said \\\"\\\"\\\" loudly\"",
            "# a bare \"\"\" in a comment",
            "TRAILING = 1  # and ''' after code",
            "INLINE = \"\"\"opened and closed\"\"\"",
        ] {
            assert_eq!(scan_line(line, Quoting::Code), Quoting::Code, "{line}");
        }
        assert_eq!(
            scan_line("DOC = \"\"\"", Quoting::Code),
            Quoting::Triple(b'"')
        );
        assert_eq!(
            scan_line("RAW = r'''", Quoting::Code),
            Quoting::Triple(b'\'')
        );
        // A mismatched delimiter never closes, and the closer may sit anywhere.
        assert_eq!(
            scan_line("'''", Quoting::Triple(b'"')),
            Quoting::Triple(b'"')
        );
        assert_eq!(
            scan_line("end\"\"\" + x", Quoting::Triple(b'"')),
            Quoting::Code
        );
    }

    #[trace("TC-1031", "FR-051-AC-20")]
    // a declaration after an embedded string keeps its true
    // container: the scope stack does not resume stale.
    //
    // This is the dominant mode by count and the one a partial fix would miss.
    // Measured on `py-project/tests/test_deps.py`, the adapter reported
    // `test_update_simple_version` — which really lives in
    // `TestTomlModification` — under `TestTomlParsing`, and saw 10 of that
    // file's 21 classes (#274).
    #[test]
    fn tc1031_the_scope_stack_survives_an_embedded_string() {
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

        // The second class is seen at all — the closing quote did not swallow it.
        assert_eq!(
            symbol(&symbols, "TestModification").kind,
            SymbolKind::Container
        );

        // And it owns its own method, rather than the method resuming under the
        // class that was open when the string started.
        let method = symbol(&symbols, "TestModification.test_writes_a_fixture");
        assert_eq!(method.kind, SymbolKind::TestFunction);
        assert_eq!(method.container.as_deref(), Some("TestModification"));

        let first = symbol(&symbols, "TestParsing.test_reads_a_fixture");
        assert_eq!(first.container.as_deref(), Some("TestParsing"));
    }

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
