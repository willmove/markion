## Context

The importer serializes each text run independently. Word also stores heading bookmarks and Office Math as separate XML elements, including properties that control the visual presentation of equations.

## Decisions

- Store the first bookmark on a heading as its Markdown heading identifier. Keep standalone targets for other bookmarks and keep IDs unique across the document.
- Coalesce adjacent text with the same effective run formatting before writing emphasis. Preserve source spacing and use supported inline HTML where Markdown delimiters would collide or fail to flank the text.
- Translate the Office Math structures used in the reference document with bounded XML traversal. Preserve operators, delimiters, limits, accents, and alphabet styles in editable LaTeX. Keep the existing fallback and diagnostic for unsupported content.
- Convert semantic quote styles and Word horizontal-rule shapes to Markdown blocks. Use GFM alignment when the column's paragraphs agree. Diagnose authored endnote references.
- Request formula renders from the active virtualized list viewport. Use a bounded window while list items are still unmeasured and submit distinct formula keys in document order. This keeps offscreen formulas from repeatedly evicting visible ones.
- Include a plain-text math summary in headings through the same outline builder in both derivation paths. Keep existing automatic fragment IDs to preserve links in older Markdown.
- Use the same flex-row alignment for mixed text and math in read-only and editable table cells.

## Risks / Trade-offs

Markdown cannot express Word page layout. Verify equation conversion against the native math renderer because parseable Markdown can still carry unsupported LaTeX.
