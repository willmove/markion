## 1. BlockQuote Children & Footnote Superscript in Preview

- [x] 1.1 Render `Heading`, `DefinitionTerm`, `DefinitionDetail`, `Image`, and `Rule` children inside `PreviewBlock::BlockQuote` in `src/app/preview.rs`, and include `DefinitionTerm`/`DefinitionDetail` in `preview_run_plain_text`.
- [x] 1.2 Wrap Read/Split preview footnote references in `script_fragment` in `rich_text_with_math_element` (`src/app/preview.rs`).

## 2. GFM Table Header Weight, Alignment & Footnote Routing

- [x] 2.1 Apply `FontWeight::SEMIBOLD` to header rows and parsed `TableAlignment` (`Left`/`Center`/`Right`) to cells in Read/Split (`PreviewBlock::Table`) and Visual Edit (`visual_table_view` / `visual_table_cell_element`) in `src/app/preview.rs`.
- [x] 2.2 Route Read/Split table cells containing footnotes (`span.footnote.is_some()`) through `rich_text_with_math_element`.

## 3. Inline HTML CSS Tolerance & Colors

- [x] 3.1 Extend `parse_css_color` and `parse_html_inline_element` in `src/parse.rs` to support numeric bold weights (`600..=900`, `bolder`), `oblique`, CSS named colors and `rgba(...)`, `background-color` highlight, and benign co-occurring typography declarations while continuing to reject layout/unsafe properties.

## 4. Cross-Span Extended Inline Syntax (`==`, `^`, `~`)

- [x] 4.1 Implement `apply_extended_delimiters_line` in `src/parse.rs` so `==**bold**==`, `==\`code\`==`, `H~**2**~O`, `x^*2*^`, etc. apply extended formatting across enclosed spans left-to-right without disturbing escapes.
- [x] 4.2 Implement `apply_extended_inline_runs` in `src/visual.rs` so Visual Edit matches Preview styling and produces exact progressive-reveal groups for cross-run extended inline constructs.

## 5. Tests & Verification

- [x] 5.1 Add unit and app tests covering blockquote child rendering, table header/alignment/footnotes, inline HTML CSS & colors, and cross-span extended inline syntax in Preview and Visual Edit.
- [x] 5.2 Run `LC_ALL=C cargo test --workspace` and verify all tests pass, then commit cleanly.
