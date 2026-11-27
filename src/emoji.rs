//! Maintained emoji shortcode table and `:` completer query.
//!
//! Canonical source stays `:name:`; preview and Visual Edit render the glyph
//! through the shared catalog. No network, no extra crate.

use std::{collections::BTreeMap, ops::Range, sync::LazyLock};

/// Existing application aliases override the shared catalog's glyph choices
/// for compatibility. Names use the ASCII shortcode charset
/// (lowercase, digits, `_`, `-`, `+`).
const LEGACY_EMOJI_SHORTCODES: &[(&str, &str)] = &[
    ("+1", "👍"),
    ("-1", "👎"),
    ("100", "💯"),
    ("book", "📘"),
    ("bug", "🐛"),
    ("bulb", "💡"),
    ("check", "✅"),
    ("clap", "👏"),
    ("eyes", "👀"),
    ("fire", "🔥"),
    ("heart", "❤️"),
    ("idea", "💡"),
    ("joy", "😂"),
    ("laughing", "😆"),
    ("memo", "📝"),
    ("ok", "👌"),
    ("pray", "🙏"),
    ("rocket", "🚀"),
    ("slightly_smiling_face", "🙂"),
    ("smile", "🙂"),
    ("sparkles", "✨"),
    ("star", "⭐"),
    ("tada", "🎉"),
    ("thinking", "🤔"),
    ("thumbsdown", "👎"),
    ("thumbsup", "👍"),
    ("warning", "⚠️"),
    ("wave", "👋"),
    ("white_check_mark", "✅"),
    ("wink", "😉"),
    ("x", "❌"),
    ("zap", "⚡"),
];

static EMOJI_SHORTCODES: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    let mut catalog: BTreeMap<_, _> = typune_markdown::emoji::shortcodes().collect();
    // Preserve existing document glyphs and aliases when adding the broader catalog.
    catalog.extend(LEGACY_EMOJI_SHORTCODES.iter().copied());
    catalog.into_iter().collect()
});

/// Open `:query` token used by the emoji completer palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmojiQuery {
    pub document_version: u64,
    /// Byte range of the open token, from the triggering `:` through the
    /// current caret (no closing colon yet).
    pub source_range: Range<usize>,
    pub query: String,
}

pub fn emoji_for_shortcode(shortcode: &str) -> Option<&'static str> {
    if !is_valid_shortcode_name(shortcode) {
        return None;
    }
    EMOJI_SHORTCODES
        .binary_search_by_key(&shortcode, |(name, _)| *name)
        .ok()
        .map(|index| EMOJI_SHORTCODES[index].1)
}

/// Prefix-filter the shortcode table. An empty prefix returns the full table
/// (still bounded by callers that cap visible rows). Names are already
/// lowercase; the needle is matched as typed ASCII.
pub fn emoji_shortcodes_matching(prefix: &str) -> Vec<(&'static str, &'static str)> {
    if !prefix.is_empty() && !is_valid_shortcode_name(prefix) {
        return Vec::new();
    }
    EMOJI_SHORTCODES
        .iter()
        .copied()
        .filter(|(name, _)| prefix.is_empty() || name.starts_with(prefix))
        .collect()
}

pub fn is_valid_shortcode_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 32 && name.chars().all(is_shortcode_char)
}

pub fn is_shortcode_char(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-' | '+')
}

/// True when `offset` is a word-boundary `:` (start of text, whitespace, or
/// opening punctuation) and not the first colon of `://`.
fn colon_is_emoji_trigger(source: &str, colon: usize) -> bool {
    if colon >= source.len() || !source.is_char_boundary(colon) || !source[colon..].starts_with(':')
    {
        return false;
    }
    if source[colon + 1..].starts_with("//") {
        return false;
    }
    if colon == 0 {
        return true;
    }
    source[..colon]
        .chars()
        .next_back()
        .is_some_and(|ch| ch.is_whitespace() || matches!(ch, '(' | '[' | '{' | '`' | '"' | '\''))
}

/// Open-token query at `cursor` when the caret sits in or just after a
/// word-boundary `:shortcode` prefix (no closing colon required).
pub fn emoji_query_at(source: &str, cursor: usize, document_version: u64) -> Option<EmojiQuery> {
    if cursor > source.len() || !source.is_char_boundary(cursor) {
        return None;
    }
    let prefix = source.get(..cursor)?;
    let colon = prefix.rfind(':')?;
    if !colon_is_emoji_trigger(source, colon) {
        return None;
    }
    let query = &prefix[colon + 1..];
    if query.contains(['\r', '\n']) || query.chars().any(|ch| !is_shortcode_char(ch)) {
        return None;
    }
    let suffix_name_len = source[cursor..]
        .char_indices()
        .take_while(|(_, ch)| is_shortcode_char(*ch))
        .map(|(index, ch)| index + ch.len_utf8())
        .last()
        .unwrap_or(0);
    let suffix_end = cursor + suffix_name_len;
    let source_end = if source[suffix_end..].starts_with(':')
        && emoji_for_shortcode(&source[colon + 1..suffix_end]).is_some()
    {
        suffix_end + 1
    } else {
        cursor
    };
    Some(EmojiQuery {
        document_version,
        source_range: colon..source_end,
        query: query.to_string(),
    })
}

/// Replacement that turns an open `:query` into the canonical `:name:`.
pub fn emoji_confirm_replacement(name: &str) -> Option<String> {
    if emoji_for_shortcode(name).is_none() {
        return None;
    }
    Some(format!(":{name}:"))
}

/// Every `:name:` token in `text` that the maintained table recognizes.
/// Ranges are relative to `text` and land on UTF-8 boundaries.
pub fn emoji_tokens_in(text: &str) -> Vec<(Range<usize>, &'static str)> {
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < text.len() {
        let rest = &text[index..];
        if let Some(stripped) = rest.strip_prefix(':')
            && let Some(end) = stripped.find(':')
        {
            let shortcode = &stripped[..end];
            if let Some(glyph) = emoji_for_shortcode(shortcode) {
                tokens.push((index..index + end + 2, glyph));
                index += end + 2;
                continue;
            }
        }
        let next = rest.chars().next().expect("non-empty remainder");
        index += next.len_utf8();
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_shortcodes_round_trip() {
        assert_eq!(emoji_for_shortcode("smile"), Some("🙂"));
        assert_eq!(emoji_for_shortcode("heart"), Some("❤️"));
        assert_eq!(emoji_for_shortcode("+1"), Some("👍"));
        assert_eq!(emoji_for_shortcode("sparkles"), Some("✨"));
        assert_eq!(emoji_for_shortcode("unknown"), None);
        assert_eq!(emoji_for_shortcode(""), None);
        assert_eq!(emoji_for_shortcode("SMILE"), None);
        assert_eq!(emoji_for_shortcode("has space"), None);
    }

    #[test]
    fn shared_catalog_preserves_aliases_and_expands_rendering() {
        let all = emoji_shortcodes_matching("");
        assert!(all.len() >= 250);
        assert!(all.windows(2).all(|pair| pair[0].0 < pair[1].0));
        for (name, glyph) in &all {
            assert_eq!(emoji_for_shortcode(name), Some(*glyph));
            assert_eq!(emoji_confirm_replacement(name), Some(format!(":{name}:")));
        }
        for (name, glyph) in LEGACY_EMOJI_SHORTCODES {
            assert_eq!(emoji_for_shortcode(name), Some(*glyph));
        }
        assert_eq!(emoji_for_shortcode("rofl"), Some("🤣"));
        assert_eq!(emoji_for_shortcode("cat"), Some("🐱"));
        assert_eq!(emoji_for_shortcode("coffee"), Some("☕"));
        eprintln!("Resolved emoji catalog: {} shortcodes", all.len());
    }

    #[test]
    fn prefix_filter_includes_empty_and_no_match() {
        let all = emoji_shortcodes_matching("");
        assert!(all.len() >= 16);
        assert!(all.iter().any(|(name, _)| *name == "smile"));
        let smi = emoji_shortcodes_matching("smi");
        assert!(smi.contains(&("smile", "🙂")));
        assert!(smi.contains(&("smiley", "😃")));
        assert!(emoji_shortcodes_matching("zzz").is_empty());
        assert!(emoji_shortcodes_matching("smi!").is_empty());
        assert!(emoji_shortcodes_matching("Smile").is_empty());
    }

    #[test]
    fn charset_rejects_uppercase_and_punctuation() {
        assert!(is_valid_shortcode_name("white_check_mark"));
        assert!(is_valid_shortcode_name("+1"));
        assert!(is_valid_shortcode_name("-1"));
        assert!(!is_valid_shortcode_name("White"));
        assert!(!is_valid_shortcode_name("smile!"));
        assert!(!is_shortcode_char(':'));
        assert!(!is_shortcode_char(' '));
    }

    #[test]
    fn word_boundary_colon_opens_a_query() {
        let q = emoji_query_at(":smi", 4, 3).unwrap();
        assert_eq!(q.source_range, 0..4);
        assert_eq!(q.query, "smi");
        assert_eq!(q.document_version, 3);

        let q = emoji_query_at("hello :sm", 9, 1).unwrap();
        assert_eq!(q.query, "sm");
        assert_eq!(&"hello :sm"[q.source_range.clone()], ":sm");

        assert!(emoji_query_at(":", 1, 1).is_some());
        assert!(emoji_query_at("(:", 2, 1).is_some());
    }

    #[test]
    fn time_urls_and_alnum_colons_do_not_open() {
        assert!(emoji_query_at("12:30", 5, 1).is_none());
        assert!(emoji_query_at("12:30", 3, 1).is_none());
        assert!(emoji_query_at("https://example.com", 6, 1).is_none());
        assert!(emoji_query_at("https://example.com", 8, 1).is_none());
        assert!(emoji_query_at("foo:bar", 4, 1).is_none());
        assert!(emoji_query_at("a:smi", 5, 1).is_none());
    }

    #[test]
    fn query_covers_an_existing_recognized_shortcode_suffix() {
        for (source, cursor, expected) in [
            (":smile:", 6, 0..7),
            (":smile:", 4, 0..7),
            (":roc: :", 4, 0..4),
        ] {
            let query = emoji_query_at(source, cursor, 1).unwrap();
            assert_eq!(query.source_range, expected, "{source:?} at {cursor}");
        }
    }

    #[test]
    fn confirm_replacement_includes_closing_colon() {
        assert_eq!(
            emoji_confirm_replacement("smile").as_deref(),
            Some(":smile:")
        );
        assert_eq!(emoji_confirm_replacement("nope"), None);
    }

    #[test]
    fn tokens_in_prose_skip_unknown_names() {
        let text = "hi :smile: and :nope: :heart:";
        let tokens = emoji_tokens_in(text);
        assert_eq!(tokens.len(), 2);
        assert_eq!(&text[tokens[0].0.clone()], ":smile:");
        assert_eq!(tokens[0].1, "🙂");
        assert_eq!(&text[tokens[1].0.clone()], ":heart:");
    }
}
