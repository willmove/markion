//! Offline, GPUI-free LaTeX math typesetting.
//!
//! RaTeX parses and lays out the formula, while `ratex-svg` emits outlined,
//! self-contained SVG. The app crate owns SVG rasterization and GPUI images.

use std::panic::{AssertUnwindSafe, catch_unwind};

use ratex_layout::{LayoutOptions, layout, to_display_list};
use ratex_parser::parse;
use ratex_svg::{SvgOptions, render_to_svg};
use ratex_types::{color::Color, math_style::MathStyle as RatexMathStyle};

pub const RENDERER_VERSION: &str = "ratex-0.1.13-embedded-katex";
pub const MAX_FORMULA_BYTES: usize = 16 * 1024;
pub const MAX_SVG_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_RENDER_DIMENSION: f32 = 16_384.0;
const MIN_FONT_SIZE: f32 = 6.0;
const MAX_FONT_SIZE: f32 = 256.0;
const DEFAULT_INLINE_FONT_SIZE: f32 = 16.0;
const DEFAULT_DISPLAY_FONT_SIZE: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MathStyle {
    Inline,
    Display,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MathRenderOptions {
    pub style: MathStyle,
    pub font_size: f32,
    pub foreground: [u8; 4],
    pub padding: f32,
}

impl MathRenderOptions {
    pub fn inline(font_size: f32, foreground: [u8; 4]) -> Self {
        Self {
            style: MathStyle::Inline,
            font_size,
            foreground,
            padding: 1.0,
        }
    }

    pub fn display(font_size: f32, foreground: [u8; 4]) -> Self {
        Self {
            style: MathStyle::Display,
            font_size,
            foreground,
            padding: 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub const fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderedMath {
    pub svg: String,
    pub dimensions: Size,
    /// Logical distance from the top edge to the formula baseline.
    pub ascent: f32,
    /// Logical distance from the formula baseline to the bottom edge.
    pub descent: f32,
    pub style: MathStyle,
}

impl RenderedMath {
    pub fn baseline(&self) -> f32 {
        self.ascent
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathErrorKind {
    Empty,
    Delimiter,
    InputTooLarge,
    InvalidOptions,
    Parse,
    Layout,
    OutputTooLarge,
    UnsafeSvg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathError {
    pub kind: MathErrorKind,
    pub message: String,
    pub position: Option<usize>,
}

impl MathError {
    pub fn new(kind: MathErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            position: None,
        }
    }

    pub fn with_position(kind: MathErrorKind, message: impl Into<String>, position: usize) -> Self {
        Self {
            kind,
            message: message.into(),
            position: Some(position),
        }
    }
}

impl std::fmt::Display for MathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.position {
            Some(position) => write!(f, "{} (at byte {})", self.message, position),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for MathError {}

#[derive(Debug, Clone, Copy, Default)]
pub struct MathRenderer;

impl MathRenderer {
    pub const fn new() -> Self {
        Self
    }

    pub fn render_inline(&self, latex: &str) -> Result<RenderedMath, MathError> {
        self.render(
            latex,
            MathRenderOptions::inline(DEFAULT_INLINE_FONT_SIZE, [0, 0, 0, 255]),
        )
    }

    pub fn render_block(&self, latex: &str) -> Result<RenderedMath, MathError> {
        self.render(
            latex,
            MathRenderOptions::display(DEFAULT_DISPLAY_FONT_SIZE, [0, 0, 0, 255]),
        )
    }

    pub fn validate_syntax(&self, latex: &str) -> Result<(), MathError> {
        validate_input(latex)?;
        parse(latex).map(|_| ()).map_err(parse_error)
    }

    pub fn render(
        &self,
        latex: &str,
        options: MathRenderOptions,
    ) -> Result<RenderedMath, MathError> {
        validate_input(latex)?;
        validate_options(options)?;

        let ast = parse(latex).map_err(parse_error)?;
        let ratex_style = match options.style {
            MathStyle::Inline => RatexMathStyle::Text,
            MathStyle::Display => RatexMathStyle::Display,
        };
        let [r, g, b, a] = options.foreground;
        let color = Color::new(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            f32::from(a) / 255.0,
        );
        let layout_options = LayoutOptions::default()
            .with_style(ratex_style)
            .with_color(color);

        let result = catch_unwind(AssertUnwindSafe(|| {
            let layout_box = layout(&ast, &layout_options);
            let display_list = to_display_list(&layout_box);
            let svg_options = SvgOptions {
                font_size: f64::from(options.font_size),
                padding: f64::from(options.padding),
                stroke_width: (f64::from(options.font_size) / 26.0).max(0.5),
                embed_glyphs: true,
                font_dir: String::new(),
            };
            let svg = render_to_svg(&display_list, &svg_options);
            (display_list, svg)
        }))
        .map_err(|_| {
            MathError::new(
                MathErrorKind::Layout,
                "math layout failed without modifying the document",
            )
        })?;

        let (display_list, svg) = result;
        let padding = options.padding;
        let ascent = display_list.height as f32 * options.font_size + padding;
        let descent = display_list.depth as f32 * options.font_size + padding;
        let width = display_list.width as f32 * options.font_size + padding * 2.0;
        let height = ascent + descent;

        if !width.is_finite()
            || !height.is_finite()
            || width <= 0.0
            || height <= 0.0
            || width > MAX_RENDER_DIMENSION
            || height > MAX_RENDER_DIMENSION
            || svg.len() > MAX_SVG_BYTES
        {
            return Err(MathError::new(
                MathErrorKind::OutputTooLarge,
                "rendered formula exceeds the configured output bounds",
            ));
        }
        if !is_self_contained_svg(&svg) {
            return Err(MathError::new(
                MathErrorKind::UnsafeSvg,
                "renderer could not produce inert self-contained SVG glyphs",
            ));
        }

        Ok(RenderedMath {
            svg,
            dimensions: Size::new(width, height),
            ascent,
            descent,
            style: options.style,
        })
    }
}

fn validate_input(latex: &str) -> Result<(), MathError> {
    if latex.trim().is_empty() {
        return Err(MathError::new(
            MathErrorKind::Empty,
            "math expression is empty",
        ));
    }
    if latex.len() > MAX_FORMULA_BYTES {
        return Err(MathError::new(
            MathErrorKind::InputTooLarge,
            "math expression exceeds the configured input bound",
        ));
    }
    if let Some(position) = latex.find('$') {
        return Err(MathError::with_position(
            MathErrorKind::Delimiter,
            "math payload must not contain dollar delimiters",
            position,
        ));
    }
    Ok(())
}

fn validate_options(options: MathRenderOptions) -> Result<(), MathError> {
    if !options.font_size.is_finite()
        || !(MIN_FONT_SIZE..=MAX_FONT_SIZE).contains(&options.font_size)
        || !options.padding.is_finite()
        || options.padding < 0.0
        || options.padding > 64.0
    {
        return Err(MathError::new(
            MathErrorKind::InvalidOptions,
            "math render size or padding is outside the configured bounds",
        ));
    }
    Ok(())
}

fn parse_error(error: ratex_parser::ParseError) -> MathError {
    match error.loc {
        Some(location) => {
            MathError::with_position(MathErrorKind::Parse, error.message, location.start)
        }
        None => MathError::new(MathErrorKind::Parse, error.message),
    }
}

/// Generated SVG is accepted only when every glyph is outlined/embedded and
/// the document contains no active or external content.
pub fn is_self_contained_svg(svg: &str) -> bool {
    let lower = svg.to_ascii_lowercase();
    if !lower.starts_with("<svg")
        || lower.contains("<text")
        || lower.contains("<script")
        || lower.contains("<foreignobject")
        || lower.contains("javascript:")
        || lower.contains("file:")
        || lower.contains("https://")
        || lower.contains("url(")
        || lower.contains(" onclick")
        || lower.contains(" onload")
        || lower.contains(" onerror")
        || lower.contains(" onmouse")
        || lower.contains(" onfocus")
        || lower.contains(" onbegin")
    {
        return false;
    }

    let without_namespace = lower.replace("http://www.w3.org/2000/svg", "");
    if without_namespace.contains("http://") {
        return false;
    }

    let mut rest = lower.as_str();
    while let Some(index) = rest.find("href=\"") {
        rest = &rest[index + "href=\"".len()..];
        if !rest.starts_with("data:image/png;base64,") {
            return false;
        }
        let Some(end) = rest.find('"') else {
            return false;
        };
        rest = &rest[end + 1..];
    }
    true
}

/// Serialize one formula as inert, self-contained HTML. The delimiter-free
/// payload is retained byte-for-byte in `data-latex`, while `authored` is used
/// for the accessible label and exact error fallback.
pub fn serialize_math_html(latex: &str, authored: &str, style: MathStyle) -> String {
    let (tag, class, style_name, options) = match style {
        MathStyle::Inline => (
            "span",
            "math math-inline markion-math",
            "text",
            MathRenderOptions::inline(DEFAULT_INLINE_FONT_SIZE, [32, 33, 36, 255]),
        ),
        MathStyle::Display => (
            "div",
            "math math-display markion-math",
            "display",
            MathRenderOptions::display(DEFAULT_DISPLAY_FONT_SIZE, [32, 33, 36, 255]),
        ),
    };
    let escaped_latex = escape_html_attribute(latex);
    let escaped_authored_attribute = escape_html_attribute(authored);
    match MathRenderer::new().render(latex, options) {
        Ok(rendered) => {
            debug_assert!(is_self_contained_svg(&rendered.svg));
            let svg =
                rendered
                    .svg
                    .replacen("<svg", "<svg aria-hidden=\"true\" focusable=\"false\"", 1);
            let baseline_style = match style {
                MathStyle::Inline => format!(
                    " style=\"display:inline-block;vertical-align:-{:.3}px\"",
                    rendered.descent
                ),
                MathStyle::Display => {
                    " style=\"display:block;overflow-x:auto;text-align:center\"".to_string()
                }
            };
            format!(
                "<{tag} class=\"{class}\" data-latex=\"{escaped_latex}\" data-style=\"{style_name}\" data-valid=\"true\" role=\"img\" aria-label=\"{escaped_authored_attribute}\"{baseline_style}>{svg}</{tag}>"
            )
        }
        Err(error) => format!(
            "<{tag} class=\"{class} math-error\" data-latex=\"{escaped_latex}\" data-style=\"{style_name}\" data-valid=\"false\" role=\"img\" aria-label=\"{escaped_authored_attribute}\" title=\"{}\"><span class=\"math-source-fallback\">{}</span></{tag}>",
            escape_html_attribute(&error.to_string()),
            escape_html_text(authored),
        ),
    }
}

fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_html_attribute(value: &str) -> String {
    escape_html_text(value)
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORPUS: &str = include_str!("../tests/fixtures/math-render-corpus.txt");

    fn corpus_cases() -> impl Iterator<Item = (&'static str, &'static str, &'static str)> {
        CORPUS.lines().filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut parts = line.splitn(3, '|');
            Some((parts.next()?, parts.next()?, parts.next()?))
        })
    }

    #[test]
    fn representative_corpus_matches_expected_outcomes() {
        let renderer = MathRenderer::new();
        for (style, outcome, latex) in corpus_cases() {
            let result = match style {
                "inline" => renderer.render_inline(latex),
                "display" => renderer.render_block(latex),
                other => panic!("unknown corpus style {other}"),
            };
            assert_eq!(
                result.is_ok(),
                outcome == "valid",
                "unexpected result for {latex:?}: {result:?}"
            );
        }
    }

    #[test]
    fn output_is_inert_self_contained_and_deterministically_colored() {
        let renderer = MathRenderer::new();
        let options = MathRenderOptions::inline(18.0, [0x12, 0x34, 0x56, 0xff]);
        let first = renderer.render(r"\frac{a}{b}+\sqrt{x}", options).unwrap();
        let second = renderer.render(r"\frac{a}{b}+\sqrt{x}", options).unwrap();
        assert_eq!(first.svg, second.svg);
        assert!(is_self_contained_svg(&first.svg));
        assert!(first.svg.contains("rgba(18,52,86,1)"));
        assert!(!first.svg.contains("<text"));
    }

    #[test]
    fn text_and_display_styles_have_measured_baselines() {
        let renderer = MathRenderer::new();
        let latex = r"\sum_{i=1}^{n} i";
        let inline = renderer
            .render(latex, MathRenderOptions::inline(20.0, [0, 0, 0, 255]))
            .unwrap();
        let display = renderer
            .render(latex, MathRenderOptions::display(20.0, [0, 0, 0, 255]))
            .unwrap();
        assert_eq!(inline.style, MathStyle::Inline);
        assert_eq!(display.style, MathStyle::Display);
        assert!(inline.ascent > 0.0 && inline.descent >= 0.0);
        assert!(display.ascent > 0.0 && display.descent >= 0.0);
        assert_ne!(inline.svg, display.svg);
    }

    #[test]
    fn parser_error_carries_a_source_position() {
        let error = MathRenderer::new()
            .render_inline(r"\frac{a}{b")
            .unwrap_err();
        assert_eq!(error.kind, MathErrorKind::Parse);
        assert!(error.position.is_some());
    }

    #[test]
    fn input_and_options_are_bounded() {
        let renderer = MathRenderer::new();
        let oversized = "x".repeat(MAX_FORMULA_BYTES + 1);
        assert_eq!(
            renderer.render_inline(&oversized).unwrap_err().kind,
            MathErrorKind::InputTooLarge
        );
        assert_eq!(
            renderer
                .render(
                    "x",
                    MathRenderOptions::inline(MAX_FONT_SIZE + 1.0, [0, 0, 0, 255]),
                )
                .unwrap_err()
                .kind,
            MathErrorKind::InvalidOptions
        );
    }

    #[test]
    fn unsafe_or_external_svg_is_rejected() {
        assert!(!is_self_contained_svg(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><script/></svg>"#
        ));
        assert!(!is_self_contained_svg(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.test/x"/></svg>"#
        ));
        assert!(!is_self_contained_svg(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><text>x</text></svg>"#
        ));
    }

    #[test]
    fn html_serializer_emits_accessible_static_svg_and_metadata() {
        let html = serialize_math_html(r"a^2+b^2=c^2", r"$a^2+b^2=c^2$", MathStyle::Inline);
        assert!(html.starts_with("<span class=\"math math-inline markion-math\""));
        assert!(html.contains("data-latex=\"a^2+b^2=c^2\""));
        assert!(html.contains("data-style=\"text\" data-valid=\"true\""));
        assert!(html.contains("aria-label=\"$a^2+b^2=c^2$\""));
        assert!(html.contains("<svg aria-hidden=\"true\""));
        assert!(!html.to_ascii_lowercase().contains("<script"));
        assert!(!html.contains("https://"));
        assert!(!html.contains("<text"));
    }

    #[test]
    fn html_serializer_escapes_exact_invalid_source_without_stale_svg() {
        let authored = "$\\frac{<img src=x onerror=alert(1)>}{b$";
        let html = serialize_math_html(
            r"\frac{<img src=x onerror=alert(1)>}{b",
            authored,
            MathStyle::Inline,
        );
        assert!(html.contains("data-valid=\"false\""));
        assert!(html.contains("math-source-fallback"));
        assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
        assert!(!html.contains("<img"));
        assert!(!html.contains("<svg"));
    }
}
