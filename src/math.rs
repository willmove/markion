//! Math formula validation and readable-text fallback rendering.
//!
//! Real KaTeX/MathJax rendering is deferred; this module degrades LaTeX to a
//! readable plain-text approximation and validates brace/environment balance.

use crate::model::RenderedMath;

pub fn render_math(latex: &str, display: bool) -> RenderedMath {
    RenderedMath {
        latex: latex.to_string(),
        display,
        text: math_preview_text(latex),
        error: validate_latex(latex).err(),
    }
}

pub fn validate_latex(latex: &str) -> Result<(), String> {
    if latex.trim().is_empty() {
        return Err("math formula is empty".to_string());
    }

    let mut braces = Vec::new();
    for (index, ch) in latex.char_indices() {
        match ch {
            '{' => braces.push(index),
            '}' if braces.pop().is_none() => {
                return Err(format!("unmatched closing brace at byte {index}"));
            }
            _ => {}
        }
    }
    if let Some(index) = braces.pop() {
        return Err(format!("unclosed brace at byte {index}"));
    }

    let begin_count = latex.matches("\\begin{").count();
    let end_count = latex.matches("\\end{").count();
    if begin_count != end_count {
        return Err("mismatched LaTeX environment delimiters".to_string());
    }

    typune_markdown::MathRenderer::new()
        .validate_syntax(latex)
        .map_err(|error| error.to_string())
}

fn math_preview_text(latex: &str) -> String {
    let mut text = latex.trim().to_string();
    text = replace_simple_fractions(&text);

    for (command, symbol) in MATH_SYMBOLS {
        text = text.replace(command, symbol);
    }

    text.replace("\\cdot", "·")
        .replace("\\times", "×")
        .replace("\\div", "÷")
        .replace("\\pm", "±")
        .replace("\\leq", "≤")
        .replace("\\geq", "≥")
        .replace("\\neq", "≠")
        .replace("\\approx", "≈")
        .replace("\\infty", "∞")
        .replace("\\sum", "∑")
        .replace("\\int", "∫")
        .replace("\\sqrt", "√")
        .replace("\\left", "")
        .replace("\\right", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

const MATH_SYMBOLS: &[(&str, &str)] = &[
    ("\\alpha", "α"),
    ("\\beta", "β"),
    ("\\gamma", "γ"),
    ("\\delta", "δ"),
    ("\\epsilon", "ε"),
    ("\\zeta", "ζ"),
    ("\\eta", "η"),
    ("\\theta", "θ"),
    ("\\lambda", "λ"),
    ("\\mu", "μ"),
    ("\\pi", "π"),
    ("\\rho", "ρ"),
    ("\\sigma", "σ"),
    ("\\tau", "τ"),
    ("\\phi", "φ"),
    ("\\omega", "ω"),
    ("\\Gamma", "Γ"),
    ("\\Delta", "Δ"),
    ("\\Theta", "Θ"),
    ("\\Lambda", "Λ"),
    ("\\Pi", "Π"),
    ("\\Sigma", "Σ"),
    ("\\Phi", "Φ"),
    ("\\Omega", "Ω"),
];

fn replace_simple_fractions(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;

    while let Some(start) = rest.find("\\frac{") {
        output.push_str(&rest[..start]);
        let after_frac = &rest[start + "\\frac{".len()..];
        let Some((numerator, after_numerator)) = take_braced_content(after_frac) else {
            output.push_str(&rest[start..]);
            return output;
        };
        let Some(after_open_denominator) = after_numerator.strip_prefix('{') else {
            output.push_str(&rest[start..start + "\\frac".len()]);
            rest = &rest[start + "\\frac".len()..];
            continue;
        };
        let Some((denominator, after_denominator)) = take_braced_content(after_open_denominator)
        else {
            output.push_str(&rest[start..]);
            return output;
        };
        output.push_str(&format!("{numerator}⁄{denominator}"));
        rest = after_denominator;
    }

    output.push_str(rest);
    output
}

fn take_braced_content(text: &str) -> Option<(&str, &str)> {
    let mut depth = 1usize;
    for (index, ch) in text.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((&text[..index], &text[index + ch.len_utf8()..]));
                }
            }
            _ => {}
        }
    }
    None
}

// --- TeX subset → OMML -------------------------------------------------------
// Direct emitter for the built-in DOCX writer's native Word equations.
// Supported: `\frac`/`\dfrac`/`\tfrac`, `\sqrt` (optional `[n]` degree),
// `^`/`_` scripts (groups or single atoms), n-ary operators (`\sum`, `\prod`,
// `\int`, `\oint`, `\bigcup`, `\bigcap`) with limits, greek letters and common
// operator symbols, `\left`/`\right`/`\big*` delimiters, and `\text`-style
// grouping commands. Anything else returns `None` so the caller can preserve
// the authored LaTeX as the math-zone text instead of approximating it.

use crate::escape::escape_xml_text;

pub(crate) fn tex_to_omml(latex: &str) -> Option<String> {
    let mut parser = OmmlParser {
        chars: latex.chars().collect(),
        pos: 0,
    };
    let body = parser.parse_sequence(false)?;
    if parser.pos != parser.chars.len() {
        // Unmatched closing brace: not our subset.
        return None;
    }
    Some(body)
}

struct OmmlParser {
    chars: Vec<char>,
    pos: usize,
}

impl OmmlParser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn skip_spaces(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_whitespace()) {
            self.pos += 1;
        }
    }

    /// A run of atoms until end of input or (inside a group) the closing brace.
    fn parse_sequence(&mut self, in_group: bool) -> Option<String> {
        let mut out = String::new();
        loop {
            self.skip_spaces();
            match self.peek() {
                None => {
                    if in_group {
                        return None;
                    }
                    break;
                }
                Some('}') if in_group => {
                    self.pos += 1;
                    break;
                }
                // Stray closer or script marker without a base: not our subset.
                Some('}') | Some('^') | Some('_') | Some('&') => return None,
                _ => self.parse_term(&mut out)?,
            }
        }
        Some(out)
    }

    /// One atom plus any trailing super/subscripts. N-ary operators absorb
    /// their limits and operand here.
    fn parse_term(&mut self, out: &mut String) -> Option<()> {
        if let Some(symbol) = self.try_parse_nary() {
            let (sub, sup) = self.parse_scripts()?;
            let mut body = String::new();
            self.skip_spaces();
            match self.peek() {
                None | Some('}') | Some('^') | Some('_') => {}
                _ => self.parse_term(&mut body)?,
            }
            // Integrals conventionally set limits to the side; sums/products
            // above and below.
            let lim_loc = if matches!(symbol, "∫" | "∮") {
                "<m:limLoc m:val=\"subSup\"/>"
            } else {
                "<m:limLoc m:val=\"undOvr\"/>"
            };
            out.push_str(&format!(
                "<m:nary><m:naryPr><m:chr m:val=\"{symbol}\"/>{lim_loc}</m:naryPr><m:sub>{sub}</m:sub><m:sup>{sup}</m:sup><m:e>{body}</m:e></m:nary>"
            ));
            return Some(());
        }
        let base = self.parse_atom()?;
        let (sub, sup) = self.parse_scripts()?;
        let rendered = match (sub.is_empty(), sup.is_empty()) {
            (true, true) => base,
            (false, true) => format!("<m:sSub><m:e>{base}</m:e><m:sub>{sub}</m:sub></m:sSub>"),
            (true, false) => format!("<m:sSup><m:e>{base}</m:e><m:sup>{sup}</m:sup></m:sSup>"),
            (false, false) => {
                format!(
                    "<m:sSubSup><m:e>{base}</m:e><m:sub>{sub}</m:sub><m:sup>{sup}</m:sup></m:sSubSup>"
                )
            }
        };
        out.push_str(&rendered);
        Some(())
    }

    fn parse_scripts(&mut self) -> Option<(String, String)> {
        let mut sub = String::new();
        let mut sup = String::new();
        loop {
            self.skip_spaces();
            match self.peek() {
                Some('^') if sup.is_empty() => {
                    self.pos += 1;
                    sup = self.parse_script_arg()?;
                }
                Some('_') if sub.is_empty() => {
                    self.pos += 1;
                    sub = self.parse_script_arg()?;
                }
                _ => break,
            }
        }
        Some((sub, sup))
    }

    fn parse_script_arg(&mut self) -> Option<String> {
        self.skip_spaces();
        match self.peek()? {
            '{' => {
                self.pos += 1;
                self.parse_sequence(true)
            }
            _ => self.parse_atom(),
        }
    }

    /// One atom: a braced group, a command, or a single character run.
    fn parse_atom(&mut self) -> Option<String> {
        match self.next_char()? {
            '{' => self.parse_sequence(true),
            '\\' => self.parse_command(),
            '~' => Some(omml_run(" ")),
            ch => Some(omml_run(&ch.to_string())),
        }
    }

    /// Consumes a supported n-ary operator command and returns its symbol.
    fn try_parse_nary(&mut self) -> Option<&'static str> {
        if self.peek() != Some('\\') {
            return None;
        }
        let mut end = self.pos + 1;
        while matches!(self.chars.get(end), Some(ch) if ch.is_ascii_alphabetic()) {
            end += 1;
        }
        let name: String = self.chars[self.pos + 1..end].iter().collect();
        let symbol = match name.as_str() {
            "sum" => "∑",
            "prod" => "∏",
            "int" => "∫",
            "oint" => "∮",
            "bigcup" => "⋃",
            "bigcap" => "⋂",
            _ => return None,
        };
        self.pos = end;
        Some(symbol)
    }

    fn parse_command(&mut self) -> Option<String> {
        let mut name = String::new();
        while matches!(self.peek(), Some(ch) if ch.is_ascii_alphabetic()) {
            name.push(self.next_char()?);
        }
        if name.is_empty() {
            // Escaped punctuation (\{, \%, ...) or a spacing command (\, \; ).
            let ch = self.next_char()?;
            return match ch {
                '{' | '}' | '%' | '$' | '#' | '_' | '&' => Some(omml_run(&ch.to_string())),
                ',' | ';' | ' ' => Some(String::new()),
                _ => None,
            };
        }
        match name.as_str() {
            "frac" | "dfrac" | "tfrac" => {
                let num = self.parse_script_arg()?;
                let den = self.parse_script_arg()?;
                Some(format!(
                    "<m:f><m:num>{num}</m:num><m:den>{den}</m:den></m:f>"
                ))
            }
            "sqrt" => {
                self.skip_spaces();
                let mut degree = String::new();
                let mut hide_degree = true;
                if self.peek() == Some('[') {
                    self.pos += 1;
                    hide_degree = false;
                    while let Some(ch) = self.peek() {
                        self.pos += 1;
                        if ch == ']' {
                            break;
                        }
                        degree.push(ch);
                    }
                }
                let body = self.parse_script_arg()?;
                let rad_pr = if hide_degree {
                    "<m:degHide m:val=\"1\"/>"
                } else {
                    ""
                };
                let degree = if hide_degree {
                    String::new()
                } else {
                    omml_run(&degree)
                };
                Some(format!(
                    "<m:rad><m:radPr>{rad_pr}</m:radPr><m:deg>{degree}</m:deg><m:e>{body}</m:e></m:rad>"
                ))
            }
            "left" | "right" | "big" | "Big" | "bigl" | "bigr" | "Bigl" | "Bigr" => {
                // Delimiter sizing commands: emit the delimiter itself.
                self.skip_spaces();
                match self.next_char()? {
                    '.' => Some(String::new()),
                    '\\' => self.parse_command(),
                    ch => Some(omml_run(&ch.to_string())),
                }
            }
            // Grouping/text commands: the group content parses normally.
            "text" | "mathrm" | "mathbf" | "mathit" | "mathsf" | "mathtt" | "operatorname"
            | "textbf" | "textit" | "boldsymbol" => self.parse_script_arg(),
            _ => omml_symbol(&name).map(omml_run),
        }
    }
}

fn omml_run(text: &str) -> String {
    format!(
        "<m:r><m:t xml:space=\"preserve\">{}</m:t></m:r>",
        escape_xml_text(text)
    )
}

/// Greek letters and common operator symbols the emitter supports.
fn omml_symbol(name: &str) -> Option<&'static str> {
    Some(match name {
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" => "ε",
        "zeta" => "ζ",
        "eta" => "η",
        "theta" => "θ",
        "iota" => "ι",
        "kappa" => "κ",
        "lambda" => "λ",
        "mu" => "μ",
        "nu" => "ν",
        "xi" => "ξ",
        "pi" => "π",
        "rho" => "ρ",
        "sigma" => "σ",
        "tau" => "τ",
        "upsilon" => "υ",
        "phi" => "φ",
        "chi" => "χ",
        "psi" => "ψ",
        "omega" => "ω",
        "Gamma" => "Γ",
        "Delta" => "Δ",
        "Theta" => "Θ",
        "Lambda" => "Λ",
        "Xi" => "Ξ",
        "Pi" => "Π",
        "Sigma" => "Σ",
        "Phi" => "Φ",
        "Psi" => "Ψ",
        "Omega" => "Ω",
        "times" => "×",
        "div" => "÷",
        "cdot" => "⋅",
        "pm" => "±",
        "mp" => "∓",
        "leq" | "le" => "≤",
        "geq" | "ge" => "≥",
        "neq" | "ne" => "≠",
        "approx" => "≈",
        "equiv" => "≡",
        "sim" => "∼",
        "propto" => "∝",
        "infty" => "∞",
        "partial" => "∂",
        "nabla" => "∇",
        "in" => "∈",
        "notin" => "∉",
        "subset" => "⊂",
        "supset" => "⊃",
        "subseteq" => "⊆",
        "supseteq" => "⊇",
        "cup" => "∪",
        "cap" => "∩",
        "vee" => "∨",
        "wedge" => "∧",
        "oplus" => "⊕",
        "otimes" => "⊗",
        "perp" => "⊥",
        "parallel" => "∥",
        "angle" => "∠",
        "degree" => "°",
        "prime" => "′",
        "rightarrow" | "to" => "→",
        "leftarrow" => "←",
        "Rightarrow" => "⇒",
        "Leftarrow" => "⇐",
        "Leftrightarrow" => "⇔",
        "mapsto" => "↦",
        "forall" => "∀",
        "exists" => "∃",
        "emptyset" => "∅",
        "neg" | "lnot" => "¬",
        "ldots" | "dots" => "…",
        "ast" => "∗",
        "star" => "⋆",
        "circ" => "∘",
        "bullet" => "∙",
        "hbar" => "ℏ",
        "ell" => "ℓ",
        "Re" => "ℜ",
        "Im" => "ℑ",
        "aleph" => "ℵ",
        _ => return None,
    })
}
