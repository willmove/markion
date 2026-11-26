## Context

Preview rendering (`src/app/preview.rs`), Markdown/HTML parsing (`src/parse.rs`), and Visual Edit inline derivation (`src/visual.rs`) each had narrow gaps where valid Markdown or safe inline HTML lost formatting between parse and paint.

## Architecture & Data Flow

### 1. BlockQuote child rendering & definition-list plain text (`src/app/preview.rs`)
- `compute_preview_blocks` already places `Paragraph`, `Heading`, `ListItem`, `Html`, `Image`, `DefinitionTerm`, `DefinitionDetail`, and (if present) `Rule` into `PreviewBlock::BlockQuote { children }`.
- `preview_block_view` updates the `PreviewBlock::BlockQuote` child loop to dispatch every emitted variant cleanly with `PreviewTextRunId::QuoteChild(child_index)` so text selection, copy, and search continue to address each child run consistently.
- `preview_run_plain_text` adds `PreviewBlock::DefinitionTerm` and `PreviewBlock::DefinitionDetail` under `PreviewTextRunId::Body`, matching `preview_block_runs`.

### 2. GFM Table header emphasis, column alignment & footnotes (`src/app/preview.rs`)
- Add a helper `table_cell_text_align(alignment: TableAlignment)` applying `.text_center()` / `.text_right()` (and flex row justification `justify_center()` / `justify_end()` for mixed flex-wrap cells) to both Read/Split (`PreviewBlock::Table`) and Visual Edit (`visual_table_view` + `visual_table_cell_element`).
- Apply `.when(row_index == 0, |style| style.font_weight(FontWeight::SEMIBOLD))` to header cells in both Read/Split and Visual Edit tables, matching HTML table headers.
- Update Read/Split table cell dispatch (`PreviewBlock::Table`) to check `span.image.is_some() || span.math.is_some() || span.footnote.is_some()` so footnote cells get hover tooltips and superscript geometry.
- Wrap Read/Split footnote spans inside `rich_text_with_math_element` with `script_fragment(...)`.

### 3. Safe inline-HTML CSS tolerance (`src/parse.rs`)
- Extend `parse_css_color` to recognize:
  - Existing `#RGB`, `#RRGGBB`, `rgb(r, g, b)`
  - `rgba(r, g, b, a)` (validating `r,g,b: u8` and finite `a` in `0.0..=1.0`, extracting RGB `u32`)
  - Standard CSS basic/common named colors (`black`, `white`, `red`, `green`, `blue`, `yellow`, `orange`, `purple`, `pink`, `gray`/`grey`, `silver`, `maroon`, `olive`, `lime`, `aqua`/`cyan`, `teal`, `navy`, `fuchsia`/`magenta`, `brown`, `gold`, `coral`, `crimson`, `indigo`, `violet`, `darkred`, `darkgreen`, `darkblue`, `darkgray`/`darkgrey`, `lightgray`/`lightgrey`)
  - Continue rejecting `url(`, `var(`, `expression`, invalid hex/channels.
- Extend `parse_html_inline_element` `style` declaration handling:
  - `font-weight`: `bold`, `bolder`, numeric `600..=900` -> `style.bold = true`; `normal` / `100..=500` -> accepted no-op.
  - `font-style`: `italic`, `oblique` -> `style.italic = true`; `normal` -> accepted no-op.
  - `text-decoration` / `text-decoration-line`: `underline` -> `style.underline = true`, `line-through` -> `style.strikethrough = true`, `none` -> accepted no-op.
  - `background-color` / `background`: valid `parse_css_color(value)` -> `style.highlight = true`.
  - Benign typography-only properties (`font-size`, `font-family`, `line-height`, `letter-spacing`, `white-space`): accepted as no-ops when their value does not contain `url(`, `var(`, or `expression`, so `<span style="font-weight:bold; font-size:18px">` preserves bold.
  - Layout/unsafe properties (`position`, `display`, `float`, `transform`, `z-index`, etc.) continue to return `None`, keeping Visual Edit conservative.
- Because `HtmlInlineState` is shared by Preview, Visual Edit, HTML blocks, and HTML tables, all four surfaces gain consistent behavior automatically.

### 4. Cross-span extended inline syntax (`src/parse.rs` & `src/visual.rs`)
- **Preview (`src/parse.rs`)**: Add `apply_cross_span_extended_spans(spans: Vec<InlineSpan>) -> Vec<InlineSpan>` invoked inside `finish_soft_rich_text` before span merging. It only matches pairs where the opening delimiter (`==`, `^`, `~` excluding `~~`) and closing delimiter lie in **different** non-code, non-math, non-image, non-footnote spans on the same line (same-span pairs were already resolved by `append_extended_text`, leaving single-span escaped forms like `\==hi==` untouched). Matched delimiters are stripped from the boundary spans and the corresponding `highlight`/`superscript`/`subscript` flag is applied to all enclosed spans.
- **Visual Edit (`src/visual.rs`)**: Add `apply_cross_run_extended_inline(text: &str, runs: &mut Vec<VisualInlineRun>, candidates: &mut Vec<RevealCandidate>)` immediately before `build_reveal_groups`. It matches eligible identity runs separated across intervening styled/code runs on the same line, checks `!is_backslash_escaped` against the canonical source `text`, validates the combined slice with `extended_inline_matches` so `reveal_group_is_exact` is guaranteed to pass, trims the delimiter bytes out of the boundary runs (leaving them as hidden marker ranges), sets the extended style flag on enclosed runs, and registers an exact `RevealCandidate`.
- Caching/versioning: All parsing still runs solely during per-version `compute_preview_blocks` / `compute_visual_blocks` derivation cached in `Arc`.

## Verification Plan
- Unit tests in `src/parse.rs` and `src/inline_html_tests.rs` for CSS weights (`600`/`800`/`bolder`), `oblique`, named/`rgba` colors, `background-color`, mixed benign CSS (`font-weight:bold; font-size:18px`), continued rejection of `position:fixed`, and cross-span `==**bold**==`, `==\`code\`==`, `H~**2**~O`, `x^*2*^`.
- Unit tests in `src/visual.rs` verifying cross-run `==**bold**==`, `==\`code\`==`, `H~**2**~O` render styled unfocused, reveal exact source ranges when focused, and preserve escaped `\==**bold**==`.
- Preview/App tests verifying blockquote nested `Heading`/`Image`/`DefinitionTerm`/`DefinitionDetail` rendering & plain-text selection, GFM table header weight + alignments + footnotes, and Read/Split footnote superscript geometry.
- Full `LC_ALL=C cargo test --workspace` pass.
