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
