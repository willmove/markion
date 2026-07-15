## MODIFIED Requirements

### Requirement: Markdown parsing via CommonMark + GFM
The parser SHALL parse Markdown using `pulldown-cmark` configured for CommonMark conformance plus the GitHub Flavored Markdown extensions in use (tables, task lists, strikethrough, footnotes, superscript/subscript, highlight, autolinks). Parsing SHALL produce structured data consumed by the preview, outline, stats, and search subsystems. Standalone raw HTML regions SHALL remain available to the rendered preview as HTML blocks in source order, and their user-visible text SHALL preserve collapsible separators across adjacent styled or linked elements.

#### Scenario: Full reparse per edit yields structured blocks
- **WHEN** the document text changes
- **THEN** the parser runs over the full document text and produces the structured preview blocks, outline, stats, and line count consumed downstream

#### Scenario: Extended inline syntax is recognized
- **WHEN** the document contains `==highlight==`, `^superscript^`, `~subscript~`, task list items, or footnote references
- **THEN** the parser recognizes these constructs and the preview renders them with their respective styles

#### Scenario: Nested Markdown constructs are preserved
- **WHEN** a block construct contains inline or nested constructs (e.g. a list with nested code, a blockquote with a table)
- **THEN** the parser handles the nesting per CommonMark precedence rules

#### Scenario: Standalone raw HTML reaches the rendered preview
- **WHEN** the document contains a standalone raw HTML region followed by a Markdown block
- **THEN** the preview contains one source-faithful HTML block for that region followed by the Markdown block in source order

#### Scenario: Visible spacing between HTML inline elements is preserved
- **WHEN** a raw HTML text block contains collapsible whitespace between adjacent styled or linked inline elements
- **THEN** the preview renders one visible separator at that boundary while preserving the elements' style and link metadata
