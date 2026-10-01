## Context

pulldown-cmark 0.13 emits `DefinitionList` > `DefinitionListTitle` (inline content) and `DefinitionListDefinition`, which holds inline content when tight and paragraphs or other blocks when loose. Its `DefinitionList` range can overrun into a following paragraph, so only title and definition ranges are trustworthy. The preview builder is one event loop over flat leaf blocks, with containers (quotes) collecting children. Visual Edit rows derive from those leaves, and line-start markers are modeled as block prefixes with progressive reveal.

## Decisions

### 1. Flat leaf blocks
`DefinitionTerm { text, source_range }` and `DefinitionDetail { text, source_range }` are leaves, like `ListItem`. There is no container block. This keeps the Visual Edit row model (one row per leaf, one owner per byte) and the splice diffing unchanged.

### 2. Building details
- The title reuses the `paragraph` sink, so inline text routing (`push_preview_rich`) needs no new branch.
- A definition opens a `definition` accumulator and also a direct `paragraph` sink for tight content. A loose definition's own paragraphs flush into the accumulator, joined by line breaks, mirroring footnote definitions.
- When a non-paragraph block starts inside a definition, the detail is emitted immediately, with its range ending at the nested block's start. The definition then closes, so the nested block and anything after it render as ordinary blocks in source order.
- Inside a blockquote, terms and details go to the quote's children.

### 3. Visual Edit marker
`VisualBlockPrefixKind::DefinitionMarker` covers `:` plus following spacing. Inline runs start after the prefix (the same branch as list items). The marker is therefore hidden until the caret enters the row, using the existing prefix-reveal path.

### 4. Exports
HTML export already runs `push_html` over `markdown_options()`, so `<dl>` output comes with the option. The PDF IR has no indented paragraph, so details are plain paragraphs there. DOCX uses `push_paragraph(.., indent_left = 720)`. LaTeX uses `\textbf{…}` for the term and a `quote` environment for the detail. `RichText::emboldened()` makes term runs bold for all three.

## Risks / Trade-offs

- [Existing documents with `:`-led lines after a paragraph change rendering] → This is the intended syntax, the same as other editors, and the change is purely presentational: the source is untouched.
- [Empty detail row when a definition starts directly with a nested block] → It owns only `:` and spacing, renders as an empty indented row, and stays editable. This is consistent with empty list items.
