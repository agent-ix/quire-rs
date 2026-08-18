//! Markdown lexical primitives shared by every reader that has to tell a
//! **mention** of syntax from a **use** of it.
//!
//! A backticked span is example data. `shall` quoted inside a code span does
//! not impose an obligation (CR-017); an `ix://` URI quoted inside one is not a
//! reference (CR-067). Both readers want the same rule, and before CR-067 the
//! grammar checks were the only ones that had it — [`mask_code_spans`] lived in
//! `grammar::ac`, so `corpus::resolve` could not reach it without either
//! widening that module's surface or writing the rule a second time. It lives
//! here instead, owned by neither consumer.

/// Neutralize the contents of every closed code span in `text`, leaving the
/// delimiters and all surrounding prose intact.
///
/// The mask is **byte-length-preserving** — each character inside a span
/// becomes as many `x` bytes as it occupied — so an offset found in the masked
/// copy indexes the original. Both consumers depend on that: `grammar::ac`'s
/// `outcome_clause` locates a keyword in the mask and slices the original, and
/// `corpus::resolve` matches links in the mask and harvests the original bytes.
///
/// Replacing the contents rather than deleting the span is what lets the
/// grammar's *vocabulary* checks keep reading the real words: a backticked
/// lexicon term still suppresses `vague-response` (FR-043-AC-3) and a
/// backticked identifier still counts as a concrete-object signal (FR-042),
/// because those checks read the unmasked original.
///
/// An **unbalanced** backtick run opens no span; the tail is returned as
/// ordinary prose. A stray backtick therefore cannot silently swallow every
/// later keyword or link in a document.
pub(crate) fn mask_code_spans(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('`') {
        let (before, from_open) = rest.split_at(open);
        out.push_str(before);
        // CommonMark: a run of N backticks is closed by the next run of exactly
        // N (CR-026). Reading only the first backtick made a ``double-tick``
        // span — the form used to quote a fragment that itself contains a code
        // span — degenerate into an empty span, leaving the quoted keywords
        // *inside* it unmasked and read as though they were used.
        let ticks = from_open.len() - from_open.trim_start_matches('`').len();
        let body = &from_open[ticks..];
        let Some(close) = find_closing_run(body, ticks) else {
            // An unbalanced run opens no span; the tail is ordinary prose.
            out.push_str(from_open);
            return out;
        };
        for _ in 0..ticks {
            out.push('`');
        }
        for c in body[..close].chars() {
            for _ in 0..c.len_utf8() {
                out.push('x');
            }
        }
        for _ in 0..ticks {
            out.push('`');
        }
        rest = &body[close + ticks..];
    }
    out.push_str(rest);
    out
}

/// The byte offset in `body` of the next backtick run of exactly `ticks`, or
/// `None` when the span is never closed. A longer run is *not* a closer — it is
/// content — which is what keeps a `` `nested` `` span inside a double-tick one
/// from ending it early.
fn find_closing_run(body: &str, ticks: usize) -> Option<usize> {
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let run = bytes[i..].iter().take_while(|&&b| b == b'`').count();
        if run == ticks {
            return Some(i);
        }
        i += run;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-880 (FR-026-AC-12/AC-13, CR-067; also the CR-017/CR-026 rule the
    // grammar has relied on since FR-047): the shared masker's contract.
    #[test]
    fn tc880_mask_code_spans_contract() {
        // Contents neutralized, delimiters and surrounding prose intact.
        assert_eq!(mask_code_spans("a `ix://` b"), "a `xxxxx` b");
        // Byte-length preserving, including multi-byte characters.
        let masked = mask_code_spans("`é→`");
        assert_eq!(masked.len(), "`é→`".len());
        assert_eq!(masked, "`xxxxx`");
        // A fenced block is a run of three backticks closed by the next run of
        // exactly three, so the same pass neutralizes its contents.
        let fenced = mask_code_spans("before\n```\nix://o/r/FR-001\n```\nafter");
        assert!(!fenced.contains("ix://"), "fenced block must be masked");
        assert!(fenced.starts_with("before\n") && fenced.ends_with("\nafter"));
        assert_eq!(
            fenced.len(),
            "before\n```\nix://o/r/FR-001\n```\nafter".len()
        );
        // An info string does not stop the fence from closing.
        assert!(!mask_code_spans("```markdown\nix://o/r/FR-001\n```").contains("ix://"));
        // A single backtick inside a fenced block is not a closer.
        let inner = mask_code_spans("```\na `b` c ix://o/r/FR-001\n```");
        assert!(
            !inner.contains("ix://"),
            "inner span must not close the fence"
        );
        // A longer run is content, not a closer (CR-026 double-tick form).
        assert_eq!(mask_code_spans("``a `b` c``"), "``xxxxxxx``");
        // An unbalanced run opens no span: later text stays readable.
        assert_eq!(
            mask_code_spans("stray ` then ix://o/r/FR-001"),
            "stray ` then ix://o/r/FR-001"
        );
        // Text with no backtick at all is returned unchanged.
        assert_eq!(
            mask_code_spans("plain ix://o/r/FR-001"),
            "plain ix://o/r/FR-001"
        );
    }
}
