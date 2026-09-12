//! Markdown delimiter auto-pair policy for the shared text-input boundary.
//!
//! GPUI-free: pairing is an editor policy over UTF-8 offsets, not a parse
//! concern. The application applies [`auto_pair_action`] before the canonical
//! source mutation.

use std::ops::Range;

use crate::model::VisualEditorFieldKind;

/// Openers that insert a matching closer. Symmetric markers appear as both
/// opener and closer (`*`, `_`, `` ` ``, `$`, `"`, `'`).
const PAIRS: &[(char, char)] = &[
    ('*', '*'),
    ('_', '_'),
    ('`', '`'),
    ('$', '$'),
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
    ('"', '"'),
    ('\'', '\''),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoPairAction {
    /// Insert or wrap, then place this selection.
    Replace {
        range: Range<usize>,
        replacement: String,
        selection_after: Range<usize>,
    },
    /// Next source character is already the closer: move the caret only.
    Skip { caret: usize },
}

/// Fenced-code, block-math, diagram, and HTML payload editors do not pair.
pub fn is_auto_pair_restricted_field(kind: VisualEditorFieldKind) -> bool {
    matches!(
        kind,
        VisualEditorFieldKind::CodePayload
            | VisualEditorFieldKind::MathPayload
            | VisualEditorFieldKind::HtmlSource
            | VisualEditorFieldKind::CodeInfo
    )
}

/// Decide how a pending insert or Backspace should be rewritten.
///
/// `new_text` is empty on Backspace (the caller has already selected the
/// previous character). Returns `None` when the keystroke is not a pairing
/// event — including a preceding `\`.
pub fn auto_pair_action(text: &str, range: Range<usize>, new_text: &str) -> Option<AutoPairAction> {
    if range.start > text.len()
        || range.end > text.len()
        || range.start > range.end
        || !text.is_char_boundary(range.start)
        || !text.is_char_boundary(range.end)
    {
        return None;
    }
    if preceded_by_backslash(text, range.start) {
        return None;
    }
    if new_text.is_empty() {
        return auto_pair_backspace(text, range);
    }
    let mut chars = new_text.chars();
    let typed = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    if range.is_empty() {
        // Skip when the typed character is a closer (including symmetric
        // `*`/`_`/`"` ) and the next source character is already that closer.
        // Typing an opener such as `(` in front of `)` still inserts a pair.
        if is_closer(typed) && next_char(text, range.end) == Some(typed) {
            return Some(AutoPairAction::Skip {
                caret: range.end + typed.len_utf8(),
            });
        }
        let closer = closer_for(typed)?;
        let replacement = format!("{typed}{closer}");
        let caret = range.start + typed.len_utf8();
        return Some(AutoPairAction::Replace {
            range,
            replacement,
            selection_after: caret..caret,
        });
    }
    let closer = closer_for(typed)?;
    let selected = &text[range.clone()];
    let replacement = format!("{typed}{selected}{closer}");
    let inner_start = range.start + typed.len_utf8();
    let inner_end = inner_start + selected.len();
    Some(AutoPairAction::Replace {
        range,
        replacement,
        selection_after: inner_start..inner_end,
    })
}

fn auto_pair_backspace(text: &str, range: Range<usize>) -> Option<AutoPairAction> {
    if range.is_empty() {
        return None;
    }
    let deleted = text.get(range.clone())?;
    let mut chars = deleted.chars();
    let opener = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    let closer = closer_for(opener)?;
    if next_char(text, range.end) != Some(closer) {
        return None;
    }
    let end = range.end + closer.len_utf8();
    Some(AutoPairAction::Replace {
        range: range.start..end,
        replacement: String::new(),
        selection_after: range.start..range.start,
    })
}

fn closer_for(opener: char) -> Option<char> {
    PAIRS
        .iter()
        .find_map(|(left, right)| (*left == opener).then_some(*right))
}

fn is_closer(ch: char) -> bool {
    PAIRS.iter().any(|(_, right)| *right == ch)
}

fn next_char(text: &str, offset: usize) -> Option<char> {
    text.get(offset..)?.chars().next()
}

fn preceded_by_backslash(text: &str, offset: usize) -> bool {
    if offset == 0 || !text.is_char_boundary(offset) {
        return false;
    }
    text[..offset].chars().next_back() == Some('\\')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn insert(text: &str, caret: usize, typed: &str) -> Option<AutoPairAction> {
        auto_pair_action(text, caret..caret, typed)
    }

    #[test]
    fn empty_caret_inserts_a_wrap_pair() {
        assert_eq!(
            insert("hello", 5, "*"),
            Some(AutoPairAction::Replace {
                range: 5..5,
                replacement: "**".into(),
                selection_after: 6..6,
            })
        );
        assert_eq!(
            insert("", 0, "("),
            Some(AutoPairAction::Replace {
                range: 0..0,
                replacement: "()".into(),
                selection_after: 1..1,
            })
        );
        assert_eq!(
            insert("", 0, "`"),
            Some(AutoPairAction::Replace {
                range: 0..0,
                replacement: "``".into(),
                selection_after: 1..1,
            })
        );
        assert_eq!(
            insert("", 0, "$"),
            Some(AutoPairAction::Replace {
                range: 0..0,
                replacement: "$$".into(),
                selection_after: 1..1,
            })
        );
    }

    #[test]
    fn selection_is_wrapped_and_stays_on_the_inner_text() {
        assert_eq!(
            auto_pair_action("hello", 0..5, "("),
            Some(AutoPairAction::Replace {
                range: 0..5,
                replacement: "(hello)".into(),
                selection_after: 1..6,
            })
        );
        assert_eq!(
            auto_pair_action("hi", 0..2, "*"),
            Some(AutoPairAction::Replace {
                range: 0..2,
                replacement: "*hi*".into(),
                selection_after: 1..3,
            })
        );
    }

    #[test]
    fn typing_an_existing_closer_skips() {
        assert_eq!(
            insert("**", 1, "*"),
            Some(AutoPairAction::Skip { caret: 2 })
        );
        assert_eq!(
            insert("()", 1, ")"),
            Some(AutoPairAction::Skip { caret: 2 })
        );
        assert_eq!(
            insert("[]", 1, "]"),
            Some(AutoPairAction::Skip { caret: 2 })
        );
        assert_eq!(
            insert("{}", 1, "}"),
            Some(AutoPairAction::Skip { caret: 2 })
        );
        assert_eq!(
            insert("``", 1, "`"),
            Some(AutoPairAction::Skip { caret: 2 })
        );
    }

    #[test]
    fn backspace_unwraps_an_empty_pair() {
        assert_eq!(
            auto_pair_action("**", 0..1, ""),
            Some(AutoPairAction::Replace {
                range: 0..2,
                replacement: String::new(),
                selection_after: 0..0,
            })
        );
        assert_eq!(
            auto_pair_action("()", 0..1, ""),
            Some(AutoPairAction::Replace {
                range: 0..2,
                replacement: String::new(),
                selection_after: 0..0,
            })
        );
    }

    #[test]
    fn backslash_and_multi_char_inserts_are_suppressed() {
        assert_eq!(insert("\\", 1, "*"), None);
        assert_eq!(insert("hello", 5, "**"), None);
        assert_eq!(insert("hello", 5, "a"), None);
        assert_eq!(insert("hello", 5, "="), None);
        assert_eq!(auto_pair_action("*x*", 0..1, ""), None);
    }

    #[test]
    fn restricted_fields_cover_code_math_html_and_info() {
        assert!(is_auto_pair_restricted_field(
            VisualEditorFieldKind::CodePayload
        ));
        assert!(is_auto_pair_restricted_field(
            VisualEditorFieldKind::MathPayload
        ));
        assert!(is_auto_pair_restricted_field(
            VisualEditorFieldKind::HtmlSource
        ));
        assert!(is_auto_pair_restricted_field(
            VisualEditorFieldKind::CodeInfo
        ));
        assert!(!is_auto_pair_restricted_field(
            VisualEditorFieldKind::TableCell { row: 0, column: 0 }
        ));
        assert!(!is_auto_pair_restricted_field(
            VisualEditorFieldKind::ImageAlt
        ));
    }
}
