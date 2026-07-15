//! Extended inline syntax parsing.
//!
//! Handles parsing of:
//! - Superscript: `^text^`
//! - Subscript: `~text~`
//! - Highlight: `==text==`
//! - Emoji: `:shortcode:`

use crate::ast::Inline;
use crate::emoji::shortcode_to_unicode;

/// Parse extended inline syntax from a text string.
///
/// This function scans through text and replaces extended syntax patterns
/// with their corresponding AST nodes.
pub fn parse_extended_inlines(text: &str) -> Vec<Inline> {
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Try to parse superscript: ^text^
        if chars[i] == '^' && i + 1 < chars.len() {
            if let Some((content, end_pos)) = extract_delimited(&chars, i, '^', '^') {
                let inner = parse_extended_inlines(&content);
                result.push(Inline::Superscript(inner));
                i = end_pos;
                continue;
            }
        }

        // Try to parse subscript: ~text~
        // Note: We need to be careful not to confuse with strikethrough ~~
        if chars[i] == '~' && i + 1 < chars.len() && chars.get(i + 1) != Some(&'~') {
            if let Some((content, end_pos)) = extract_delimited(&chars, i, '~', '~') {
                // Make sure it's not strikethrough
                if end_pos < chars.len() && chars.get(end_pos) != Some(&'~') {
                    let inner = parse_extended_inlines(&content);
                    result.push(Inline::Subscript(inner));
                    i = end_pos;
                    continue;
                }
            }
        }

        // Try to parse highlight: ==text==
        if chars[i] == '=' && i + 1 < chars.len() && chars[i + 1] == '=' {
            if let Some((content, end_pos)) = extract_double_delimited(&chars, i, '=', '=') {
                let inner = parse_extended_inlines(&content);
                result.push(Inline::Highlight(inner));
                i = end_pos;
                continue;
            }
        }

        // Try to parse emoji: :shortcode:
        if chars[i] == ':' {
            if let Some((shortcode, end_pos)) = extract_emoji_shortcode(&chars, i) {
                if let Some(unicode) = shortcode_to_unicode(&shortcode) {
                    result.push(Inline::Emoji {
                        shortcode: shortcode.clone(),
                        unicode: unicode.to_string(),
                    });
                    i = end_pos;
                    continue;
                } else {
                    // Not a valid emoji, treat as regular text
                    result.push(Inline::Text(chars[i].to_string()));
                    i += 1;
                }
            } else {
                result.push(Inline::Text(chars[i].to_string()));
                i += 1;
            }
            continue;
        }

        // Regular character
        // Accumulate consecutive regular characters into a single Text node
        let mut text_acc = String::new();
        while i < chars.len() {
            let ch = chars[i];
            // Check if this is the start of a special sequence
            if ch == '^'
                || ch == '~'
                || ch == ':'
                || (ch == '=' && i + 1 < chars.len() && chars[i + 1] == '=')
            {
                // If we reach a special character at the very start of this run
                // (text_acc empty), it means the construct starting here failed
                // to parse above (e.g. an unclosed `^`). Emit it as a literal
                // character and advance so we always make progress and never
                // loop forever.
                if text_acc.is_empty() {
                    text_acc.push(ch);
                    i += 1;
                }
                break;
            }
            text_acc.push(ch);
            i += 1;
        }
        if !text_acc.is_empty() {
            // Merge with a preceding Text node to avoid fragmentation.
            if let Some(Inline::Text(prev)) = result.last_mut() {
                prev.push_str(&text_acc);
            } else {
                result.push(Inline::Text(text_acc));
            }
        }
    }

    // If we collected nothing, return a single empty text node
    if result.is_empty() && !text.is_empty() {
        result.push(Inline::Text(text.to_string()));
    }

    result
}

/// Extract content between single-character delimiters.
///
/// Returns `Some((content, end_position))` if found, where `end_position` points
/// to the character after the closing delimiter.
fn extract_delimited(
    chars: &[char],
    start: usize,
    open_delim: char,
    close_delim: char,
) -> Option<(String, usize)> {
    if chars.get(start) != Some(&open_delim) {
        return None;
    }

    let mut i = start + 1;
    let mut content = String::new();

    while i < chars.len() {
        let ch = chars[i];
        if ch == close_delim {
            // Found closing delimiter
            return Some((content, i + 1));
        }
        if ch == '\\' && i + 1 < chars.len() {
            // Escape sequence
            i += 1;
            content.push(chars[i]);
            i += 1;
        } else {
            content.push(ch);
            i += 1;
        }
    }

    // No closing delimiter found
    None
}

/// Extract content between double-character delimiters (like ==text==).
///
/// Returns `Some((content, end_position))` if found.
fn extract_double_delimited(
    chars: &[char],
    start: usize,
    open_delim: char,
    close_delim: char,
) -> Option<(String, usize)> {
    if start + 1 >= chars.len() {
        return None;
    }
    if chars[start] != open_delim || chars[start + 1] != open_delim {
        return None;
    }

    let mut i = start + 2;
    let mut content = String::new();

    while i + 1 < chars.len() {
        if chars[i] == close_delim && chars[i + 1] == close_delim {
            // Found closing delimiter
            return Some((content, i + 2));
        }
        if chars[i] == '\\' && i + 1 < chars.len() {
            // Escape sequence
            i += 1;
            content.push(chars[i]);
            i += 1;
        } else {
            content.push(chars[i]);
            i += 1;
        }
    }

    // No closing delimiter found
    None
}

/// Extract an emoji shortcode starting from a `:` character.
///
/// Returns `Some((shortcode, end_position))` if a valid shortcode pattern is found.
fn extract_emoji_shortcode(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars.get(start) != Some(&':') {
        return None;
    }

    let mut i = start + 1;
    let mut shortcode = String::new();

    // Shortcodes can contain lowercase letters, numbers, and underscores
    while i < chars.len() {
        let ch = chars[i];
        if ch == ':' {
            // Found closing colon
            if !shortcode.is_empty() {
                return Some((shortcode, i + 1));
            } else {
                return None;
            }
        }
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' {
            shortcode.push(ch);
            i += 1;
        } else {
            // Invalid character for shortcode
            return None;
        }
    }

    // No closing colon found
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superscript() {
        let result = parse_extended_inlines("x^2^ equals 4");
        assert!(matches!(result.get(1), Some(Inline::Superscript(_))));
        if let Some(Inline::Superscript(inner)) = result.get(1) {
            assert_eq!(inner, &vec![Inline::Text("2".to_string())]);
        }
    }

    #[test]
    fn test_subscript() {
        let result = parse_extended_inlines("H~2~O is water");
        assert!(matches!(result.get(1), Some(Inline::Subscript(_))));
        if let Some(Inline::Subscript(inner)) = result.get(1) {
            assert_eq!(inner, &vec![Inline::Text("2".to_string())]);
        }
    }

    #[test]
    fn test_highlight() {
        let result = parse_extended_inlines("This is ==important==!");
        assert!(matches!(result.get(1), Some(Inline::Highlight(_))));
        if let Some(Inline::Highlight(inner)) = result.get(1) {
            assert_eq!(inner, &vec![Inline::Text("important".to_string())]);
        }
    }

    #[test]
    fn test_emoji() {
        let result = parse_extended_inlines("I :heart: Rust!");
        assert!(matches!(result.get(1), Some(Inline::Emoji { .. })));
        if let Some(Inline::Emoji { shortcode, unicode }) = result.get(1) {
            assert_eq!(shortcode, "heart");
            assert_eq!(unicode, "❤️");
        }
    }

    #[test]
    fn test_invalid_emoji() {
        let result = parse_extended_inlines("Not an :invalid_emoji: here");
        // Should be treated as regular text
        assert!(result.iter().all(|i| matches!(i, Inline::Text(_))));
    }

    #[test]
    fn test_mixed_syntax() {
        let result = parse_extended_inlines("E=mc^2^ :smile: and ==highlight==");
        assert!(result.iter().any(|i| matches!(i, Inline::Superscript(_))));
        assert!(result.iter().any(|i| matches!(i, Inline::Emoji { .. })));
        assert!(result.iter().any(|i| matches!(i, Inline::Highlight(_))));
    }

    #[test]
    fn test_nested_delimiters() {
        let result = parse_extended_inlines("^x^2^^");
        // Should parse "x" as superscript, then regular "2^^"
        assert!(matches!(result.first(), Some(Inline::Superscript(_))));
    }

    #[test]
    fn test_escaped_delimiter() {
        let result = parse_extended_inlines("^escaped\\^caret^");
        if let Some(Inline::Superscript(inner)) = result.first() {
            assert_eq!(inner, &vec![Inline::Text("escaped^caret".to_string())]);
        }
    }

    #[test]
    fn test_empty_delimiters() {
        let result = parse_extended_inlines("^^");
        // Empty superscript should be parsed
        assert!(matches!(result.first(), Some(Inline::Superscript(_))));
    }

    #[test]
    fn test_unclosed_delimiter() {
        let result = parse_extended_inlines("^unclosed");
        // Should be treated as regular text
        assert!(matches!(result.first(), Some(Inline::Text(_))));
    }

    #[test]
    fn test_distinguish_subscript_from_strikethrough() {
        let result = parse_extended_inlines("H~2~O not ~~strike~~");
        // First should be subscript
        assert!(matches!(result.get(1), Some(Inline::Subscript(_))));
        // Note: Strikethrough ~~text~~ would be handled by the main parser,
        // not this extended inline parser
    }
}
