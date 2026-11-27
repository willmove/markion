use roxmltree::Node;

pub(crate) const M_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

pub(crate) fn is_m(node: Node<'_, '_>, name: &str) -> bool {
    node.tag_name().namespace() == Some(M_NS) && node.tag_name().name() == name
}

pub(crate) fn omml_to_latex(node: Node<'_, '_>) -> Option<String> {
    let tex = expression(node)?;
    let tex = tex.trim();
    (!tex.is_empty()).then(|| tex.to_owned())
}

fn expression(node: Node<'_, '_>) -> Option<String> {
    let mut tex = String::new();
    for child in node.children().filter(Node::is_element) {
        if child.tag_name().namespace() != Some(M_NS) {
            continue;
        }
        let term = match child.tag_name().name() {
            "r" => run(child)?,
            "f" => format!(
                "\\frac{{{}}}{{{}}}",
                argument(child, "num")?,
                argument(child, "den")?,
            ),
            "rad" => {
                let degree = argument(child, "deg")?;
                let base = argument(child, "e")?;
                if degree.trim().is_empty() || property_on(child, "radPr", "degHide") {
                    format!("\\sqrt{{{base}}}")
                } else {
                    format!("\\sqrt[{degree}]{{{base}}}")
                }
            }
            "sSup" | "sSub" | "sSubSup" => {
                let mut base = script_base(argument(child, "e")?);
                append_script(&mut base, argument(child, "sub")?, '_');
                append_script(&mut base, argument(child, "sup")?, '^');
                base
            }
            "d" => {
                let open = delimiter(property(child, "dPr", "begChr").unwrap_or("("))?;
                let close = delimiter(property(child, "dPr", "endChr").unwrap_or(")"))?;
                let separator = property(child, "dPr", "sepChr").unwrap_or("|");
                let mut arguments = child
                    .children()
                    .filter(|node| is_m(*node, "e"))
                    .map(expression);
                let mut inner = arguments.next().unwrap_or_else(|| Some(String::new()))?;
                for argument in arguments {
                    inner.push_str("\\middle");
                    inner.push_str(delimiter(separator)?);
                    inner.push(' ');
                    inner.push_str(&argument?);
                }
                format!("\\left{open} {inner}\\right{close} ")
            }
            "nary" => nary(child)?,
            "acc" => {
                let accent = match property(child, "accPr", "chr").unwrap_or("\u{302}") {
                    "\u{302}" | "^" => "hat",
                    "\u{304}" | "\u{305}" | "¯" | "‾" => "overline",
                    "\u{303}" | "~" => "tilde",
                    "\u{307}" | "˙" => "dot",
                    "\u{308}" | "¨" => "ddot",
                    "\u{20d7}" | "→" => "vec",
                    _ => return None,
                };
                format!("\\{accent}{{{}}}", argument(child, "e")?)
            }
            "limLow" | "limUpp" => {
                let base = argument(child, "e")?;
                let limit = argument(child, "lim")?;
                if is_m(child, "limLow") {
                    format!("\\underset{{{limit}}}{{{base}}}")
                } else {
                    format!("\\overset{{{limit}}}{{{base}}}")
                }
            }
            "oMath" => expression(child)?,
            // Properties are read by their structural parent. They carry no formula text.
            "oMathParaPr" | "ctrlPr" => continue,
            _ => return None,
        };
        tex.push_str(&term);
    }
    Some(tex)
}

fn nary(node: Node<'_, '_>) -> Option<String> {
    let operator = match property(node, "naryPr", "chr").unwrap_or("∫") {
        "∫" => "\\int",
        "∬" => "\\iint",
        "∭" => "\\iiint",
        "∮" => "\\oint",
        "∑" => "\\sum",
        "∏" => "\\prod",
        "∐" => "\\coprod",
        "⋃" => "\\bigcup",
        "⋂" => "\\bigcap",
        "⋁" => "\\bigvee",
        "⋀" => "\\bigwedge",
        _ => return None,
    };
    let mut tex = operator.to_owned();
    match property(node, "naryPr", "limLoc") {
        Some("undOvr") => tex.push_str("\\limits"),
        Some("subSup") => tex.push_str("\\nolimits"),
        _ => {}
    }
    if !property_on(node, "naryPr", "subHide") {
        append_script(&mut tex, argument(node, "sub")?, '_');
    }
    if !property_on(node, "naryPr", "supHide") {
        append_script(&mut tex, argument(node, "sup")?, '^');
    }
    tex.push(' ');
    tex.push_str(&argument(node, "e")?);
    Some(tex)
}

fn run(node: Node<'_, '_>) -> Option<String> {
    let text: String = node
        .children()
        .filter(|node| is_m(*node, "t"))
        .filter_map(|node| node.text())
        .collect();
    if property_on(node, "rPr", "nor") {
        return Some(format!("\\text{{{}}}", escape_text(&text)));
    }
    let mode = match (
        property(node, "rPr", "scr").unwrap_or("roman"),
        property(node, "rPr", "sty").unwrap_or("i"),
    ) {
        ("roman", "p") if text.chars().any(char::is_alphabetic) => Some("mathrm"),
        ("roman", "i" | "p") => None,
        ("roman", "b") => Some("mathbf"),
        ("roman", "bi") => Some("boldsymbol"),
        ("script", _) => Some("mathcal"),
        ("double-struck", _) => Some("mathbb"),
        ("fraktur", _) => Some("mathfrak"),
        ("sans-serif", _) => Some("mathsf"),
        ("monospace", _) => Some("mathtt"),
        _ => return None,
    };
    let mut tex = String::new();
    for ch in text.chars() {
        match ch {
            '\u{200b}' | '\u{2061}'..='\u{2064}' => {}
            '\u{2009}' => tex.push_str("\\,"),
            '\u{2001}' => tex.push_str("\\quad "),
            '∣' | '|' => tex.push_str("\\mid "),
            '⊤' => tex.push_str("\\top "),
            '◼' => tex.push_str("\\blacksquare "),
            '⏟' => tex.push_str("\\underbrace{\\phantom{x}}"),
            '…' => tex.push_str("\\ldots "),
            '⋯' => tex.push_str("\\cdots "),
            'ϵ' => tex.push_str("\\epsilon "),
            'ϕ' => tex.push_str("\\phi "),
            '≲' => tex.push_str("\\lesssim "),
            '⟨' => tex.push_str("\\langle "),
            '⟩' => tex.push_str("\\rangle "),
            '⟹' => tex.push_str("\\implies "),
            '\\' => tex.push_str("\\backslash "),
            '{' | '}' | '$' | '%' | '#' | '&' | '_' => {
                tex.push('\\');
                tex.push(ch);
            }
            '^' => tex.push_str("\\^{ }"),
            '~' => tex.push_str("\\sim "),
            '\r' | '\n' | '\t' => tex.push(' '),
            _ => tex.push(ch),
        }
    }
    let Some(mode) = mode else {
        return Some(tex);
    };
    Some(format!("\\{mode}{{{tex}}}"))
}

fn escape_text(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        match ch {
            '\\' => escaped.push_str("\\textbackslash "),
            '{' | '}' | '$' | '%' | '#' | '&' | '_' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            '\r' | '\n' | '\t' => escaped.push(' '),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn delimiter(value: &str) -> Option<&str> {
    match value {
        "" => Some("."),
        "(" | ")" | "[" | "]" | "." | "/" => Some(value),
        "{" => Some("\\{"),
        "}" => Some("\\}"),
        "|" | "∣" => Some("\\vert"),
        "‖" => Some("\\Vert"),
        "⟨" => Some("\\langle"),
        "⟩" => Some("\\rangle"),
        "⌊" => Some("\\lfloor"),
        "⌋" => Some("\\rfloor"),
        "⌈" => Some("\\lceil"),
        "⌉" => Some("\\rceil"),
        _ => None,
    }
}

fn append_script(tex: &mut String, value: String, marker: char) {
    if !value.trim().is_empty() {
        tex.push(marker);
        tex.push('{');
        tex.push_str(&value);
        tex.push('}');
    }
}

fn script_base(value: String) -> String {
    if value.chars().count() == 1 {
        value
    } else {
        format!("{{{value}}}")
    }
}

fn argument(node: Node<'_, '_>, name: &str) -> Option<String> {
    match node.children().find(|node| is_m(*node, name)) {
        Some(child) => expression(child),
        None => Some(String::new()),
    }
}

fn property_element<'a, 'input>(
    node: Node<'a, 'input>,
    group: &str,
    name: &str,
) -> Option<Node<'a, 'input>> {
    node.children()
        .find(|node| is_m(*node, group))?
        .children()
        .find(|node| is_m(*node, name))
}

fn property<'a>(node: Node<'a, '_>, group: &str, name: &str) -> Option<&'a str> {
    property_element(node, group, name)?.attribute((M_NS, "val"))
}

fn property_on(node: Node<'_, '_>, group: &str, name: &str) -> bool {
    property_element(node, group, name)
        .is_some_and(|node| !matches!(node.attribute((M_NS, "val")), Some("0" | "false" | "off")))
}
