//! Forgiving tokenizer, entity decoder, and tree builder for
//! machine-generated HTML. The parser never fails: malformed constructs are
//! skipped, recovered, or treated as literal text.

/// Pathological nesting is flattened past this depth so recursive rendering
/// stays bounded.
const MAX_DEPTH: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Node {
    Text(String),
    Element(Element),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Element {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Node>,
}

impl Element {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

pub(crate) fn is_void_tag(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "source"
            | "track"
            | "wbr"
    )
}

fn is_raw_text_tag(tag: &str) -> bool {
    matches!(tag, "script" | "style")
}

/// Tags whose opening closes an open `p` (the HTML5 "close a p element" set,
/// trimmed to what clipboard HTML realistically carries).
fn closes_paragraph(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "center"
            | "dd"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hr"
            | "li"
            | "main"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "ul"
            | "o:p"
    )
}

struct OpenTag {
    name: String,
    attrs: Vec<(String, String)>,
    self_closing: bool,
    end: usize,
}

pub(crate) fn parse(html: &str) -> Element {
    // The element stack; index 0 is the document root and is never popped.
    let mut stack = vec![Element::default()];
    let bytes = html.as_bytes();
    let mut pos = 0usize;
    while pos < bytes.len() {
        if bytes[pos] != b'<' {
            let end = html[pos..]
                .find('<')
                .map(|rel| pos + rel)
                .unwrap_or(bytes.len());
            push_text(&mut stack, &decode_entities(&html[pos..end]));
            pos = end;
            continue;
        }
        match bytes.get(pos + 1) {
            Some(b'!') if html[pos..].starts_with("<!--") => {
                // Comments, including Word conditional comments ending in
                // `<![endif]-->`: everything through the first `-->` goes away.
                match html[pos + 4..].find("-->") {
                    Some(rel) => pos += 4 + rel + 3,
                    None => break,
                }
            }
            Some(b'!') if html[pos..].starts_with("<![CDATA[") => {
                match html[pos + 9..].find("]]>") {
                    Some(rel) => {
                        push_text(&mut stack, &html[pos + 9..pos + 9 + rel]);
                        pos += 9 + rel + 3;
                    }
                    None => {
                        push_text(&mut stack, &html[pos + 9..]);
                        break;
                    }
                }
            }
            Some(b'!') | Some(b'?') => {
                // Doctype, other declarations, processing instructions.
                match html[pos..].find('>') {
                    Some(rel) => pos += rel + 1,
                    None => break,
                }
            }
            Some(b'/') => {
                let (name, end) = scan_close_tag(html, pos);
                pos = end;
                close_element(&mut stack, &name);
            }
            Some(ch) if ch.is_ascii_alphabetic() => match scan_open_tag(html, pos) {
                Some(tag) => {
                    pos = tag.end;
                    if is_raw_text_tag(&tag.name) {
                        // `script`/`style` content is dropped through the
                        // matching close tag; an unterminated one swallows
                        // the rest of the input.
                        match find_raw_text_close(html, pos, &tag.name) {
                            Some(end) => pos = end,
                            None => break,
                        }
                        continue;
                    }
                    open_element(&mut stack, tag);
                }
                None => {
                    push_text(&mut stack, "<");
                    pos += 1;
                }
            },
            _ => {
                push_text(&mut stack, "<");
                pos += 1;
            }
        }
    }
    // Unclosed elements still open at end of input keep their content.
    pop_to(&mut stack, 1);
    stack.into_iter().next().unwrap_or_default()
}

fn push_text(stack: &mut [Element], text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(top) = stack.last_mut() {
        if let Some(Node::Text(existing)) = top.children.last_mut() {
            existing.push_str(text);
            return;
        }
        top.children.push(Node::Text(text.to_owned()));
    }
}

fn open_element(stack: &mut Vec<Element>, tag: OpenTag) {
    match tag.name.as_str() {
        "li" => close_implied(stack, &["ul", "ol"], &["li"]),
        "td" | "th" => close_implied(
            stack,
            &["tr", "table", "thead", "tbody", "tfoot"],
            &["td", "th"],
        ),
        "tr" => close_implied(stack, &["table", "thead", "tbody", "tfoot"], &["tr"]),
        _ => {}
    }
    if closes_paragraph(&tag.name) && stack.last().map(|top| top.tag.as_str()) == Some("p") {
        pop_to(stack, stack.len() - 1);
    }
    let element = Element {
        tag: tag.name,
        attrs: tag.attrs,
        children: Vec::new(),
    };
    if tag.self_closing || is_void_tag(&element.tag) {
        if let Some(top) = stack.last_mut() {
            top.children.push(Node::Element(element));
        }
        return;
    }
    if stack.len() > MAX_DEPTH {
        // Flatten absurd nesting: drop the wrapper, keep parsing its
        // children into the current parent.
        return;
    }
    stack.push(element);
}

/// Closes an implied open element (e.g. a new `li` closes the previous `li`)
/// by scanning the stack downward, stopping at scope boundaries.
fn close_implied(stack: &mut Vec<Element>, boundaries: &[&str], targets: &[&str]) {
    for index in (1..stack.len()).rev() {
        let tag = stack[index].tag.as_str();
        if boundaries.contains(&tag) {
            return;
        }
        if targets.contains(&tag) {
            pop_to(stack, index);
            return;
        }
    }
}

/// A close tag pops up to and including the matching open element; a close
/// tag with no matching open element is ignored.
fn close_element(stack: &mut Vec<Element>, name: &str) {
    if name.is_empty() {
        return;
    }
    for index in (1..stack.len()).rev() {
        if stack[index].tag == name {
            pop_to(stack, index);
            return;
        }
    }
}

/// Pops every open element above `index`, attaching each to its parent's
/// children so completed subtrees keep their content.
fn pop_to(stack: &mut Vec<Element>, index: usize) {
    while stack.len() > index {
        if let Some(element) = stack.pop()
            && let Some(top) = stack.last_mut()
        {
            top.children.push(Node::Element(element));
        }
    }
}

fn is_tag_name_char(byte: u8) -> bool {
    !byte.is_ascii_whitespace() && byte != b'/' && byte != b'>'
}

/// Scans an open tag starting at `start` (which points at `<` followed by an
/// ASCII letter). Returns `None` for unterminated tags so the caller can
/// treat the `<` as literal text.
fn scan_open_tag(src: &str, start: usize) -> Option<OpenTag> {
    let bytes = src.as_bytes();
    let mut index = start + 1;
    let name_start = index;
    while index < bytes.len() && is_tag_name_char(bytes[index]) {
        index += 1;
    }
    let name = src[name_start..index].to_ascii_lowercase();
    let mut attrs: Vec<(String, String)> = Vec::new();
    loop {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() {
            return None;
        }
        match bytes[index] {
            b'>' => {
                return Some(OpenTag {
                    name,
                    attrs,
                    self_closing: false,
                    end: index + 1,
                });
            }
            b'/' => {
                if bytes.get(index + 1) == Some(&b'>') {
                    return Some(OpenTag {
                        name,
                        attrs,
                        self_closing: true,
                        end: index + 2,
                    });
                }
                index += 1;
            }
            _ => {
                let attr_start = index;
                while index < bytes.len()
                    && !bytes[index].is_ascii_whitespace()
                    && !matches!(bytes[index], b'=' | b'>' | b'/')
                {
                    index += 1;
                }
                let attr_name = src[attr_start..index].to_ascii_lowercase();
                while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                    index += 1;
                }
                let mut attr_value = String::new();
                if index < bytes.len() && bytes[index] == b'=' {
                    index += 1;
                    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                        index += 1;
                    }
                    if index < bytes.len() && matches!(bytes[index], b'"' | b'\'') {
                        let quote = bytes[index];
                        index += 1;
                        let value_start = index;
                        while index < bytes.len() && bytes[index] != quote {
                            index += 1;
                        }
                        attr_value = decode_entities(&src[value_start..index]);
                        if index < bytes.len() {
                            index += 1;
                        }
                    } else {
                        let value_start = index;
                        while index < bytes.len()
                            && !bytes[index].is_ascii_whitespace()
                            && bytes[index] != b'>'
                        {
                            index += 1;
                        }
                        attr_value = decode_entities(&src[value_start..index]);
                    }
                }
                if !attr_name.is_empty() && !attrs.iter().any(|(key, _)| *key == attr_name) {
                    attrs.push((attr_name, attr_value));
                }
            }
        }
    }
}

fn scan_close_tag(src: &str, start: usize) -> (String, usize) {
    let bytes = src.as_bytes();
    let mut index = start + 2;
    let name_start = index;
    while index < bytes.len() && is_tag_name_char(bytes[index]) {
        index += 1;
    }
    let name = src[name_start..index].to_ascii_lowercase();
    while index < bytes.len() && bytes[index] != b'>' {
        index += 1;
    }
    let end = if index < bytes.len() {
        index + 1
    } else {
        bytes.len()
    };
    (name, end)
}

/// Finds the close tag of a raw-text element (`script`/`style`) starting the
/// search at `from`; returns the position just past it.
fn find_raw_text_close(src: &str, from: usize, name: &str) -> Option<usize> {
    let needle = format!("</{name}");
    let rel = find_ascii_case_insensitive(src.get(from..)?, &needle)?;
    let tag_start = from + rel;
    let close = src[tag_start..].find('>')?;
    Some(tag_start + close + 1)
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    let hay = haystack.as_bytes();
    let ned = needle.as_bytes();
    if ned.is_empty() || hay.len() < ned.len() {
        return None;
    }
    hay.windows(ned.len())
        .position(|window| window.eq_ignore_ascii_case(ned))
}

pub(crate) fn decode_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];
        match decode_entity(after) {
            Some((decoded, consumed)) => {
                out.push_str(&decoded);
                rest = &after[consumed..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Decodes the entity at the start of `after_amp` (the text right after `&`),
/// returning the replacement and the consumed length including the `;`.
/// Unknown entities pass through literally.
fn decode_entity(after_amp: &str) -> Option<(String, usize)> {
    let semi = after_amp.find(';')?;
    let candidate = &after_amp[..semi];
    if candidate.is_empty()
        || candidate.len() > 32
        || !candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '#')
    {
        return None;
    }
    if let Some(number) = candidate.strip_prefix('#') {
        let value = if let Some(hex) = number.strip_prefix(|ch| ch == 'x' || ch == 'X') {
            u32::from_str_radix(hex, 16).ok()?
        } else {
            number.parse::<u32>().ok()?
        };
        if value == 0 {
            return None;
        }
        return char::from_u32(value).map(|ch| (ch.to_string(), semi + 1));
    }
    named_entity(candidate).map(|replacement| (replacement.to_owned(), semi + 1))
}

fn named_entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        "nbsp" | "ensp" | "emsp" | "thinsp" => " ",
        "copy" => "©",
        "reg" => "®",
        "trade" => "™",
        "hellip" => "…",
        "mdash" => "—",
        "ndash" => "–",
        "lsquo" => "\u{2018}",
        "rsquo" => "\u{2019}",
        "ldquo" => "\u{201C}",
        "rdquo" => "\u{201D}",
        "laquo" => "«",
        "raquo" => "»",
        "times" => "×",
        "divide" => "÷",
        "plusmn" => "±",
        "deg" => "°",
        "para" => "¶",
        "sect" => "§",
        "middot" => "·",
        "bull" => "•",
        "dagger" => "†",
        "Dagger" => "‡",
        "permil" => "‰",
        "euro" => "€",
        "pound" => "£",
        "yen" => "¥",
        "cent" => "¢",
        "sup1" => "¹",
        "sup2" => "²",
        "sup3" => "³",
        "frac12" => "½",
        "frac14" => "¼",
        "frac34" => "¾",
        "iexcl" => "¡",
        "iquest" => "¿",
        "szlig" => "ß",
        "agrave" => "à",
        "aacute" => "á",
        "egrave" => "è",
        "eacute" => "é",
        "uml" => "¨",
        "ccedil" => "¸",
        "atilde" => "ã",
        "oslash" => "ø",
        "aring" => "å",
        "aelig" => "æ",
        "oelig" => "œ",
        "OElig" => "Œ",
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" => "ε",
        "theta" => "θ",
        "lambda" => "λ",
        "mu" => "μ",
        "pi" => "π",
        "sigma" => "σ",
        "phi" => "φ",
        "omega" => "ω",
        "Alpha" => "Α",
        "Beta" => "Β",
        "Gamma" => "Γ",
        "Delta" => "Δ",
        "Theta" => "Θ",
        "Lambda" => "Λ",
        "Pi" => "Π",
        "Sigma" => "Σ",
        "Phi" => "Φ",
        "Omega" => "Ω",
        "infin" => "∞",
        "ne" => "≠",
        "le" => "≤",
        "ge" => "≥",
        "minus" => "−",
        "asymp" => "≈",
        "larr" => "←",
        "uarr" => "↑",
        "rarr" => "→",
        "darr" => "↓",
        "harr" => "↔",
        "spades" => "♠",
        "clubs" => "♣",
        "hearts" => "♥",
        "diams" => "♦",
        _ => return None,
    })
}
