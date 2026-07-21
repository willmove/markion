## Why

Markion currently displays math as a readable Unicode approximation or raw LaTeX, so common notation such as fractions, matrices, aligned equations, stretchy delimiters, and text/display-style operators is not rendered faithfully. High-quality native math rendering is needed for preview and reading workflows without requiring a browser, network access, or an external TeX installation.

## What Changes

- Render valid inline and display math as high-quality, transparent, theme-aware formula images in Split Preview, Read, and Visual Edit.
- Give inline math semantic layout behavior, including text-style typesetting, baseline alignment, wrapping as an atomic inline item, selection/source mapping, and source-preserving copy.
- Keep math source-backed in Visual Edit: show rendered formulas while unfocused and reveal the authored LaTeX for focused editing; never edit a rendered tree ambiguously.
- Support the existing `$...$` and `$$...$$` forms plus fenced `math` blocks, using a KaTeX-compatible core and common AMS-style constructs rather than arbitrary TeX packages.
- Render formulas asynchronously through a bounded cache keyed by source, math style, theme/color, size, and display scale so unchanged formulas are reused without adding work to every keystroke.
- Show a localized, bounded source fallback for invalid or unsupported expressions, without crashes, stale images, or document-state mutation from late render completions.
- Embed valid formulas as self-contained, sanitized inline SVG in styled and plain HTML exports while retaining authored LaTeX metadata and an accessible/source fallback.
- Preserve the existing per-document-version derived-state caches and keep GPUI types out of workspace member crates.

Non-goals: full TeX or arbitrary package compatibility, `\(...\)` / `\[...\]` delimiter aliases, MathML authoring, interactive rendered-tree formula editing, or upgrading DOCX/PDF/PNG/JPEG export math in this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `code-and-math`: Replace preview-only Unicode math fallback with semantic, high-quality inline and display rendering, bounded failure behavior, and performance requirements while retaining format-specific fallbacks where unchanged.
- `markdown-editing`: Define baseline-aligned inline math and source-backed focus/edit behavior for math in Split Preview, Read, and Visual Edit.
- `export`: Require self-contained SVG math in HTML exports while preserving authored LaTeX metadata and safe fallback behavior; other export formats remain unchanged.

## Impact

- Affects Markdown event-to-preview modeling, inline rich-text layout, block preview rendering, Visual Edit source islands, hit testing/selection, localization, and HTML export.
- Introduces a pinned pure-Rust math typesetting dependency behind a GPUI-free adapter, subject to a compatibility and license spike before integration, plus reuse or extraction of the existing SVG rasterization path in the root app crate.
- Extends render cache keys and asynchronous completion handling while preserving document-version-derived `Arc` caches, memoized highlighting, cached text handles, and the invariant that typing does not synchronously typeset formulas.
- Requires focused unit, integration, snapshot/fixture, and visual verification across Windows, macOS, and Linux display scales and themes.
