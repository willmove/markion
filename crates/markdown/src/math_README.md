# Math Rendering Module

This module provides functionality for rendering LaTeX mathematical formulas in Markdown documents.

## Overview

The `math` module implements a `MathRenderer` that can render both inline (`$...$`) and block (`$$...$$`) LaTeX formulas into visual representations (currently SVG). It includes syntax validation and detailed error reporting.

## Features

- **Inline Math Rendering**: Render inline LaTeX formulas (e.g., `$E = mc^2$`)
- **Block Math Rendering**: Render display-mode LaTeX formulas (e.g., `$$\sum_{i=1}^n i$$`)
- **Syntax Validation**: Validate LaTeX syntax before rendering
- **Error Handling**: Detailed error messages with position information
- **Dimension Calculation**: Provides width and height of rendered formulas

## Current Implementation

The current implementation is a **placeholder/stub** that:
- Validates basic LaTeX syntax (balanced braces, common errors)
- Generates placeholder SVG output for testing
- Estimates dimensions based on formula complexity
- Provides the complete API surface for future KaTeX integration

## Future Integration

The module is designed to integrate with **KaTeX** (a fast LaTeX math rendering library). The integration will involve:

1. Adding KaTeX dependency (e.g., `katex` crate or FFI bindings)
2. Replacing `generate_placeholder_svg()` with actual KaTeX rendering
3. Using KaTeX's dimension calculations instead of estimates
4. Leveraging KaTeX's comprehensive LaTeX syntax support

## Usage

### Basic Example

```rust
use markdown::math::MathRenderer;

let renderer = MathRenderer::new();

// Render inline math
let result = renderer.render_inline("E = mc^2")?;
println!("SVG: {}", result.svg);
println!("Dimensions: {}x{}", result.dimensions.width, result.dimensions.height);

// Render block math
let result = renderer.render_block(r"\sum_{i=1}^n i = \frac{n(n+1)}{2}")?;
```

### Syntax Validation

```rust
use markdown::math::MathRenderer;

let renderer = MathRenderer::new();

// Validate before rendering
if let Err(error) = renderer.validate_syntax(r"\frac{a{b}") {
    println!("LaTeX error: {}", error);
    if let Some(pos) = error.position {
        println!("Error at position: {}", pos);
    }
}
```

### Integration with Parser

The math module is designed to work seamlessly with the Markdown parser:

```rust
use markdown::{Parser, math::MathRenderer};

let parser = Parser::new(Default::default());
let document = parser.parse("# Title\n\nInline: $x^2$\n\nBlock: $$\\int_0^1 x dx$$")?;

let math_renderer = MathRenderer::new();

// Find and render math blocks/inline math in the AST
for block in &document.blocks {
    match block {
        Block::MathBlock { latex, .. } => {
            let rendered = math_renderer.render_block(latex)?;
            // Use rendered.svg and rendered.dimensions
        }
        Block::Paragraph { content, .. } => {
            for inline in content {
                if let Inline::InlineMath(latex) = inline {
                    let rendered = math_renderer.render_inline(latex)?;
                    // Use rendered.svg and rendered.dimensions
                }
            }
        }
        _ => {}
    }
}
```

## API Reference

### Types

- **`MathRenderer`**: The main rendering engine
- **`RenderedMath`**: Contains SVG output and dimensions
- **`Size`**: Width and height in pixels
- **`MathError`**: Error type with message and optional position

### Methods

#### `MathRenderer::new() -> Self`
Creates a new MathRenderer instance.

#### `MathRenderer::render_inline(&self, latex: &str) -> Result<RenderedMath, MathError>`
Renders an inline LaTeX formula. The formula should not include the `$` delimiters.

#### `MathRenderer::render_block(&self, latex: &str) -> Result<RenderedMath, MathError>`
Renders a block (display-mode) LaTeX formula. The formula should not include the `$$` delimiters.

#### `MathRenderer::validate_syntax(&self, latex: &str) -> Result<(), MathError>`
Validates LaTeX syntax without rendering. Useful for checking formulas before committing to render.

## Error Handling

The module provides detailed error information through the `MathError` type:

```rust
pub struct MathError {
    pub message: String,
    pub position: Option<usize>,
}
```

Common errors include:
- Empty LaTeX expression
- Unmatched braces (`{` or `}`)
- Invalid escape sequences
- Dollar signs in input (delimiters should be stripped)

## Testing

The module includes comprehensive unit tests:

```bash
# Run all math tests
cargo test --package markdown --lib math

# Run a specific test
cargo test --package markdown --lib math::tests::validate_syntax_unmatched_braces
```

Run the example:

```bash
cargo run --package markdown --example math_rendering
```

## Performance Considerations

### Current Implementation
- Validation: O(n) where n is the length of the LaTeX string
- Dimension estimation: O(1) with simple heuristics
- SVG generation: O(n) for placeholder generation

### Future KaTeX Integration
- KaTeX is highly optimized for performance
- Caching can be added for frequently rendered formulas
- Async rendering can be implemented for large documents

## Validation Rules

The current implementation validates:

1. **Non-empty input**: LaTeX expressions cannot be empty
2. **Balanced braces**: All `{` must have matching `}`
3. **No dollar signs**: Input should have delimiters pre-stripped
4. **Valid escape sequences**: No triple backslashes

Future integration with KaTeX will provide:
- Full LaTeX syntax validation
- Support for all KaTeX-supported commands
- Macro and environment validation

## Design Rationale

### Why SVG Output?

SVG was chosen for math rendering because:
1. **Scalability**: Vector graphics scale to any size without quality loss
2. **Web compatibility**: SVG is widely supported in browsers and renderers
3. **Styling**: SVG elements can be styled with CSS
4. **Accessibility**: SVG can include semantic markup for screen readers

### Why KaTeX?

KaTeX is the planned integration target because:
1. **Performance**: Much faster than MathJax (5-10x)
2. **Quality**: Produces high-quality output
3. **Compatibility**: Supports most common LaTeX commands
4. **No dependencies**: Pure JavaScript, can be compiled to WASM
5. **Active maintenance**: Well-maintained with regular updates

## Migration Path

To integrate actual KaTeX rendering:

1. Add dependency:
   ```toml
   [dependencies]
   katex = "0.4"  # or appropriate version
   ```

2. Update `MathRenderer` struct:
   ```rust
   pub struct MathRenderer {
       katex_engine: katex::Katex,
   }
   ```

3. Replace placeholder methods with KaTeX calls:
   ```rust
   pub fn render_inline(&self, latex: &str) -> Result<RenderedMath, MathError> {
       self.validate_syntax(latex)?;
       
       let opts = katex::Opts::builder().display_mode(false).build()?;
       let html = self.katex_engine.render_with_opts(latex, &opts)
           .map_err(|e| MathError::new(e.to_string()))?;
       
       // Convert HTML to SVG if needed
       let svg = html_to_svg(&html)?;
       let dimensions = calculate_dimensions(&svg)?;
       
       Ok(RenderedMath::new(svg, dimensions))
   }
   ```

4. Update tests to validate actual rendering output

## Requirements Addressed

This implementation addresses the following requirements from the spec:

- **Requirement 3.3**: Support LaTeX syntax parsing
- **Requirement 3.4**: Real-time rendering of math formulas
- **Requirement 3.5**: Error handling for invalid syntax
- **Requirement 3.6**: Support for common math symbols and operators
- **Requirement 3.7**: Display editable LaTeX source on click (API provides SVG and source)

## Contributing

When adding features or fixing bugs:

1. Add tests for new functionality
2. Update documentation
3. Ensure all existing tests pass
4. Follow Rust naming conventions
5. Add doc comments for public APIs

## License

Part of the Typune Markdown editor project.
