## ADDED Requirements

### Requirement: CRLF HTML events coalesce into one preview block

When pulldown-cmark emits consecutive `Event::Html` pieces whose source ranges are separated only by whitespace that is not a CommonMark blank line (including a lone CR left after CRLF normalization), the parser SHALL emit a single `PreviewBlock::Html`. That block’s `source_range` SHALL be the contiguous span from the first piece through the last, and its `html` string SHALL be the corresponding slice of canonical document text (not a concatenation of event payloads that omit CR). Two HTML blocks separated by a blank line SHALL remain two preview blocks. The same coalescing SHALL apply to HTML accumulated into list items and blockquotes. Incremental source-mapped derivation SHALL match this full-parse result.

#### Scenario: CRLF table lines become one HTML preview block

- **WHEN** the document contains a multi-line raw HTML `<table>…</table>` whose line endings are CRLF and which contains no blank line between tags
- **THEN** `preview_blocks()` contains exactly one `PreviewBlock::Html` for that table
- **AND** that block’s `source_range` covers the authored `<table` through `</table>`
- **AND** the block’s `html` matches the document slice for that range

#### Scenario: LF table lines stay one HTML preview block

- **WHEN** the same table markup uses LF line endings
- **THEN** `preview_blocks()` still contains exactly one `PreviewBlock::Html` for that table

#### Scenario: Blank line keeps two HTML blocks apart

- **WHEN** the document contains two complete raw HTML blocks separated by a blank line (for example two `<p>…</p>` blocks)
- **THEN** `preview_blocks()` contains two `PreviewBlock::Html` entries in document order
