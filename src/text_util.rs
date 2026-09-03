//! Small string/offset helpers shared across parsing, editing, and rendering.

/// Clamp `index` to the nearest UTF-8 character boundary at or before it.
pub fn clamp_to_char_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// Clamp both ends of a range to character boundaries, keeping `end >= start`.
pub fn clamp_range_to_char_boundaries(
    text: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let start = clamp_to_char_boundary(text, range.start);
    let end = clamp_to_char_boundary(text, range.end).max(start);
    start..end
}

/// Add a signed delta to an offset without underflow.
pub fn offset_with_delta(offset: usize, delta: isize) -> usize {
    if delta >= 0 {
        offset + delta as usize
    } else {
        offset.saturating_sub((-delta) as usize)
    }
}

/// 1-based (line, column) for a byte offset, clamped to a char boundary.
pub fn line_column_at(text: &str, offset: usize) -> (usize, usize) {
    let offset = clamp_to_char_boundary(text, offset);
    let line_start = text[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line = text[..offset].bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = text[line_start..offset].chars().count() + 1;
    (line, column)
}

/// Trimmed text of the 1-based `line_number`, or empty if out of range.
pub fn line_snippet_at(text: &str, line_number: usize) -> String {
    text.lines()
        .nth(line_number.saturating_sub(1))
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Maximal contiguous run of same-class characters around `offset`.
///
/// Characters are classified as word, whitespace, or punctuation. Word
/// characters split by script — ASCII letters/digits and CJK ideographs or
/// kana form separate runs (so double-clicking an embedded acronym selects
/// just the acronym), while other alphanumeric characters (e.g. `é`, `й`)
/// join whichever word run they touch. The run is anchored on the character
/// starting at the clamped `offset`, falling back to the final character at
/// end of text.
pub fn char_run_range(text: &str, offset: usize) -> std::ops::Range<usize> {
    #[derive(PartialEq, Eq, Clone, Copy)]
    enum CharRunClass {
        AsciiWord,
        CjkWord,
        OtherWord,
        Whitespace,
        Punctuation,
    }

    fn class_of(character: char) -> CharRunClass {
        if character.is_ascii_alphanumeric() {
            CharRunClass::AsciiWord
        } else if is_cjk_word_char(character) {
            CharRunClass::CjkWord
        } else if character.is_alphanumeric() {
            CharRunClass::OtherWord
        } else if character.is_whitespace() {
            CharRunClass::Whitespace
        } else {
            CharRunClass::Punctuation
        }
    }

    fn is_cjk_word_char(character: char) -> bool {
        matches!(character,
            '\u{4E00}'..='\u{9FFF}'    // CJK Unified Ideographs
            | '\u{3400}'..='\u{4DBF}'  // CJK Extension A
            | '\u{F900}'..='\u{FAFF}'  // CJK Compatibility Ideographs
            | '\u{3040}'..='\u{30FF}'  // Hiragana + Katakana
            | '\u{AC00}'..='\u{D7AF}') // Hangul Syllables
    }

    fn is_word_class(class: CharRunClass) -> bool {
        matches!(
            class,
            CharRunClass::AsciiWord | CharRunClass::CjkWord | CharRunClass::OtherWord
        )
    }

    // Two adjacent word characters join the same run when they share a
    // script or when either is a non-ASCII, non-CJK letter/digit that has
    // no script of its own to split on (caf + é = café).
    fn same_run(previous: char, next: char) -> bool {
        let (previous_class, next_class) = (class_of(previous), class_of(next));
        previous_class == next_class
            || (is_word_class(previous_class)
                && is_word_class(next_class)
                && (previous_class == CharRunClass::OtherWord
                    || next_class == CharRunClass::OtherWord))
    }

    let offset = clamp_to_char_boundary(text, offset);
    if text.is_empty() {
        return offset..offset;
    }
    let (anchor, anchor_character) = if offset < text.len() {
        (
            offset,
            text[offset..].chars().next().expect("char boundary"),
        )
    } else {
        let character = text[..offset].chars().next_back().expect("non-empty");
        (offset - character.len_utf8(), character)
    };
    let mut start = anchor;
    while start > 0 {
        let previous = text[..start].chars().next_back().expect("char boundary");
        let current = text[start..].chars().next().expect("char boundary");
        if !same_run(previous, current) {
            break;
        }
        start -= previous.len_utf8();
    }
    let mut end = anchor + anchor_character.len_utf8();
    while end < text.len() {
        let previous = text[..end].chars().next_back().expect("char boundary");
        let next = text[end..].chars().next().expect("char boundary");
        if !same_run(previous, next) {
            break;
        }
        end += next.len_utf8();
    }
    start..end
}

#[cfg(test)]
mod tests {
    use super::char_run_range;

    fn run_at(text: &str, offset: usize) -> &str {
        &text[char_run_range(text, offset)]
    }

    #[test]
    fn word_run_covers_mid_and_edges() {
        let text = "hello world";
        assert_eq!(run_at(text, 2), "hello");
        assert_eq!(run_at(text, 0), "hello");
        assert_eq!(run_at(text, 4), "hello");
        assert_eq!(run_at(text, 6), "world");
        assert_eq!(run_at(text, 10), "world");
        assert_eq!(run_at(text, text.len()), "world");
    }

    #[test]
    fn boundary_hit_resolves_the_character_at_the_offset() {
        // Offset 5 is the space: deterministic right-side rule selects the
        // whitespace run, not the preceding word.
        assert_eq!(run_at("hello world", 5), " ");
    }

    #[test]
    fn digit_runs_form_their_own_words() {
        assert_eq!(run_at("abc 12345 x", 7), "12345");
    }

    #[test]
    fn cjk_run_is_bounded_by_whitespace_and_punctuation() {
        let text = "今天 内存疯了，不错";
        assert_eq!(run_at(text, 10), "内存疯了");
        assert_eq!(run_at(text, 7), "内存疯了");
        assert_eq!(run_at(text, 0), "今天");
        let text = "内存，疯了";
        assert_eq!(run_at(text, 6), "，");
        assert_eq!(run_at(text, 9), "疯了");
    }

    #[test]
    fn latin_cjk_transition_splits_runs() {
        let text = "使用HBM显存";
        assert_eq!(run_at(text, 1), "使用");
        assert_eq!(run_at(text, 7), "HBM");
        assert_eq!(run_at(text, 10), "显存");
    }

    #[test]
    fn accented_latin_joins_the_adjacent_word() {
        assert_eq!(run_at("café latte", 3), "café");
        assert_eq!(run_at("naïve", 2), "naïve");
    }

    #[test]
    fn punctuation_and_whitespace_form_runs() {
        assert_eq!(run_at("wait... ok", 5), "...");
        assert_eq!(run_at("a——b", 2), "——");
        assert_eq!(run_at("a   b", 2), "   ");
        assert_eq!(run_at("a \t b", 2), " \t ");
        assert_eq!(run_at("snake_case", 5), "_");
    }

    #[test]
    fn empty_text_returns_empty_range() {
        assert_eq!(char_run_range("", 0), 0..0);
        assert_eq!(char_run_range("", 5), 0..0);
    }

    #[test]
    fn mid_codepoint_offsets_clamp_to_the_containing_character() {
        // 怎 occupies bytes 1..4 in "a怎么".
        assert_eq!(run_at("a怎么", 2), "怎么");
        assert_eq!(run_at("怎么a", 1), "怎么");
    }

    #[test]
    fn offsets_beyond_the_text_clamp_to_the_final_run() {
        assert_eq!(run_at("hello", 100), "hello");
    }
}
