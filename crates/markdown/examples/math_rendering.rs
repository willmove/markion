//! Example demonstrating the MathRenderer functionality.
//!
//! This example shows how to use the MathRenderer to render inline and block
//! LaTeX math formulas.

use markdown::math::{MathRenderer, RenderedMath};

fn main() {
    let renderer = MathRenderer::new();

    println!("=== Math Rendering Example ===\n");

    // Example 1: Simple inline math
    println!("1. Simple inline formula:");
    let formula = "E = mc^2";
    match renderer.render_inline(formula) {
        Ok(result) => print_result(formula, &result),
        Err(e) => println!("   Error: {}", e),
    }

    // Example 2: Inline math with fractions
    println!("\n2. Inline formula with fraction:");
    let formula = r"\frac{a}{b}";
    match renderer.render_inline(formula) {
        Ok(result) => print_result(formula, &result),
        Err(e) => println!("   Error: {}", e),
    }

    // Example 3: Block math with summation
    println!("\n3. Block formula with summation:");
    let formula = r"\sum_{i=1}^n i = \frac{n(n+1)}{2}";
    match renderer.render_block(formula) {
        Ok(result) => print_result(formula, &result),
        Err(e) => println!("   Error: {}", e),
    }

    // Example 4: Block math with integral
    println!("\n4. Block formula with integral:");
    let formula = r"\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}";
    match renderer.render_block(formula) {
        Ok(result) => print_result(formula, &result),
        Err(e) => println!("   Error: {}", e),
    }

    // Example 5: Invalid LaTeX (unmatched braces)
    println!("\n5. Invalid LaTeX (unmatched braces):");
    let formula = r"\frac{a{b}";
    match renderer.render_inline(formula) {
        Ok(result) => print_result(formula, &result),
        Err(e) => println!("   LaTeX: {}\n   Error: {}", formula, e),
    }

    // Example 6: Empty formula
    println!("\n6. Empty formula:");
    let formula = "";
    match renderer.render_inline(formula) {
        Ok(result) => print_result(formula, &result),
        Err(e) => println!("   Error: {}", e),
    }

    // Example 7: Validate syntax without rendering
    println!("\n7. Syntax validation:");
    let formulas = vec![
        ("x + y", true),
        (r"a^2 + b^2 = c^2", true),
        (r"\frac{a}{b}", true),
        (r"\int_0^1 x dx", true),
        (r"\frac{a{b}", false),
        (r"$x + y$", false),
        ("", false),
    ];

    for (formula, expected_valid) in formulas {
        let is_valid = renderer.validate_syntax(formula).is_ok();
        let status = if is_valid { "✓" } else { "✗" };
        let match_expected = is_valid == expected_valid;
        println!(
            "   {} {} - {}",
            status,
            if match_expected { "✓" } else { "!" },
            formula
        );
        if !match_expected {
            println!("      WARNING: Validation result doesn't match expected!");
        }
    }

    println!("\n=== End of Example ===");
}

fn print_result(formula: &str, result: &RenderedMath) {
    println!("   LaTeX: {}", formula);
    println!(
        "   Dimensions: {:.1} x {:.1} px",
        result.dimensions.width, result.dimensions.height
    );
    println!("   SVG Preview (first 100 chars):");
    let svg_preview = if result.svg.len() > 100 {
        format!("{}...", &result.svg[..100])
    } else {
        result.svg.clone()
    };
    println!("   {}", svg_preview);
}
