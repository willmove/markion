//! Pure UTF-8 CF_HTML codec, also testable without constructing a native window.
//! https://learn.microsoft.com/en-us/windows/win32/dataxchg/html-clipboard-format

/// Offsets count bytes in the complete payload; the final NUL is outside HTML.
pub(crate) fn encode_cf_html(fragment: &str) -> Vec<u8> {
    const PREFIX: &str = "<html><body><!--StartFragment-->";
    const SUFFIX: &str = "<!--EndFragment--></body></html>";
    const HEADER: &str = concat!(
        "Version:1.0\r\n",
        "StartHTML:0000000000\r\n",
        "EndHTML:0000000000\r\n",
        "StartFragment:0000000000\r\n",
        "EndFragment:0000000000\r\n",
    );
    let start_html = HEADER.len();
    let start_fragment = start_html + PREFIX.len();
    let end_fragment = start_fragment + fragment.len();
    let end_html = end_fragment + SUFFIX.len();
    let header = format!(
        "Version:1.0\r\nStartHTML:{start_html:010}\r\nEndHTML:{end_html:010}\r\nStartFragment:{start_fragment:010}\r\nEndFragment:{end_fragment:010}\r\n"
    );
    debug_assert_eq!(header.len(), HEADER.len());
    let mut bytes = Vec::with_capacity(end_html + 1);
    bytes.extend_from_slice(header.as_bytes());
    bytes.extend_from_slice(PREFIX.as_bytes());
    bytes.extend_from_slice(fragment.as_bytes());
    bytes.extend_from_slice(SUFFIX.as_bytes());
    bytes.push(0);
    bytes
}

/// Invalid offsets retain the original UTF-8 payload; invalid UTF-8 lets callers
/// fall back to Unicode text instead of importing replacement characters.
pub(crate) fn extract_cf_html_fragment(bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8(bytes.to_vec()).ok()?;
    let (mut start, mut end) = (None, None);
    for line in text.lines().take(16) {
        if let Some(value) = line.strip_prefix("StartFragment:") {
            start = value.trim().parse::<usize>().ok();
        } else if let Some(value) = line.strip_prefix("EndFragment:") {
            end = value.trim().parse::<usize>().ok();
        } else if line.starts_with('<') {
            break;
        }
    }
    match (start, end) {
        (Some(start), Some(end))
            if start <= end
                && end <= text.len()
                && text.is_char_boundary(start)
                && text.is_char_boundary(end) =>
        {
            Some(text[start..end].to_string())
        }
        _ => Some(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_fragment_round_trip_and_byte_offsets() {
        let fragment =
            "<p>中文 <strong>😀 e\u{301}</strong></p><table><tr><td>格</td></tr></table>";
        let bytes = encode_cf_html(fragment);
        assert_eq!(bytes.last(), Some(&0));
        let payload = &bytes[..bytes.len() - 1];
        assert_eq!(extract_cf_html_fragment(payload).as_deref(), Some(fragment));
        let text = std::str::from_utf8(payload).unwrap();
        let offset = |name: &str| -> usize {
            text.lines()
                .find_map(|line| line.strip_prefix(name))
                .unwrap()
                .trim()
                .parse()
                .unwrap()
        };
        assert!(text[offset("StartHTML:")..].starts_with("<html>"));
        assert_eq!(offset("EndHTML:"), payload.len());
        assert_eq!(
            &text[offset("StartFragment:")..offset("EndFragment:")],
            fragment
        );
    }

    #[test]
    fn empty_fragment_and_embedded_marker_round_trip() {
        for fragment in ["", "<p><!--EndFragment-->中文</p>"] {
            let bytes = encode_cf_html(fragment);
            assert_eq!(
                extract_cf_html_fragment(&bytes[..bytes.len() - 1]).as_deref(),
                Some(fragment)
            );
        }
    }

    #[test]
    fn malformed_input_falls_back_without_splitting_utf8() {
        assert_eq!(
            extract_cf_html_fragment(b"<p>plain</p>"),
            Some("<p>plain</p>".into())
        );
        assert_eq!(extract_cf_html_fragment(&[0xff, 0xfe]), None);
        let invalid = "StartFragment:1\nEndFragment:999999\n<p>中文</p>";
        assert_eq!(
            extract_cf_html_fragment(invalid.as_bytes()).as_deref(),
            Some(invalid)
        );
        let reversed = "StartFragment:9\nEndFragment:1\n<p>text</p>";
        assert_eq!(
            extract_cf_html_fragment(reversed.as_bytes()).as_deref(),
            Some(reversed)
        );
        let encoded = encode_cf_html("中");
        let text = std::str::from_utf8(&encoded[..encoded.len() - 1]).unwrap();
        let start = text.find('中').unwrap();
        let split = text.replacen(
            &format!("StartFragment:{start:010}"),
            &format!("StartFragment:{:010}", start + 1),
            1,
        );
        assert_eq!(
            extract_cf_html_fragment(split.as_bytes()).as_deref(),
            Some(split.as_str())
        );
    }
}
