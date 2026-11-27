## Why

README-style HTML paragraphs lose their horizontal alignment when the text contains a line break or subscript. In Visual Edit, a source editor wrapper also draws a frame around ordinary rendered HTML, making headings, links, and images look like separate cards.

## What Changes

- Apply authored horizontal alignment to each rendered line of standalone HTML text in Read, Split Preview, and Visual Edit.
- Present rendered HTML directly in Visual Edit and retain the collapsible source editor. HTML table grids and expanded source fields retain their own styling.
- Non-goals include browser-level HTML layout and changes to HTML export.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing` specifies aligned HTML text lines and borderless rendered HTML in Visual Edit. Raw HTML remains in the rendered WYSIWYG coverage class.

## Impact

The change affects the shared preview and Visual Edit layout in `src/app/preview.rs` and the Visual Edit coverage evidence. Document source and the per-version derived Markdown cache are unaffected by presentation changes.
