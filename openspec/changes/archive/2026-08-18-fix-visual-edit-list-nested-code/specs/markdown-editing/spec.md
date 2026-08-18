# Delta: markdown-editing

## ADDED Requirements

### Requirement: Document-ordered block stream for list-nested blocks

When a list item contains nested block constructs (fenced code blocks, tables, blockquotes, HTML blocks), the parsed preview block stream SHALL present the list item and each nested block as separate blocks in document (source) order: the list item's text content appears before any block nested inside it. A list item block's source range SHALL NOT swallow the source range of a nested block that is also emitted as its own block; consumers that assume monotonically ordered, disjoint block source ranges SHALL NOT encounter an overlap from this pattern.

#### Scenario: List item with trailing nested fenced code block

- **WHEN** a list item's text is followed by a fenced code block indented to nest inside that same item
- **THEN** the parsed block stream contains the list item block before the code block
- **AND** the list item's source range ends no later than the nested code block's source range start

#### Scenario: Multiple list items each with nested code

- **WHEN** several sibling list items each contain a nested fenced code block
- **THEN** the block stream alternates item, code block, item, code block in source order
- **AND** reading mode renders each code block below the bullet it belongs to

#### Scenario: List items without nested blocks are unaffected

- **WHEN** a list contains only plain or inline-formatted items
- **THEN** block variants, content, ordering, and source ranges are unchanged from CommonMark event order

### Requirement: Visual Edit list items with nested fenced code blocks

In Visual Edit mode, a list item containing a nested fenced code block SHALL render the item's text as one normal, directly editable list row and the nested code block as one source-backed code editor row, in source order. Neither the item text nor the code content SHALL appear twice on screen, and neither SHALL fall back to a conservative raw-source box solely because of the nesting structure.

#### Scenario: Nested fence renders item row plus code editor row

- **WHEN** Visual Edit displays a list item whose indented continuation contains a fenced code block
- **THEN** the item's bullet and inline content render as a normal editable list row
- **AND** the fenced code block renders below it as the code editor row used for top-level fences
- **AND** no raw `- `, link syntax, or literal fence markers are shown for either row

#### Scenario: No duplicated content

- **WHEN** Visual Edit displays the document region spanning such a list item and its nested code block
- **THEN** every source byte of the region is owned by exactly one visual row
- **AND** no row is force-marked as an unsupported source island due to range overlap

#### Scenario: Editing either row stays source-backed

- **WHEN** the user edits the item text row or the nested code payload in Visual Edit
- **THEN** the edit applies to the canonical Markdown source through the existing mutation paths
- **AND** the other row's rendered content remains intact
