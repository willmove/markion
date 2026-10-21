//! Update warm counters using only the edited text and adjacent characters.
use crate::model::DocumentBasicStats;
use std::ops::Range;

pub(crate) fn after_edit(
    source: &str,
    range: Range<usize>,
    replacement: &str,
    old: DocumentBasicStats,
) -> DocumentBasicStats {
    let removed = &source[range.clone()];
    let before = source[..range.start].chars().next_back();
    let after = source[range.end..].chars().next();
    // Include the following character: a splice can join or split a word.
    let starts = |text: &str| {
        let mut previous_space = before.is_none_or(char::is_whitespace);
        let mut count = 0;
        for ch in text.chars().chain(after) {
            let space = ch.is_whitespace();
            count += usize::from(previous_space && !space);
            previous_space = space;
        }
        count
    };
    let old_newlines = if source.is_empty() {
        0
    } else {
        old.lines - usize::from(!source.ends_with('\n'))
    };
    let newlines = old_newlines - removed.bytes().filter(|&b| b == b'\n').count()
        + replacement.bytes().filter(|&b| b == b'\n').count();
    let bytes = source.len() - removed.len() + replacement.len();
    let last = if range.end < source.len() {
        source.as_bytes().last().copied()
    } else if !replacement.is_empty() {
        replacement.as_bytes().last().copied()
    } else {
        source.as_bytes()[..range.start].last().copied()
    };
    DocumentBasicStats {
        bytes,
        chars: old.chars - removed.chars().count() + replacement.chars().count(),
        words: old.words - starts(removed) + starts(replacement),
        lines: (newlines + usize::from(bytes > 0 && last != Some(b'\n'))).max(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full(text: &str) -> DocumentBasicStats {
        DocumentBasicStats {
            bytes: text.len(),
            chars: text.chars().count(),
            words: text.split_whitespace().count(),
            lines: text.lines().count().max(1),
        }
    }

    #[test]
    fn every_splice_matches_full_unicode_and_newline_counts() {
        let samples = [
            "",
            "\n",
            "\r",
            "\r\n",
            "a\r\nb\n",
            "日本語入力",
            "a b",
            "  a  b  ",
            "a\u{3000}日本\u{a0}語",
            "👨‍👩‍👧‍👦e\u{301}\n",
        ];
        let replacements = [
            "",
            "x",
            "日本語",
            " ",
            "\n",
            "\r",
            "\r\n",
            "\u{3000}",
            "a b\n",
            "🙂",
        ];
        for source in samples {
            let boundaries: Vec<_> = (0..=source.len())
                .filter(|&i| source.is_char_boundary(i))
                .collect();
            for &start in &boundaries {
                for &end in boundaries.iter().filter(|&&i| i >= start) {
                    for replacement in replacements {
                        let actual = after_edit(source, start..end, replacement, full(source));
                        let mut expected = source.to_owned();
                        expected.replace_range(start..end, replacement);
                        assert_eq!(
                            actual,
                            full(&expected),
                            "{source:?} {start}..{end} {replacement:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn repeated_edits_keep_counters_exact() {
        let mut text = "# 日本語\r\n\n本文 **太字**\n".to_owned();
        let mut counts = full(&text);
        let mut seed = 91u64;
        let fragments = [
            "にほんご",
            "日本語",
            "入力",
            "",
            " ",
            "\n",
            "\r",
            "\u{2003}",
            "🙂",
            "**",
        ];
        for _ in 0..5000 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let boundaries: Vec<_> = (0..=text.len())
                .filter(|&i| text.is_char_boundary(i))
                .collect();
            let a = seed as usize % boundaries.len();
            let b = (seed >> 32) as usize % boundaries.len();
            let range = boundaries[a.min(b)]..boundaries[a.max(b)];
            let replacement = fragments[(seed >> 16) as usize % fragments.len()];
            counts = after_edit(&text, range.clone(), replacement, counts);
            text.replace_range(range, replacement);
            assert_eq!(counts, full(&text));
        }
    }
}
