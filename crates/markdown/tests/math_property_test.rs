//! Property-based tests for math formula parsing and rendering.
//!
//! Implements the following design properties:
//!
//! - **Property 7: 数学公式 AST 创建** — the parser creates `MathBlock` /
//!   `InlineMath` AST nodes for block / inline LaTeX and preserves the original
//!   LaTeX content. **Validates: Requirements 3.1, 3.2**
//! - **Property 8: 有效数学公式渲染成功** — any syntactically valid LaTeX renders
//!   successfully into a non-empty visual representation. **Validates: Requirements 3.4**
//! - **Property 9: 无效数学公式返回错误** — any syntactically invalid LaTeX returns
//!   an error (with a non-empty message) instead of panicking. **Validates: Requirements 3.5**
//!
//! ## Note on block-math syntax
//!
//! The current parser recognises inline math from `$...$` text and block math
//! from fenced ```` ```math ```` code blocks (both map to the `MathBlock` /
//! `InlineMath` AST nodes). The `$$…$$` display-delimiter form is not yet wired
//! into block parsing (pulldown-cmark math extension is disabled), so these
//! property tests exercise the syntaxes the parser actually supports when
//! validating AST-node creation.

use markdown::ast::{Block, Inline};
use markdown::math::MathRenderer;
use markdown::parser::{Parser, ParserOptions};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Generators
// ---------------------------------------------------------------------------

/// Generate "plain" LaTeX content that survives Markdown inline parsing without
/// being reinterpreted as other Markdown constructs. Uses a safe character set
/// (no `$`, backslashes, braces, or Markdown-active characters) so the exact
/// content is preserved through the parser and can be asserted.
fn arb_plain_latex() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 +=^()-]{1,40}"
        .prop_filter("must contain a non-space char", |s| !s.trim().is_empty())
}

/// A single "atom" of guaranteed-valid LaTeX. Each atom on its own is balanced
/// (no unmatched braces), contains no `$`, and no triple backslash.
fn arb_valid_atom() -> impl Strategy<Value = String> {
    prop_oneof![
        // Plain tokens: letters, digits, common operators.
        "[a-zA-Z0-9 +=^_()-]{1,10}",
        // Balanced group.
        "[a-zA-Z0-9 +-]{1,6}".prop_map(|s| format!("{{{s}}}")),
        // A fraction with balanced groups.
        ("[a-zA-Z0-9]{1,4}", "[a-zA-Z0-9]{1,4}").prop_map(|(a, b)| format!("\\frac{{{a}}}{{{b}}}")),
        // A common command.
        Just("\\alpha".to_string()),
        Just("\\sum".to_string()),
        Just("\\int".to_string()),
        Just("\\sqrt{2}".to_string()),
    ]
}

/// Generate a syntactically valid LaTeX expression by concatenating valid atoms.
/// The result is non-empty, has balanced braces, contains no `$`, and no triple
/// backslash — i.e. it satisfies `MathRenderer::validate_syntax`.
fn arb_valid_latex() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_valid_atom(), 1..5)
        .prop_map(|atoms| atoms.join(" "))
        .prop_filter("must be non-empty after trim", |s| !s.trim().is_empty())
}

/// Generate LaTeX that is guaranteed to be *invalid* per the renderer's syntax
/// rules. Each variant injects a specific defect the validator must reject.
fn arb_invalid_latex() -> impl Strategy<Value = String> {
    prop_oneof![
        // Empty / whitespace only.
        "[ \t]{0,5}".prop_map(|s| s),
        // Contains a dollar sign (delimiters must have been stripped).
        "[a-zA-Z0-9 +=-]{0,20}".prop_map(|s| format!("{s}$x")),
        // Unmatched opening brace.
        "[a-zA-Z0-9]{1,10}".prop_map(|s| format!("\\frac{{{s}")),
        // Unmatched closing brace.
        "[a-zA-Z0-9]{1,10}".prop_map(|s| format!("{s}}}")),
        // Triple backslash escape sequence.
        "[a-zA-Z0-9]{1,10}".prop_map(|s| format!("{s}\\\\\\")),
    ]
}

fn parser() -> Parser {
    Parser::new(ParserOptions::default())
}

// ---------------------------------------------------------------------------
// Property 7: 数学公式 AST 创建
// ---------------------------------------------------------------------------

proptest! {
    /// **Property 7 (inline): 数学公式 AST 创建**
    ///
    /// **Validates: Requirements 3.2**
    ///
    /// For any inline LaTeX formula written as `$formula$`, the parser creates an
    /// `Inline::InlineMath` node preserving the original LaTeX content.
    #[test]
    fn prop_inline_math_ast_creation(latex in arb_plain_latex()) {
        let src = format!("${latex}$");
        let doc = parser().parse(&src).expect("parse should succeed");

        // Find the InlineMath node produced.
        let mut found = None;
        for block in &doc.blocks {
            if let Block::Paragraph { content, .. } = block {
                for inline in content {
                    if let Inline::InlineMath(m) = inline {
                        found = Some(m.clone());
                    }
                }
            }
        }

        let math = found.expect("expected an InlineMath node to be created");
        // The parser trims the delimiters; content (trimmed) must be preserved.
        prop_assert_eq!(math.trim(), latex.trim());
    }

    /// **Property 7 (block): 数学公式 AST 创建**
    ///
    /// **Validates: Requirements 3.1**
    ///
    /// For any block LaTeX formula written as a fenced ```` ```math ```` block,
    /// the parser creates a `Block::MathBlock` node preserving the LaTeX content.
    #[test]
    fn prop_block_math_ast_creation(latex in arb_plain_latex()) {
        let src = format!("```math\n{latex}\n```");
        let doc = parser().parse(&src).expect("parse should succeed");

        let mut found = None;
        for block in &doc.blocks {
            if let Block::MathBlock { latex, .. } = block {
                found = Some(latex.clone());
            }
        }

        let math = found.expect("expected a MathBlock node to be created");
        prop_assert_eq!(math.trim(), latex.trim());
    }
}

// ---------------------------------------------------------------------------
// Property 8: 有效数学公式渲染成功
// ---------------------------------------------------------------------------

proptest! {
    /// **Property 8: 有效数学公式渲染成功**
    ///
    /// **Validates: Requirements 3.4**
    ///
    /// For any syntactically valid LaTeX, both inline and block rendering succeed
    /// and return a non-empty visual representation with positive dimensions.
    #[test]
    fn prop_valid_math_renders_successfully(latex in arb_valid_latex()) {
        let renderer = MathRenderer::new();

        // Precondition: the generated LaTeX really is valid.
        prop_assume!(renderer.validate_syntax(&latex).is_ok());

        let inline = renderer.render_inline(&latex);
        prop_assert!(inline.is_ok(), "inline render failed for valid latex: {:?}", latex);
        let inline = inline.unwrap();
        prop_assert!(!inline.svg.is_empty(), "inline svg should be non-empty");
        prop_assert!(inline.dimensions.width > 0.0);
        prop_assert!(inline.dimensions.height > 0.0);

        let block = renderer.render_block(&latex);
        prop_assert!(block.is_ok(), "block render failed for valid latex: {:?}", latex);
        let block = block.unwrap();
        prop_assert!(!block.svg.is_empty(), "block svg should be non-empty");
        prop_assert!(block.dimensions.width > 0.0);
        prop_assert!(block.dimensions.height > 0.0);
    }
}

// ---------------------------------------------------------------------------
// Property 9: 无效数学公式返回错误
// ---------------------------------------------------------------------------

proptest! {
    /// **Property 9: 无效数学公式返回错误**
    ///
    /// **Validates: Requirements 3.5**
    ///
    /// For any syntactically invalid LaTeX, the renderer returns an error (with a
    /// non-empty message) rather than panicking.
    #[test]
    fn prop_invalid_math_returns_error(latex in arb_invalid_latex()) {
        let renderer = MathRenderer::new();

        // Precondition: the generated LaTeX really is invalid.
        prop_assume!(renderer.validate_syntax(&latex).is_err());

        let inline = renderer.render_inline(&latex);
        prop_assert!(inline.is_err(), "expected inline render error for invalid latex: {:?}", latex);
        prop_assert!(
            !inline.unwrap_err().message.is_empty(),
            "error message should be non-empty"
        );

        let block = renderer.render_block(&latex);
        prop_assert!(block.is_err(), "expected block render error for invalid latex: {:?}", latex);
        prop_assert!(
            !block.unwrap_err().message.is_empty(),
            "error message should be non-empty"
        );
    }
}
