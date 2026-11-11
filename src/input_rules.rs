//! Narrow, source-preserving input conversions shared by Source and Visual Edit.

use std::ops::Range;

use crate::AutoPairAction;

/// Convert a recognized URL immediately before typed whitespace into one exact
/// Markdown link. Callers remain responsible for excluding restricted fields
/// and IME composition.
pub fn url_auto_link_action(
    text: &str,
    range: Range<usize>,
    new_text: &str,
) -> Option<AutoPairAction> {
    if !range.is_empty() || !matches!(new_text, " " | "\n" | "\r\n") {
        return None;
    }
    let cursor = range.start;
    if cursor > text.len() || !text.is_char_boundary(cursor) {
        return None;
    }
    let line_start = text[..cursor].rfind('\n').map_or(0, |index| index + 1);
    let token_start = text[line_start..cursor]
        .char_indices()
        .rev()
        .find(|(_, character)| character.is_whitespace())
        .map_or(line_start, |(index, character)| {
            line_start + index + character.len_utf8()
        });
    let token = &text[token_start..cursor];
    let destination = if token.starts_with("https://") || token.starts_with("http://") {
        token
    } else if token.starts_with("www.") {
        return convert(
            token,
            format!("https://{token}"),
            token_start,
            cursor,
            new_text,
        );
    } else {
        return None;
    };
    convert(
        token,
        destination.to_string(),
        token_start,
        cursor,
        new_text,
    )
}

fn convert(
    token: &str,
    destination: String,
    start: usize,
    end: usize,
    suffix: &str,
) -> Option<AutoPairAction> {
    let host = destination
        .split_once("://")
        .map(|(_, host)| host)
        .unwrap_or_default();
    if host.is_empty()
        || !host.contains('.')
        || host.starts_with('.')
        || token.chars().any(|character| {
            character.is_whitespace() || matches!(character, '[' | ']' | '(' | ')' | '<' | '>')
        })
    {
        return None;
    }
    let replacement = format!("[{token}]({destination}){suffix}");
    let selection = start + replacement.len();
    Some(AutoPairAction::Replace {
        range: start..end,
        replacement,
        selection_after: selection..selection,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action_at(text: &str, cursor: usize) -> Option<AutoPairAction> {
        url_auto_link_action(text, cursor..cursor, " ")
    }

    #[test]
    fn converts_http_and_www_tokens() {
        let text = "see https://example.com";
        let Some(AutoPairAction::Replace {
            range,
            replacement,
            selection_after,
        }) = action_at(text, text.len())
        else {
            panic!("expected conversion")
        };
        assert_eq!(range, 4..text.len());
        assert_eq!(replacement, "[https://example.com](https://example.com) ");
        assert_eq!(selection_after.start, 4 + replacement.len());

        let text = "www.example.org";
        let Some(AutoPairAction::Replace {
            replacement: converted,
            ..
        }) = action_at(text, text.len())
        else {
            panic!("expected conversion")
        };
        assert_eq!(converted, "[www.example.org](https://www.example.org) ");
    }

    #[test]
    fn rejects_partial_and_non_url_tokens() {
        for text in [
            "https://",
            "example.com",
            "http://localhost",
            "see example.com",
        ] {
            assert!(action_at(text, text.len()).is_none(), "{text}");
        }
        assert!(action_at("https://example.com", 0).is_none());
        assert!(url_auto_link_action("https://example.com", 0..3, " ").is_none());
        assert!(url_auto_link_action("https://example.com", 23..23, "x").is_none());
    }
}
