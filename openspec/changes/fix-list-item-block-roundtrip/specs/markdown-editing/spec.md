## ADDED Requirements

### Requirement: Markdown AST serialization preserves list-item block structure
`render_to_markdown` SHALL serialize a list item's child blocks (`ListItem.blocks`) with the blank-line separation CommonMark requires, so that parsing the rendered output reproduces the item's block structure: a child paragraph or other child block SHALL re-parse as a separate child block of the same item rather than merging into the item's inline content as a lazy continuation. Sibling child blocks of one item SHALL likewise be separated so they do not merge with each other. Tight items (inline content only, or inline content followed by nested sub-items) SHALL render as compactly as before, with no added blank lines.

#### Scenario: Loose list item survives round-trip
- **WHEN** a document contains a list item with a second paragraph (e.g. source `- first paragraph\n\n  second paragraph\n`)
- **THEN** rendering the parsed document and re-parsing the output yields a list item that still contains the second paragraph as a separate child block
- **AND** the item's inline content is unchanged

#### Scenario: Sibling child blocks stay separate
- **WHEN** a list item contains more than one child block (e.g. two paragraphs, or a paragraph followed by a fenced code block)
- **THEN** rendering and re-parsing preserves each child block as a distinct block of the item

#### Scenario: Tight items render compactly
- **WHEN** a list item has only inline content, or inline content followed by nested sub-items
- **THEN** the rendered output contains no blank lines inside the item and matches the previous compact rendering

#### Scenario: Export input preserves loose list items
- **WHEN** a document with a multi-paragraph list item is exported through the pandoc-backed path
- **THEN** the Markdown handed to pandoc keeps the list item's paragraph structure intact
