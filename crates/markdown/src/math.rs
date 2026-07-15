//! Math rendering functionality for LaTeX formulas.
//!
//! This module provides the MathRenderer for rendering LaTeX math expressions
//! (both inline `$...$` and block `$$...$$`) into visual representations.

// Note: MarkdownError and MarkdownResult may be used in future integration
#[allow(unused_imports)]
use crate::error::{MarkdownError, MarkdownResult};

/// Dimensions of rendered math content (width and height in pixels).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    /// Creates a new Size with the given width and height.
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Creates a zero-sized dimension.
    pub fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Rendered math formula output containing SVG and dimensions.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedMath {
    /// The rendered math as SVG markup.
    pub svg: String,
    /// Dimensions of the rendered output.
    pub dimensions: Size,
}

impl RenderedMath {
    /// Creates a new RenderedMath with the given SVG and dimensions.
    pub fn new(svg: String, dimensions: Size) -> Self {
        Self { svg, dimensions }
    }
}

/// Error type for math rendering operations.
#[derive(Debug, Clone, PartialEq)]
pub struct MathError {
    /// Human-readable error message.
    pub message: String,
    /// Optional position in the LaTeX source where the error occurred.
    pub position: Option<usize>,
}

impl MathError {
    /// Creates a new MathError with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position: None,
        }
    }

    /// Creates a new MathError with a message and position.
    pub fn with_position(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position: Some(position),
        }
    }
}

impl std::fmt::Display for MathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pos) = self.position {
            write!(f, "{} (at position {})", self.message, pos)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for MathError {}

/// Math rendering engine that uses KaTeX (or compatible) to render LaTeX formulas.
///
/// # Example
///
/// ```
/// use markdown::math::{MathRenderer, RenderedMath};
///
/// let renderer = MathRenderer::new();
///
/// // Render inline math
/// let result = renderer.render_inline("E = mc^2");
/// assert!(result.is_ok());
///
/// // Render block math
/// let result = renderer.render_block(r"\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}");
/// assert!(result.is_ok());
/// ```
pub struct MathRenderer {
    // Future: This will hold the KaTeX engine instance or FFI bindings
    // For now, we'll use a placeholder implementation
    _engine: (),
}

impl MathRenderer {
    /// Creates a new MathRenderer instance.
    ///
    /// This initializes the underlying math rendering engine (KaTeX or compatible).
    pub fn new() -> Self {
        Self { _engine: () }
    }

    /// Renders an inline LaTeX formula (typically used with `$...$`).
    ///
    /// Returns a `RenderedMath` containing SVG markup and dimensions,
    /// or a `MathError` if the LaTeX syntax is invalid.
    ///
    /// # Arguments
    ///
    /// * `latex` - The LaTeX formula source (without delimiters)
    ///
    /// # Example
    ///
    /// ```
    /// use markdown::math::MathRenderer;
    ///
    /// let renderer = MathRenderer::new();
    /// let result = renderer.render_inline("x^2 + y^2 = r^2");
    /// assert!(result.is_ok());
    /// ```
    pub fn render_inline(&self, latex: &str) -> Result<RenderedMath, MathError> {
        // Validate syntax first
        self.validate_syntax(latex)?;

        // TODO: Integrate actual KaTeX rendering
        // For now, return a placeholder SVG
        let svg = self.generate_placeholder_svg(latex, false);
        let dimensions = self.estimate_dimensions(latex, false);

        Ok(RenderedMath::new(svg, dimensions))
    }

    /// Renders a block (display-mode) LaTeX formula (typically used with `$$...$$`).
    ///
    /// Block formulas are typically centered and rendered larger than inline formulas.
    ///
    /// # Arguments
    ///
    /// * `latex` - The LaTeX formula source (without delimiters)
    ///
    /// # Example
    ///
    /// ```
    /// use markdown::math::MathRenderer;
    ///
    /// let renderer = MathRenderer::new();
    /// let result = renderer.render_block(r"\sum_{i=1}^n i = \frac{n(n+1)}{2}");
    /// assert!(result.is_ok());
    /// ```
    pub fn render_block(&self, latex: &str) -> Result<RenderedMath, MathError> {
        // Validate syntax first
        self.validate_syntax(latex)?;

        // TODO: Integrate actual KaTeX rendering
        // For now, return a placeholder SVG
        let svg = self.generate_placeholder_svg(latex, true);
        let dimensions = self.estimate_dimensions(latex, true);

        Ok(RenderedMath::new(svg, dimensions))
    }

    /// Validates LaTeX syntax without rendering.
    ///
    /// Returns `Ok(())` if the syntax is valid, or a detailed `MathError` if invalid.
    ///
    /// # Arguments
    ///
    /// * `latex` - The LaTeX formula source to validate
    ///
    /// # Example
    ///
    /// ```
    /// use markdown::math::MathRenderer;
    ///
    /// let renderer = MathRenderer::new();
    /// assert!(renderer.validate_syntax("x + y").is_ok());
    /// assert!(renderer.validate_syntax("{unclosed").is_err());
    /// ```
    pub fn validate_syntax(&self, latex: &str) -> Result<(), MathError> {
        // Basic syntax validation
        if latex.trim().is_empty() {
            return Err(MathError::new("Empty LaTeX expression"));
        }

        // Check for balanced braces
        let mut brace_depth = 0;
        let mut brace_pos = None;
        for (i, ch) in latex.chars().enumerate() {
            match ch {
                '{' => {
                    if brace_pos.is_none() {
                        brace_pos = Some(i);
                    }
                    brace_depth += 1;
                }
                '}' => {
                    brace_depth -= 1;
                    if brace_depth < 0 {
                        return Err(MathError::with_position("Unmatched closing brace '}'", i));
                    }
                }
                _ => {}
            }
        }

        if brace_depth > 0 {
            return Err(MathError::with_position(
                "Unmatched opening brace '{'",
                brace_pos.unwrap_or(0),
            ));
        }

        // Check for common LaTeX errors
        if latex.contains("\\\\\\") {
            return Err(MathError::new("Invalid escape sequence: triple backslash"));
        }

        // Check for unmatched $ signs (should be pre-processed)
        if latex.contains('$') {
            return Err(MathError::new(
                "LaTeX source should not contain dollar signs (delimiters should be removed)",
            ));
        }

        // TODO: Add more sophisticated validation when KaTeX is integrated
        // For now, accept as valid if basic checks pass
        Ok(())
    }

    // Helper: Generate placeholder SVG for testing
    // This will be replaced by actual KaTeX rendering
    fn generate_placeholder_svg(&self, latex: &str, is_block: bool) -> String {
        let display_type = if is_block { "block" } else { "inline" };
        let escaped_latex = latex
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;");

        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"200\" height=\"50\">\
             <rect width=\"100%\" height=\"100%\" fill=\"#f0f0f0\" />\
             <text x=\"10\" y=\"25\" font-family=\"monospace\" font-size=\"12\" fill=\"black\">\
             [{}] {}\
             </text>\
             </svg>",
            display_type, escaped_latex
        )
    }

    // Helper: Estimate dimensions based on LaTeX content
    // This will be replaced by actual measurements from KaTeX
    fn estimate_dimensions(&self, latex: &str, is_block: bool) -> Size {
        let base_width = latex.len() as f32 * 8.0 + 20.0; // Rough estimate: 8px per char
        let base_height = if is_block { 50.0 } else { 20.0 };

        // Add extra height for complex expressions
        let has_fraction = latex.contains("\\frac");
        let has_sum = latex.contains("\\sum");
        let has_int = latex.contains("\\int");
        let extra_height = if has_fraction || has_sum || has_int {
            30.0
        } else {
            0.0
        };

        Size::new(base_width.min(800.0), base_height + extra_height)
    }
}

impl Default for MathRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_creation() {
        let _renderer = MathRenderer::new();
    }

    #[test]
    fn render_inline_simple() {
        let renderer = MathRenderer::new();
        let result = renderer.render_inline("x + y");
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(!rendered.svg.is_empty());
        assert!(rendered.dimensions.width > 0.0);
        assert!(rendered.dimensions.height > 0.0);
    }

    #[test]
    fn render_block_simple() {
        let renderer = MathRenderer::new();
        let result = renderer.render_block(r"\sum_{i=1}^n i");
        assert!(result.is_ok());

        let rendered = result.unwrap();
        assert!(!rendered.svg.is_empty());
        assert!(rendered.dimensions.width > 0.0);
        assert!(rendered.dimensions.height > 0.0);
    }

    #[test]
    fn validate_syntax_valid() {
        let renderer = MathRenderer::new();

        assert!(renderer.validate_syntax("x + y").is_ok());
        assert!(renderer.validate_syntax("a^2 + b^2 = c^2").is_ok());
        assert!(renderer.validate_syntax(r"\frac{a}{b}").is_ok());
        assert!(renderer.validate_syntax(r"\int_0^1 x dx").is_ok());
    }

    #[test]
    fn validate_syntax_empty() {
        let renderer = MathRenderer::new();
        let result = renderer.validate_syntax("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Empty LaTeX expression");
    }

    #[test]
    fn validate_syntax_unmatched_braces() {
        let renderer = MathRenderer::new();

        // Unmatched opening brace
        let result = renderer.validate_syntax(r"\frac{a{b}");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("opening brace"));

        // Unmatched closing brace
        let result = renderer.validate_syntax(r"\frac{a}}");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("closing brace"));
    }

    #[test]
    fn validate_syntax_dollar_signs() {
        let renderer = MathRenderer::new();
        let result = renderer.validate_syntax("$x + y$");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("dollar signs"));
    }

    #[test]
    fn render_invalid_latex() {
        let renderer = MathRenderer::new();

        // Empty expression
        let result = renderer.render_inline("");
        assert!(result.is_err());

        // Unmatched braces
        let result = renderer.render_block(r"\frac{a{b}");
        assert!(result.is_err());
    }

    #[test]
    fn size_creation() {
        let size = Size::new(100.0, 50.0);
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 50.0);

        let zero = Size::zero();
        assert_eq!(zero.width, 0.0);
        assert_eq!(zero.height, 0.0);
    }

    #[test]
    fn rendered_math_creation() {
        let svg = "<svg>test</svg>".to_string();
        let dims = Size::new(200.0, 50.0);
        let rendered = RenderedMath::new(svg.clone(), dims);

        assert_eq!(rendered.svg, svg);
        assert_eq!(rendered.dimensions, dims);
    }

    #[test]
    fn math_error_display() {
        let err = MathError::new("Test error");
        assert_eq!(err.to_string(), "Test error");

        let err = MathError::with_position("Syntax error", 42);
        assert_eq!(err.to_string(), "Syntax error (at position 42)");
    }

    #[test]
    fn dimensions_estimation_inline() {
        let renderer = MathRenderer::new();
        let dims = renderer.estimate_dimensions("x + y", false);
        assert!(dims.width > 0.0);
        assert!(dims.height > 0.0);
        assert!(dims.height < 50.0); // Inline should be smaller
    }

    #[test]
    fn dimensions_estimation_block() {
        let renderer = MathRenderer::new();
        let dims = renderer.estimate_dimensions(r"\sum_{i=1}^n i", true);
        assert!(dims.width > 0.0);
        assert!(dims.height >= 50.0); // Block should be larger
    }

    #[test]
    fn dimensions_complex_expression() {
        let renderer = MathRenderer::new();

        // Expression with fraction gets extra height
        let dims_frac = renderer.estimate_dimensions(r"\frac{a}{b}", false);
        let dims_simple = renderer.estimate_dimensions("a + b", false);
        assert!(dims_frac.height > dims_simple.height);
    }
}
