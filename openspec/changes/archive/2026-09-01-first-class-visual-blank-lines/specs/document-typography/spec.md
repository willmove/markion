## ADDED Requirements

### Requirement: Visual Edit authored blank lines follow body line height
Visual Edit SHALL paint each authored blank-line (`Whitespace`) row at the rendered body paragraph line height derived from the rendered-document font size, one painted line per covered newline (floored at one line, capped at the existing pathological bound). Changing the rendered-document font size SHALL reflow those rows. Changing the paragraph-spacing preference SHALL NOT change blank-line row height and MUST NOT insert, remove, or rewrite authored blank lines. Typography-only reflow MUST NOT increment the Markdown document version or rebuild per-version derived caches.

#### Scenario: Blank-line height tracks rendered body size
- **WHEN** Visual Edit displays a document that contains a blank line between two headings
- **THEN** that `Whitespace` row occupies the current rendered body paragraph line height rather than a fixed 12px strip
- **AND** increasing or decreasing the rendered-document font size reflows the blank-line row on the next render

#### Scenario: Paragraph spacing does not absorb authored blank lines
- **WHEN** the user changes the paragraph-spacing preference while Visual Edit shows authored blank lines between blocks
- **THEN** those `Whitespace` rows keep body paragraph line height
- **AND** the document text, dirty state, undo history, and authored blank lines remain unchanged
