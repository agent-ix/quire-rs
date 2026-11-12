//! The document line of a frontmatter key (FR-033-AC-18).
//!
//! The parsed frontmatter is a `serde_json::Value` with no positions, so the
//! line of a `frontmatter_field` locator's value is read back from the
//! authored YAML text: the line where the last key of the locator's `path`
//! begins, a flow-style (`{a: 1}`) parent included.

/// One content line of the frontmatter block.
struct Line<'a> {
    /// 1-based document line.
    number: usize,
    indent: usize,
    text: &'a str,
}

/// The 1-based document line where the last key of `path` begins in the
/// frontmatter of `raw`, or `None` when the text has no frontmatter block or
/// the key cannot be found in it (for example one produced by a YAML merge
/// key).
pub(super) fn key_line(raw: &str, path: &[String]) -> Option<usize> {
    let lines = block_lines(raw);
    let mut scope = Scope::Block {
        from: 0,
        to: lines.len(),
    };
    let mut found = None;
    for key in path {
        let (line, next) = find_key(&lines, scope, key)?;
        found = Some(line);
        scope = next;
    }
    found
}

/// Where the next path key is looked for.
#[derive(Clone, Copy)]
enum Scope {
    /// Block style: lines `from..to` of the block, at the first line's indent.
    Block { from: usize, to: usize },
    /// Flow style: from this line index and column, up to the closing brace.
    Flow { line: usize, col: usize },
}

/// The content lines between the opening and closing `---`, blank and comment
/// lines left out.
fn block_lines(raw: &str) -> Vec<Line<'_>> {
    let raw = raw.strip_prefix('\u{FEFF}').unwrap_or(raw);
    let mut all = raw.lines().enumerate();
    if !all
        .next()
        .is_some_and(|(_, first)| first.trim_end() == "---")
    {
        return Vec::new();
    }
    all.take_while(|(_, line)| !line.starts_with("---"))
        .filter_map(|(index, line)| {
            let text = line.trim_start();
            (!text.is_empty() && !text.starts_with('#')).then_some(Line {
                number: index + 1,
                indent: line.len() - text.len(),
                text,
            })
        })
        .collect()
}

/// Does `text` begin with the mapping key `key`? Returns the text after the
/// colon.
fn after_key<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let rest = ["", "\"", "'"].iter().find_map(|quote| {
        text.strip_prefix(quote)?
            .strip_prefix(key)?
            .strip_prefix(quote)
    })?;
    rest.trim_start().strip_prefix(':')
}

/// The line of `key` within `scope`, and the scope its value opens.
fn find_key(lines: &[Line<'_>], scope: Scope, key: &str) -> Option<(usize, Scope)> {
    match scope {
        Scope::Block { from, to } => {
            let level = lines.get(from)?.indent;
            let index = (from..to)
                .find(|&i| lines[i].indent == level && after_key(lines[i].text, key).is_some())?;
            let line = &lines[index];
            let value = after_key(line.text, key)?.trim_start();
            let next = match value.find('{').filter(|_| value.starts_with('{')) {
                Some(offset) => Scope::Flow {
                    line: index,
                    col: line.text.len() - value.len() + offset,
                },
                None => Scope::Block {
                    from: index + 1,
                    to: (index + 1..to)
                        .find(|&i| lines[i].indent <= line.indent)
                        .unwrap_or(to),
                },
            };
            Some((line.number, next))
        }
        Scope::Flow { line, col } => {
            let mut depth = 0i32;
            for (index, current) in lines.iter().enumerate().skip(line) {
                let text = current.text;
                let start = if index == line { col } else { 0 };
                // A key opening a continuation line follows the previous
                // line's comma.
                if index > line && depth == 1 {
                    if let Some(value) = after_key(text, key) {
                        let col = text.len() - value.trim_start().len();
                        return Some((current.number, Scope::Flow { line: index, col }));
                    }
                }
                for (offset, ch) in text.char_indices().skip_while(|(o, _)| *o < start) {
                    match ch {
                        '{' | '[' => depth += 1,
                        '}' | ']' => depth -= 1,
                        _ => {}
                    }
                    if depth <= 0 {
                        return None;
                    }
                    if depth == 1 && matches!(ch, '{' | ',') {
                        let candidate = text[offset + 1..].trim_start();
                        if let Some(value) = after_key(candidate, key) {
                            let value = value.trim_start();
                            let col = text.len() - value.len();
                            return Some((current.number, Scope::Flow { line: index, col }));
                        }
                    }
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(keys: &[&str]) -> Vec<String> {
        keys.iter().map(|k| k.to_string()).collect()
    }

    #[test]
    fn top_level_nested_and_flow_keys_resolve_to_their_own_line() {
        let raw = "---\nid: A\nmeta:\n  owner: x\n  deep:\n    leaf: 1\nflow: {a: 1,\n  b: 2}\ntags: [x, y]\n---\nbody\n";
        assert_eq!(key_line(raw, &path(&["id"])), Some(2));
        assert_eq!(key_line(raw, &path(&["meta"])), Some(3));
        assert_eq!(key_line(raw, &path(&["meta", "owner"])), Some(4));
        assert_eq!(key_line(raw, &path(&["meta", "deep", "leaf"])), Some(6));
        assert_eq!(key_line(raw, &path(&["flow", "a"])), Some(7));
        assert_eq!(key_line(raw, &path(&["flow", "b"])), Some(8));
        assert_eq!(key_line(raw, &path(&["tags"])), Some(9));
        assert_eq!(key_line(raw, &path(&["nope"])), None);
    }

    #[test]
    fn no_frontmatter_has_no_key_line() {
        assert_eq!(key_line("# heading\nid: A\n", &path(&["id"])), None);
    }

    #[test]
    fn a_bom_does_not_shift_the_line() {
        assert_eq!(
            key_line("\u{FEFF}---\nid: A\n---\n", &path(&["id"])),
            Some(2)
        );
    }
}
