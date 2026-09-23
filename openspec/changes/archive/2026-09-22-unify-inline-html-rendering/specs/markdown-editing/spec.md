## ADDED Requirements

### Requirement: Shared inline HTML semantics across rendered contexts

Read, Split Preview and Visual Edit SHALL consistently render supported inline HTML in Markdown paragraphs, headings, lists, blockquotes and GFM cells, and in standalone HTML content. Supported elements SHALL include strong/b, em/i, s/del/strike, u/ins, code/kbd/samp, mark, sub/sup, neutral span, colored span/font, and a anchors. Supported inert attributes SHALL not expose tags or discard the element's styling. Nested styles and links SHALL restore their enclosing state when closed. Escaped or entity-decoded tag text SHALL remain literal.

#### Scenario: Mixed prose retains colored linked formatting
- **WHEN** prose contains a colored span with bold content and an HTML anchor
- **THEN** all rendered modes retain the color, bold style and decoded destination with hidden tag syntax
- **AND** following text restores its previous style and link

#### Scenario: Visual HTML remains exactly editable
- **WHEN** the caret enters supported HTML, including an anchor wrapping an image
- **THEN** Visual Edit reveals its exact safe containing source group and preserves UTF-8 mapping
- **AND** edits use the existing mutation and undo paths without rewriting unrelated markup

#### Scenario: Harmless wrappers and aliases stay rendered
- **WHEN** text uses span wrappers, title/class/id metadata or kbd/samp tags
- **THEN** neutral wrappers preserve content and code-like aliases use the code style
- **AND** malformed or unsupported forms retain source-backed fallback

### Requirement: Explicit inline HTML breaks are preserved

Every authored br element SHALL contribute one rendered line break in mixed Markdown and shared HTML rendering, including consecutive, leading and trailing breaks. Structural HTML boundaries SHALL not accidentally merge or multiply explicit breaks.

#### Scenario: Repeated and edge breaks survive
- **WHEN** content contains br elements at its beginning, end or consecutively in the middle
- **THEN** each explicit break occupies its corresponding line transition in every rendered mode
- **AND** Visual Edit retains the exact tag source mapping
