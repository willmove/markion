## Context

Preview already has the parser's source range for every GFM table. Editing independently searches through consecutive pipe-containing lines and then splits those lines with a different backslash rule.

## Decisions

- Resolve editing ranges from the versioned preview blocks. Width comments can extend a table block only across whitespace at the same nesting. Source commands and the inline-format guard use the table slice after any absorbed comment, and Visual Edit continues to use the same parser-owned block it displays.
- Share the GFM pipe delimiter scan between cell splitting and caret column lookup. A pipe immediately following one or more backslashes stays in its authored cell. Preserve raw cell text so Format can change spacing without decoding or re-escaping literal pipes.
- Use the owned header and its next line as the delimiter row. Keep the existing ragged cell behavior. A structurally unproven range cannot authorize a rewrite. Format only the owned table span and keep its line ending and continuation indentation.

## Risks and Trade-offs

- Visual Edit and preview can reuse their existing document-version cache. The first explicit table lookup in Source mode can populate that same cache.
- The parser pads and truncates rendered rows to the header width. Preserve the existing authored-cell representation for ragged rows so edits do not discard extra source cells.
