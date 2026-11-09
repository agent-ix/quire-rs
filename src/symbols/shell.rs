//! Shell source adapter (FR-051 shell adapter, CR-213, PLAT-362).
//!
//! Parses a `.sh` file with the `tree-sitter-bash` grammar through the
//! dependency-boundary crate `quire-code-extraction` (the `shell-symbols`
//! feature forwards to `quire-code-parse`'s `bash` feature), and reads every
//! symbol, span and comment from the tree's nodes — never from line text. It
//! never executes or sources the script it reads (FR-051-CON-1).
//!
//! Two kinds of symbol, both read without consulting module data so the
//! candidate population is the same with or without a module (AC-23):
//!
//! - a [`SymbolKind::Function`] for every `function_definition` node outside a
//!   command substitution (`$( )`, backticks: a subshell's function is not a
//!   global one, and FR-051-AC-36 lists them as decoys), whatever its spelling. Shell functions are global, so the qualified name is the
//!   function's own name even when the definition sits inside another
//!   function's body; the enclosing function is only its `container`.
//! - one [`SymbolKind::EntryPoint`] per file, named `entry_point` and spanning
//!   the whole file (the Rust `fuzz_target` precedent). Whether it may bind is
//!   the binding stage's question (`entry_paths`), not this file's.
//!
//! **The tag channel is the comment node.** Shell has no attribute syntax, so
//! each symbol carries the `#` comment nodes of its channel in
//! [`RawSymbol::tag_comments`] (AC-40): the contiguous own-line comment block
//! immediately above a function (a blank line ends it), and the first run of
//! consecutive own-line comments of the file's header — before the first
//! command, after any `#!` line and blank lines — for the entry point. A `#`
//! inside a string, a heredoc, `$#`, `${#x}`, `${x#pat}` or a word is never a
//! comment node, so none of them is ever a tag (AC-41).

use quire_code_extraction::tree_sitter::Node;
use quire_code_extraction::{parse_file, Language};

use super::{RawSymbol, SourceAttribute, SymbolKind};

/// What one `.sh` file yields.
pub(crate) struct Parsed {
    pub symbols: Vec<RawSymbol>,
    /// Every comment node of the file, in source order (AC-41, AC-44).
    pub comments: Vec<SourceAttribute>,
}

/// A comment node and whether it is the first thing on its line.
struct Comment {
    attribute: SourceAttribute,
    own_line: bool,
}

struct Function {
    name: String,
    line: usize,
    column: usize,
    end_line: usize,
    container: Option<String>,
}

pub(crate) fn parse(source: &str) -> Result<Parsed, String> {
    let parsed =
        parse_file(Language::Bash, "<source>", source).map_err(|e| format!("parse: {e}"))?;
    // The grammar yields a tree for every input; a file is unparseable only
    // where the tree carries an error over its declaration structure, and the
    // diagnostic names the line the error sits at (FR-051-AC-45).
    if let Some(diagnostic) = parsed.diagnostic() {
        return Err(format!(
            "line {}: unresolvable declaration structure (column {})",
            diagnostic.line(),
            diagnostic.column()
        ));
    }
    let root = parsed.root_node();
    let line_starts = line_starts(source);

    let mut functions: Vec<Function> = Vec::new();
    let mut comments: Vec<Comment> = Vec::new();
    // Open function bodies, innermost last: `(end byte, name)`.
    let mut enclosing: Vec<(usize, String)> = Vec::new();
    walk(
        root,
        source,
        &line_starts,
        &mut enclosing,
        &mut functions,
        &mut comments,
    );

    let line_count = source.lines().count().max(1);
    let header = header_comments(root, &comments);
    let mut symbols = vec![RawSymbol {
        attributes: Vec::new(),
        qualified_name: "entry_point".to_string(),
        kind: SymbolKind::EntryPoint,
        line: 1,
        column: 1,
        leading_line: 1,
        end_line: line_count,
        container: None,
        tag_comments: header,
        ignored: false,
    }];
    for function in functions {
        let block = block_above(&comments, function.line);
        symbols.push(RawSymbol {
            attributes: Vec::new(),
            qualified_name: function.name,
            kind: SymbolKind::Function,
            line: function.line,
            column: function.column,
            leading_line: block.first().map_or(function.line, |c| c.line),
            end_line: function.end_line,
            container: function.container,
            tag_comments: block,
            ignored: false,
        });
    }
    Ok(Parsed {
        symbols,
        comments: comments.into_iter().map(|c| c.attribute).collect(),
    })
}

/// Byte offset of every line start.
fn line_starts(source: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(source.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

/// Pre-order walk with an explicit cursor, so a deeply nested script cannot
/// exhaust the stack.
fn walk(
    root: Node<'_>,
    source: &str,
    line_starts: &[usize],
    enclosing: &mut Vec<(usize, String)>,
    functions: &mut Vec<Function>,
    comments: &mut Vec<Comment>,
) {
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        while enclosing
            .last()
            .is_some_and(|(end, _)| *end <= node.start_byte())
        {
            enclosing.pop();
        }
        match node.kind() {
            "comment" => {
                let start = node.start_position();
                let text = source[node.byte_range()].trim_end_matches('\r');
                let before = &source[line_starts[start.row]..node.start_byte()];
                comments.push(Comment {
                    attribute: SourceAttribute {
                        text: text.to_string(),
                        line: start.row + 1,
                    },
                    own_line: before.trim().is_empty(),
                });
            }
            // A definition inside a command substitution (`$( )` or backticks)
            // lives in a subshell and is not a global function; FR-051-AC-36
            // lists the substitution among the places a definition-shaped line
            // mints nothing.
            "function_definition" if !in_command_substitution(node) => {
                if let Some(name) = node
                    .child_by_field_name("name")
                    .map(|n| source[n.byte_range()].to_string())
                {
                    let start = node.start_position();
                    let end = node.end_position();
                    // A node ending at column 0 ends on the previous line.
                    let end_line = if end.column == 0 && end.row > start.row {
                        end.row
                    } else {
                        end.row + 1
                    };
                    functions.push(Function {
                        container: enclosing.last().map(|(_, n)| n.clone()),
                        name: name.clone(),
                        line: start.row + 1,
                        column: start.column + 1,
                        end_line,
                    });
                    enclosing.push((node.end_byte(), name));
                }
            }
            _ => {}
        }
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return;
            }
        }
    }
}

fn in_command_substitution(node: Node<'_>) -> bool {
    std::iter::successors(node.parent(), |n| n.parent()).any(|n| n.kind() == "command_substitution")
}

/// The contiguous own-line comment lines ending on the line just above `line`.
fn block_above(comments: &[Comment], line: usize) -> Vec<SourceAttribute> {
    let mut block: Vec<SourceAttribute> = Vec::new();
    let mut expected = line.saturating_sub(1);
    for comment in comments.iter().rev() {
        if comment.attribute.line > expected || !comment.own_line {
            continue;
        }
        if comment.attribute.line != expected || expected == 0 {
            break;
        }
        block.push(comment.attribute.clone());
        expected -= 1;
    }
    block.reverse();
    block
}

/// The file's header block: the first run of consecutive own-line comments
/// that precedes the first command, skipping a `#!` line (AC-40).
fn header_comments(root: Node<'_>, comments: &[Comment]) -> Vec<SourceAttribute> {
    let mut cursor = root.walk();
    let first_command_line = root
        .named_children(&mut cursor)
        .find(|n| n.kind() != "comment")
        .map_or(usize::MAX, |n| n.start_position().row + 1);
    let mut run: Vec<SourceAttribute> = Vec::new();
    for comment in comments {
        let attribute = &comment.attribute;
        if attribute.line >= first_command_line {
            break;
        }
        if !comment.own_line || (attribute.line == 1 && attribute.text.starts_with("#!")) {
            continue;
        }
        match run.last() {
            Some(last) if last.line + 1 != attribute.line => break,
            _ => run.push(attribute.clone()),
        }
    }
    run
}
