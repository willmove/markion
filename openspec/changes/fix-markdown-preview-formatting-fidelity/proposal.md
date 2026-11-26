## Why

End-to-end auditing of Markion's Markdown and HTML rendering pipeline identified five distinct formatting-loss defects across Split Preview, Read mode, and Visual Edit:

1. Read/Split preview silently skips `Heading`, `DefinitionTerm`, `DefinitionDetail`, `Image`, and `Rule` children inside `PreviewBlock::BlockQuote` even though the parser emits them into `children`, and `preview_run_plain_text` omits top-level `DefinitionTerm`/`DefinitionDetail` runs.
2. GFM Markdown tables in Split Preview, Read mode, and Visual Edit ignore parsed column `alignments` (`Left`, `Center`, `Right`) and omit header-row semibold emphasis (unlike HTML tables), and Read/Split table cells bypass the mixed-content path for footnote-only cells (losing footnote hover tooltips).
3. Footnote references in Read/Split preview are not wrapped in `script_fragment`, so they render at full body size/baseline rather than superscript geometry (diverging from Visual Edit and `^sup^`).
4. Inline HTML `style` parsing rejects common CSS formatting values (`font-weight: 600`/`800`/`900`/`bolder`, `font-style: oblique`, named CSS colors like `red`, `rgba(...)`, and `background-color`), and rejects the entire tag whenever harmless typography properties like `font-size` appear alongside `font-weight: bold`.
5. Extended inline delimiters (`==highlight==`, `^sup^`, `~sub~`) only match within a single `Event::Text` fragment, so wrapping standard Markdown syntax (such as `==**bold**==`, `==\`code\`==`, or `H~**2**~O`) loses the outer extended formatting in both Preview and Visual Edit.

## What Changes

- Render all supported child block variants (`Heading`, `DefinitionTerm`, `DefinitionDetail`, `Image`, `Rule`, in addition to `Paragraph`, `Html`, `ListItem`) inside `PreviewBlock::BlockQuote` in Read/Split preview, and include `DefinitionTerm`/`DefinitionDetail` in `preview_run_plain_text`.
- Apply `FontWeight::SEMIBOLD` to GFM Markdown table header rows and apply parsed `TableAlignment` (`Left`/`Center`/`Right`) to GFM table cells in Split Preview, Read mode, and Visual Edit; route Read/Split table cells containing footnotes through `rich_text_with_math_element`.
- Wrap Read/Split preview footnote references in `script_fragment` so they render with superscript size and raised baseline matching Visual Edit.
- Broaden safe inline-HTML CSS parsing in `src/parse.rs` to support `font-weight` (`600..=900`, `bolder`, `normal`), `font-style: oblique`/`normal`, `text-decoration-line`, `background-color` (mapped to highlight), standard CSS named colors and `rgba(...)`, and harmless co-occurring typography declarations (`font-size`, `font-family`, `line-height`, `letter-spacing`) while still rejecting unsafe/layout CSS properties (such as `position:fixed`, `url(...)`, `var(...)`, `expression(...)`) and event-handler attributes.
- Add cross-span/cross-run resolution for extended inline delimiters (`==highlight==`, `^sup^`, `~sub~`) wrapping standard Markdown formatting in both Preview (`src/parse.rs`) and Visual Edit (`src/visual.rs`) while preserving backslash escapes and exact Visual Edit source-reveal mappings.
- **Non-goals:** changing CommonMark flanking-delimiter rules for CJK punctuation, changing GPUI `HighlightStyle` to support per-span inline-code font families, or redesigning Visual Edit blockquote container chrome for nested blocks.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: require Read/Split blockquotes to render all emitted child block types, Read/Split footnote references to render with superscript geometry, safe inline-HTML CSS formatting tolerance (weights, oblique, named/`rgba` colors, background highlight, benign co-occurring typography declarations), and cross-span extended inline syntax (`==...==`, `^...^`, `~...~`) composing around standard Markdown inline formatting in both Preview and Visual Edit.
- `tables-outline`: require GFM Markdown tables to render header rows with semibold emphasis, honor parsed column alignments in Read, Split Preview, and Visual Edit, and preserve footnote tooltips in preview table cells.

## Impact

- `src/app/preview.rs`: blockquote child rendering, `preview_run_plain_text` definition-list coverage, GFM table header weight + alignment + footnote routing, and Read/Split footnote `script_fragment`.
- `src/parse.rs`: CSS color/weight/style/background-color support and cross-span extended inline resolution in `finish_rich_text`.
- `src/visual.rs`: cross-run extended inline resolution with exact `RevealCandidate` ranges before `build_reveal_groups`.
- Unit and integration tests in `src/parse.rs`, `src/inline_html_tests.rs`, `src/visual.rs`, and `src/app/tests.rs`.
- Preserves all architecture invariants: derived preview/visual blocks remain cached per document version via `Arc`; no new dependencies; workspace members under `crates/*` remain GPUI-free.
