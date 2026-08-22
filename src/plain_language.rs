//! Reader-visible prose and project-owned plain-language checks (FR-063).
//!
//! This is a view over the existing authored Markdown input, not another
//! document model. It extracts only the visible block shapes the three bounded
//! checks need and keeps their source locations.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::diagnostic::Diagnostic;
use crate::lint::LintSeverity;
use crate::parser::FrontmatterStatus;

pub const SENTENCE_LENGTH: &str = "sentence-length";
pub const HEADING_SKIP: &str = "heading-skip";
pub const UNDEFINED_ACRONYM: &str = "undefined-acronym";

/// One named profile body. The profile id is the key under
/// `plain_language_profiles:` in a module manifest (or the id supplied by a
/// project caller); keeping it outside the body makes first-wins merge explicit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlainLanguageProfile {
    pub version: String,
    /// Frontmatter `type` values in scope. Empty means every loaded document.
    #[serde(default)]
    pub document_types: Vec<String>,
    pub sentence_word_limit: usize,
    pub max_heading_level_step: u8,
    #[serde(default)]
    pub known_acronyms: BTreeMap<String, String>,
    /// Intentional uppercase words which are neither acronyms nor errors (for
    /// example, a corpus's normative modal vocabulary).
    #[serde(default)]
    pub ignored_uppercase_terms: BTreeSet<String>,
}

impl PlainLanguageProfile {
    /// Validate the typed profile after deserialization. No defaults are
    /// supplied: every effective threshold is attributable to the profile.
    pub fn validate(&self, id: &str) -> Result<(), String> {
        if id.trim().is_empty() {
            return Err("plain-language profile id must not be empty".to_string());
        }
        if self.version.trim().is_empty() {
            return Err(format!(
                "plain-language profile '{id}' has an empty `version`"
            ));
        }
        if self.sentence_word_limit == 0 {
            return Err(format!(
                "plain-language profile '{id}' has zero `sentence_word_limit`"
            ));
        }
        if self
            .document_types
            .iter()
            .any(|kind| kind.trim().is_empty())
        {
            return Err(format!(
                "plain-language profile '{id}' has an empty `document_types` entry"
            ));
        }
        if !(1..=5).contains(&self.max_heading_level_step) {
            return Err(format!(
                "plain-language profile '{id}' has `max_heading_level_step` outside 1..=5"
            ));
        }
        for (acronym, definition) in &self.known_acronyms {
            if !is_acronym(acronym) {
                return Err(format!(
                    "plain-language profile '{id}' acronym '{acronym}' must be 2..=12 uppercase letters/digits"
                ));
            }
            if definition.trim().is_empty() {
                return Err(format!(
                    "plain-language profile '{id}' acronym '{acronym}' has an empty definition"
                ));
            }
        }
        for term in &self.ignored_uppercase_terms {
            if !is_acronym(term) {
                return Err(format!(
                    "plain-language profile '{id}' ignored uppercase term '{term}' must be 2..=12 uppercase letters/digits"
                ));
            }
        }
        Ok(())
    }

    /// Stable identity of the complete effective configuration.
    pub fn fingerprint(&self, id: &str) -> String {
        let body = serde_json::to_vec(&(id, self))
            .expect("PlainLanguageProfile contains only serializable fields");
        let digest = Sha256::digest(body);
        format!("sha256:{digest:x}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReaderBlockKind {
    Heading,
    Paragraph,
    ListItem,
    Quote,
    Alert,
    TableCell,
}

/// A normalized visible block. `line` is 1-based in the original document,
/// including valid frontmatter lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReaderBlock {
    pub kind: ReaderBlockKind,
    pub text: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_level: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlainLanguageFinding {
    pub rule: String,
    pub severity: LintSeverity,
    pub path: PathBuf,
    pub line: usize,
    pub message: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedPlainLanguageInput {
    pub path: PathBuf,
    pub reason: String,
    pub message: String,
}

/// Accountable batch output. In particular, `readable_blocks == 0` cannot be
/// mistaken for a clean run over content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlainLanguageReport {
    pub profile: String,
    pub profile_version: String,
    pub configuration_fingerprint: String,
    pub documents_examined: usize,
    pub readable_documents: usize,
    pub readable_blocks: usize,
    pub findings: Vec<PlainLanguageFinding>,
    pub skipped_inputs: Vec<SkippedPlainLanguageInput>,
}

/// Extract the source-located reader-visible block view from authored Markdown.
pub fn reader_blocks(markdown: &str) -> Vec<ReaderBlock> {
    let fm = crate::parser::frontmatter::extract_frontmatter_ref(markdown);
    let body = fm.body;
    let body_start = markdown.len().saturating_sub(body.len());
    let base_line = markdown[..body_start]
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
        + 1;

    // Malformed frontmatter follows FR-006's content fallback for this pure
    // function. The repository batch path records it as a skipped non-document
    // before this function is called.
    let _status: FrontmatterStatus = fm.status;
    let mut blocks = Vec::new();
    let mut pending: Option<PendingBlock> = None;
    let mut fence: Option<(u8, usize)> = None;
    let mut in_comment = false;
    let mut active_alert = false;
    let mut in_table = false;
    let mut lines = body.split('\n').enumerate().peekable();

    while let Some((index, raw_line)) = lines.next() {
        let line_no = base_line + index;
        let (quote_depth, quoted) = strip_quote_prefix(raw_line);
        let fence_view = quoted.trim_start();
        if let Some((marker, width)) = fence_marker(fence_view) {
            flush_pending(&mut pending, &mut blocks);
            in_table = false;
            match fence {
                None => fence = Some((marker, width)),
                Some((open, open_width)) if open == marker && width >= open_width => fence = None,
                Some(_) => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }

        let visible = strip_html_comments(quoted, &mut in_comment);
        let trimmed = visible.trim();
        if quote_depth == 0 {
            active_alert = false;
        }
        if trimmed.is_empty() {
            flush_pending(&mut pending, &mut blocks);
            in_table = false;
            continue;
        }
        if is_indented_code(quoted) {
            flush_pending(&mut pending, &mut blocks);
            continue;
        }
        if let Some((level, text)) = heading(trimmed) {
            flush_pending(&mut pending, &mut blocks);
            let text = normalize_inline(text);
            if !text.is_empty() {
                blocks.push(ReaderBlock {
                    kind: ReaderBlockKind::Heading,
                    text,
                    line: line_no,
                    heading_level: Some(level),
                });
            }
            continue;
        }
        if is_thematic_break(trimmed) {
            flush_pending(&mut pending, &mut blocks);
            continue;
        }
        if !trimmed.contains('|') {
            in_table = false;
        }
        if trimmed.contains('|') {
            let starts_table = !in_table
                && lines.peek().is_some_and(|(_, next)| {
                    let (_, next) = strip_quote_prefix(next);
                    let next = next.trim();
                    next.contains('|') && is_table_delimiter(&table_cells(next))
                });
            if in_table || starts_table {
                let cells = table_cells(trimmed);
                if cells.is_empty() {
                    in_table = false;
                } else {
                    flush_pending(&mut pending, &mut blocks);
                    in_table = true;
                    if !is_table_delimiter(&cells) {
                        for cell in cells {
                            let text = normalize_inline(&cell);
                            if !text.is_empty() {
                                blocks.push(ReaderBlock {
                                    kind: ReaderBlockKind::TableCell,
                                    text,
                                    line: line_no,
                                    heading_level: None,
                                });
                            }
                        }
                    }
                    continue;
                }
            }
        }

        let (list_item, content) = strip_list_marker(trimmed);
        if quote_depth > 0 && is_alert_marker(content) {
            flush_pending(&mut pending, &mut blocks);
            active_alert = true;
            continue;
        }
        let list_continuation = !list_item
            && pending
                .as_ref()
                .is_some_and(|block| block.kind == ReaderBlockKind::ListItem)
            && visible.starts_with(char::is_whitespace);
        let kind = if quote_depth > 0 && active_alert {
            ReaderBlockKind::Alert
        } else if list_item || list_continuation {
            ReaderBlockKind::ListItem
        } else if quote_depth > 0 {
            ReaderBlockKind::Quote
        } else {
            ReaderBlockKind::Paragraph
        };
        let text = normalize_inline(content);
        if text.is_empty() {
            flush_pending(&mut pending, &mut blocks);
            continue;
        }

        // Every explicit list marker starts a new block. Wrapped lines keep the
        // preceding kind/location; a change between quote and normal prose also
        // starts a block.
        let starts_new =
            list_item || pending.as_ref().is_some_and(|p| p.kind != kind) || pending.is_none();
        if starts_new {
            flush_pending(&mut pending, &mut blocks);
            pending = Some(PendingBlock {
                kind,
                text,
                line: line_no,
            });
        } else if let Some(block) = pending.as_mut() {
            block.text.push(' ');
            block.text.push_str(&text);
        }
    }
    flush_pending(&mut pending, &mut blocks);
    blocks
}

/// Evaluate one document against an explicit profile.
pub fn check_plain_language(
    path: &Path,
    markdown: &str,
    profile: &PlainLanguageProfile,
) -> Vec<PlainLanguageFinding> {
    if !profile_applies(profile, markdown) {
        return Vec::new();
    }
    let blocks = reader_blocks(markdown);
    check_blocks(path, &blocks, profile)
}

/// Walk a bounded document root and produce an accountable batch report.
pub fn check_plain_language_at(
    root: &Path,
    profile_id: &str,
    profile: &PlainLanguageProfile,
) -> PlainLanguageReport {
    check_plain_language_repo_load(crate::corpus::walk::load_repo(root), profile_id, profile)
}

/// Batch counterpart for a caller that already paid for the FR-024 walk.
pub fn check_plain_language_repo_load(
    load: crate::corpus::walk::RepoLoad,
    profile_id: &str,
    profile: &PlainLanguageProfile,
) -> PlainLanguageReport {
    let mut skipped_inputs = Vec::new();
    for diagnostic in &load.diagnostics {
        match diagnostic {
            Diagnostic::DocumentWithoutFrontmatter { path, malformed } => {
                let reason = if *malformed {
                    "malformed-frontmatter"
                } else {
                    "no-frontmatter"
                };
                skipped_inputs.push(SkippedPlainLanguageInput {
                    path: path.clone(),
                    reason: reason.to_string(),
                    message: "file is not a corpus document and was not analyzed".to_string(),
                });
            }
            Diagnostic::DocumentUnreadable { path, reason } => {
                skipped_inputs.push(SkippedPlainLanguageInput {
                    path: path.clone(),
                    reason: "unreadable".to_string(),
                    message: reason.clone(),
                });
            }
            Diagnostic::SearchPathMissing { path }
            | Diagnostic::SearchPathNotADirectory { path } => {
                skipped_inputs.push(SkippedPlainLanguageInput {
                    path: path.clone(),
                    reason: "unavailable-root".to_string(),
                    message: "document root is unavailable".to_string(),
                });
            }
            _ => {}
        }
    }

    let documents_examined = load.documents.len();
    let mut readable_documents = 0;
    let mut readable_blocks = 0;
    let mut findings = Vec::new();
    for document in load.documents {
        if !profile.document_types.is_empty()
            && !document
                .concept_type()
                .is_some_and(|kind| profile.document_types.iter().any(|allowed| allowed == kind))
        {
            skipped_inputs.push(SkippedPlainLanguageInput {
                path: document.path.clone(),
                reason: "profile-not-applicable".to_string(),
                message: "document type is outside the selected profile".to_string(),
            });
            continue;
        }
        let blocks = reader_blocks(document.raw());
        if blocks.is_empty() {
            skipped_inputs.push(SkippedPlainLanguageInput {
                path: document.path.clone(),
                reason: "no-readable-prose".to_string(),
                message: "document contains no reader-visible prose blocks".to_string(),
            });
            continue;
        }
        readable_documents += 1;
        readable_blocks += blocks.len();
        findings.extend(check_blocks(&document.path, &blocks, profile));
    }

    findings.sort_by(|a, b| {
        (&a.path, a.line, &a.rule, &a.message).cmp(&(&b.path, b.line, &b.rule, &b.message))
    });
    skipped_inputs.sort_by(|a, b| (&a.path, &a.reason).cmp(&(&b.path, &b.reason)));
    skipped_inputs.dedup_by(|a, b| a.path == b.path && a.reason == b.reason);

    PlainLanguageReport {
        profile: profile_id.to_string(),
        profile_version: profile.version.clone(),
        configuration_fingerprint: profile.fingerprint(profile_id),
        documents_examined,
        readable_documents,
        readable_blocks,
        findings,
        skipped_inputs,
    }
}

fn profile_applies(profile: &PlainLanguageProfile, markdown: &str) -> bool {
    if profile.document_types.is_empty() {
        return true;
    }
    crate::parser::parse_header(markdown)
        .and_then(|header| header.type_)
        .is_some_and(|kind| {
            profile
                .document_types
                .iter()
                .any(|allowed| allowed == &kind)
        })
}

fn check_blocks(
    path: &Path,
    blocks: &[ReaderBlock],
    profile: &PlainLanguageProfile,
) -> Vec<PlainLanguageFinding> {
    let mut findings = Vec::new();
    let mut previous_heading: Option<u8> = None;
    let mut defined: BTreeSet<String> = profile.known_acronyms.keys().cloned().collect();
    defined.extend(profile.ignored_uppercase_terms.iter().cloned());
    let mut reported: BTreeSet<String> = BTreeSet::new();

    for block in blocks {
        if let Some(level) = block.heading_level {
            if let Some(previous) = previous_heading {
                if level > previous.saturating_add(profile.max_heading_level_step) {
                    findings.push(finding(
                        path,
                        block,
                        HEADING_SKIP,
                        format!(
                            "heading level {level} skips beyond configured step {} from level {previous}",
                            profile.max_heading_level_step
                        ),
                    ));
                }
            }
            previous_heading = Some(level);
        }

        for sentence in sentence_slices(&block.text) {
            let words = word_count(sentence);
            if words > profile.sentence_word_limit {
                findings.push(finding(
                    path,
                    block,
                    SENTENCE_LENGTH,
                    format!(
                        "sentence has {words} words; configured limit is {}",
                        profile.sentence_word_limit
                    ),
                ));
            }
        }

        let inline = inline_definitions(&block.text);
        for (offset, acronym) in acronym_tokens(&block.text) {
            // A definition suppresses its own parenthesized occurrence and
            // later uses, never an earlier use in the same prose block.
            if inline
                .get(acronym)
                .is_some_and(|definition_offset| *definition_offset <= offset)
            {
                defined.insert(acronym.to_string());
                continue;
            }
            if defined.contains(acronym) || !reported.insert(acronym.to_string()) {
                continue;
            }
            findings.push(finding(
                path,
                block,
                UNDEFINED_ACRONYM,
                format!("acronym '{acronym}' is not defined in this document or profile"),
            ));
        }
        defined.extend(inline.into_keys());
    }
    findings
}

fn finding(path: &Path, block: &ReaderBlock, rule: &str, message: String) -> PlainLanguageFinding {
    PlainLanguageFinding {
        rule: rule.to_string(),
        severity: LintSeverity::Warning,
        path: path.to_path_buf(),
        line: block.line,
        message,
        excerpt: block.text.chars().take(160).collect(),
    }
}

#[derive(Debug)]
struct PendingBlock {
    kind: ReaderBlockKind,
    text: String,
    line: usize,
}

fn flush_pending(pending: &mut Option<PendingBlock>, out: &mut Vec<ReaderBlock>) {
    if let Some(block) = pending.take() {
        out.push(ReaderBlock {
            kind: block.kind,
            text: block.text,
            line: block.line,
            heading_level: None,
        });
    }
}

fn strip_quote_prefix(mut line: &str) -> (usize, &str) {
    let mut depth = 0;
    loop {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix('>') else {
            return (depth, line);
        };
        depth += 1;
        line = rest.strip_prefix(' ').unwrap_or(rest);
    }
}

fn fence_marker(line: &str) -> Option<(u8, usize)> {
    let marker = *line.as_bytes().first()?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let width = line.bytes().take_while(|b| *b == marker).count();
    (width >= 3).then_some((marker, width))
}

fn is_indented_code(line: &str) -> bool {
    line.starts_with('\t') || line.starts_with("    ")
}

fn heading(line: &str) -> Option<(u8, &str)> {
    let hashes = line.bytes().take_while(|b| *b == b'#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = &line[hashes..];
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let text = rest.trim().trim_end_matches('#').trim_end();
    (!text.is_empty()).then_some((hashes as u8, text))
}

fn is_thematic_break(line: &str) -> bool {
    let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    compact.len() >= 3
        && compact
            .chars()
            .next()
            .is_some_and(|first| "-* _".contains(first) && compact.chars().all(|c| c == first))
}

fn table_cells(line: &str) -> Vec<String> {
    crate::query::split_row_unescaped(line)
}

fn is_table_delimiter(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let cell = cell.trim_matches(':').trim();
            cell.len() >= 3 && cell.chars().all(|c| c == '-')
        })
}

fn strip_list_marker(line: &str) -> (bool, &str) {
    if let Some(rest) = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| line.strip_prefix("+ "))
    {
        return (true, rest);
    }
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0 {
        let rest = &line[digits..];
        if let Some(rest) = rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")) {
            return (true, rest);
        }
    }
    (false, line)
}

fn is_alert_marker(line: &str) -> bool {
    let line = line.trim();
    line.starts_with("[!") && line.ends_with(']') && line.len() > 3
}

fn strip_html_comments(line: &str, in_comment: &mut bool) -> String {
    let mut rest = line;
    let mut out = String::new();
    loop {
        if *in_comment {
            let Some(end) = rest.find("-->") else {
                return out;
            };
            rest = &rest[end + 3..];
            *in_comment = false;
            continue;
        }
        let Some(start) = rest.find("<!--") else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..start]);
        rest = &rest[start + 4..];
        *in_comment = true;
    }
}

fn normalize_inline(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let width = bytes[i..].iter().take_while(|b| **b == b'`').count();
            let mut j = i + width;
            let mut close = None;
            while j < bytes.len() {
                if bytes[j] == b'`' {
                    let found = bytes[j..].iter().take_while(|b| **b == b'`').count();
                    if found == width {
                        close = Some(j + width);
                        break;
                    }
                    j += found;
                } else {
                    j += 1;
                }
            }
            i = close.unwrap_or(bytes.len());
            continue;
        }
        if bytes[i] == b']' && bytes.get(i + 1) == Some(&b'(') {
            if let Some(end) = input[i + 2..].find(')') {
                i += 2 + end + 1;
                continue;
            }
        }
        let ch = input[i..].chars().next().expect("i is a char boundary");
        let width = ch.len_utf8();
        match ch {
            '[' | ']' | '*' | '_' | '~' => {}
            '<' => {
                if let Some(end) = input[i..].find('>') {
                    i += end + 1;
                    continue;
                }
                out.push(ch);
            }
            '\\' => {
                if let Some(next) = input[i + width..].chars().next() {
                    out.push(next);
                    i += next.len_utf8();
                }
            }
            _ => out.push(ch),
        }
        i += width;
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn sentence_slices(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    for (index, ch) in text.char_indices() {
        if matches!(ch, '.' | '?' | '!') {
            let end = index + ch.len_utf8();
            if !text[start..end].trim().is_empty() {
                out.push(text[start..end].trim());
            }
            start = end;
        }
    }
    if !text[start..].trim().is_empty() {
        out.push(text[start..].trim());
    }
    out
}

fn word_count(text: &str) -> usize {
    let mut count = 0;
    let mut in_word = false;
    for ch in text.chars() {
        let word = ch.is_alphanumeric();
        if word && !in_word {
            count += 1;
        }
        in_word = word;
    }
    count
}

fn acronym_tokens(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start = None;
    for (index, ch) in text.char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            start.get_or_insert(index);
        } else if let Some(token_start) = start.take() {
            let token = &text[token_start..index];
            if is_acronym(token) {
                out.push((token_start, token));
            }
        }
    }
    if let Some(token_start) = start {
        let token = &text[token_start..];
        if is_acronym(token) {
            out.push((token_start, token));
        }
    }
    out
}

fn is_acronym(token: &str) -> bool {
    let len = token.len();
    (2..=12).contains(&len)
        && !is_artifact_identifier(token)
        && token.chars().filter(|c| c.is_ascii_alphabetic()).count() >= 2
        && token
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
}

fn is_artifact_identifier(token: &str) -> bool {
    let mut parts = token.split('-');
    let Some(prefix) = parts.next() else {
        return false;
    };
    let Some(number) = parts.next() else {
        return false;
    };
    prefix.len() >= 2
        && prefix.chars().all(|c| c.is_ascii_uppercase())
        && number.chars().all(|c| c.is_ascii_digit())
}

fn inline_definitions(text: &str) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for (open, _) in text.match_indices('(') {
        let rest = &text[open + 1..];
        let Some(close) = rest.find(')') else {
            continue;
        };
        let acronym = rest[..close].trim();
        if !is_acronym(acronym) {
            continue;
        }
        let count = acronym.chars().filter(|c| c.is_ascii_alphabetic()).count();
        let words: Vec<&str> = text[..open]
            .split(|c: char| !c.is_alphanumeric())
            .filter(|word| !word.is_empty())
            .collect();
        if words.len() < count {
            continue;
        }
        let initials: String = words[words.len() - count..]
            .iter()
            .filter_map(|word| word.chars().next())
            .flat_map(char::to_uppercase)
            .collect();
        let letters: String = acronym
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect();
        if initials == letters {
            let leading_space = rest[..close].len() - rest[..close].trim_start().len();
            out.entry(acronym.to_string())
                .or_insert(open + 1 + leading_space);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    fn profile(limit: usize) -> PlainLanguageProfile {
        PlainLanguageProfile {
            version: "1.0.0".to_string(),
            document_types: Vec::new(),
            sentence_word_limit: limit,
            max_heading_level_step: 1,
            known_acronyms: BTreeMap::from([(
                "API".to_string(),
                "application programming interface".to_string(),
            )]),
            ignored_uppercase_terms: BTreeSet::new(),
        }
    }

    #[trace("TC-970", "FR-063-AC-1")]
    #[test]
    fn reader_view_excludes_metadata_and_code_with_document_lines() {
        let input = "---\nid: FR-1\ntype: FR\n---\n# Visible\nText with `HIDDEN API` here.\n```rust\nCODE API\n```\n    indented API\n>     quoted code API\n<!-- SECRET API -->\nLast words.";
        let blocks = reader_blocks(input);
        assert_eq!(
            blocks.iter().map(|b| b.line).collect::<Vec<_>>(),
            vec![5, 6, 13]
        );
        let all = blocks
            .iter()
            .map(|b| b.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(all.contains("Visible"));
        assert!(all.contains("Last words"));
        assert!(!all.contains("HIDDEN"));
        assert!(!all.contains("CODE"));
        assert!(!all.contains("SECRET"));
        assert!(!all.contains("quoted code"));
    }

    #[trace("TC-971", "FR-063-AC-2")]
    #[test]
    fn compound_reader_blocks_keep_shapes_and_wrapping() {
        let input = "Paragraph wraps\nonto another line.\n\n- first\n  wraps here\n  - nested\n> quote wraps\n> here\n> [!NOTE]\n> alert body";
        let blocks = reader_blocks(input);
        assert_eq!(blocks[0].text, "Paragraph wraps onto another line.");
        assert_eq!(blocks[1].kind, ReaderBlockKind::ListItem);
        assert_eq!(blocks[1].text, "first wraps here");
        assert_eq!(blocks[2].text, "nested");
        assert_eq!(blocks[3].kind, ReaderBlockKind::Quote);
        assert_eq!(blocks[3].text, "quote wraps here");
        assert_eq!(blocks[4].kind, ReaderBlockKind::Alert);
        assert_eq!(blocks[4].text, "alert body");
    }

    #[trace("TC-972", "FR-063-AC-3")]
    #[test]
    fn tables_and_malformed_input_are_bounded() {
        let input = "Choice A | choice B remains prose.\n\n| Name | Meaning |\n|---|:---:|\n| API | visible words |\n| `A\\|B` | still visible |\n\n| Solo |\n|---|\n| value |\n```\nunclosed";
        let first = reader_blocks(input);
        assert_eq!(first, reader_blocks(input));
        assert_eq!(first[0].kind, ReaderBlockKind::Paragraph);
        assert_eq!(first[0].text, "Choice A | choice B remains prose.");
        assert_eq!(
            first
                .iter()
                .filter(|b| b.kind == ReaderBlockKind::TableCell)
                .count(),
            7
        );
        assert!(first.iter().any(|block| block.text == "Solo"));
        assert!(!first.iter().any(|b| b.text.contains("---")));
    }

    #[trace("TC-973", "FR-063-AC-4")]
    #[test]
    fn sentence_limit_is_strict_and_ignores_non_reader_text() {
        let p = profile(5);
        assert!(check_plain_language(Path::new("a.md"), "One two three four five.", &p).is_empty());
        let findings = check_plain_language(
            Path::new("a.md"),
            "One two three four five six. [Link](https://very.long.destination/with/words) `seven eight`.",
            &p,
        );
        assert_eq!(
            findings
                .iter()
                .filter(|f| f.rule == SENTENCE_LENGTH)
                .count(),
            1
        );
    }

    #[trace("TC-974", "FR-063-AC-5")]
    #[test]
    fn heading_skip_respects_boundary() {
        let findings = check_plain_language(
            Path::new("a.md"),
            "## First\n### Fine\n##### Skip\n#### Descend",
            &profile(50),
        );
        let skips: Vec<_> = findings.iter().filter(|f| f.rule == HEADING_SKIP).collect();
        assert_eq!(skips.len(), 1);
        assert_eq!(skips[0].line, 3);
    }

    #[trace("TC-975", "FR-063-AC-6")]
    #[test]
    fn acronym_first_use_definition_vocabulary_and_code() {
        let mut p = profile(50);
        p.ignored_uppercase_terms.insert("SHALL".to_string());
        let findings = check_plain_language(
            Path::new("a.md"),
            "FR-063 SHALL use API. Service level objective (SLO) is stated. XYZ appears. XYZ repeats. `HID`.",
            &p,
        );
        let acronyms: Vec<_> = findings
            .iter()
            .filter(|f| f.rule == UNDEFINED_ACRONYM)
            .map(|f| f.message.as_str())
            .collect();
        assert_eq!(acronyms.len(), 1);
        assert!(acronyms[0].contains("XYZ"));

        let late_definition = check_plain_language(
            Path::new("a.md"),
            "SLO appears first. Service level objective (SLO) is defined later.",
            &p,
        );
        assert_eq!(
            late_definition
                .iter()
                .filter(|finding| {
                    finding.rule == UNDEFINED_ACRONYM && finding.message.contains("SLO")
                })
                .count(),
            1
        );
    }

    #[trace("TC-979", "FR-063-AC-10")]
    #[test]
    fn fingerprint_covers_effective_configuration() {
        let p = profile(20);
        assert_eq!(p.fingerprint("docs"), p.fingerprint("docs"));
        assert_ne!(p.fingerprint("docs"), p.fingerprint("other"));
        assert_ne!(p.fingerprint("docs"), profile(21).fingerprint("docs"));
        let mut version = p.clone();
        version.version = "2.0.0".to_string();
        assert_ne!(p.fingerprint("docs"), version.fingerprint("docs"));
        let mut applicability = p.clone();
        applicability.document_types.push("FR".to_string());
        assert_ne!(p.fingerprint("docs"), applicability.fingerprint("docs"));
        let mut heading = p.clone();
        heading.max_heading_level_step = 2;
        assert_ne!(p.fingerprint("docs"), heading.fingerprint("docs"));
        let mut vocabulary = p.clone();
        vocabulary
            .known_acronyms
            .insert("SLO".to_string(), "service level objective".to_string());
        assert_ne!(p.fingerprint("docs"), vocabulary.fingerprint("docs"));
        let mut ignored = p.clone();
        ignored.ignored_uppercase_terms.insert("SHALL".to_string());
        assert_ne!(p.fingerprint("docs"), ignored.fingerprint("docs"));
    }

    #[trace("TC-980", "FR-063-AC-11")]
    #[test]
    fn findings_have_common_advisory_location_shape() {
        let findings = check_plain_language(
            Path::new("doc.md"),
            "# A\n### XYZ has many extra words here.",
            &profile(3),
        );
        assert!(!findings.is_empty());
        let valid = BTreeSet::from([SENTENCE_LENGTH, HEADING_SKIP, UNDEFINED_ACRONYM]);
        assert!(findings.iter().all(|f| {
            f.severity == LintSeverity::Warning
                && f.path == Path::new("doc.md")
                && f.line > 0
                && !f.excerpt.is_empty()
                && valid.contains(f.rule.as_str())
        }));
    }
}
