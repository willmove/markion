## MODIFIED Requirements

### Requirement: Document-ordered block stream for list-nested blocks

When a list item contains nested block constructs (fenced code blocks, tables, blockquotes, HTML blocks), the parsed preview block stream SHALL present the list item and each nested block as separate blocks in document (source) order: the list item's text content appears before any block nested inside it. A list item block's source range SHALL NOT swallow the source range of a nested block that is also emitted as its own block; consumers that assume monotonically ordered, disjoint block source ranges SHALL NOT encounter an overlap from this pattern. Each nested block emitted while list containers are open SHALL record how many list levels enclose it (0 when not inside any list), so renderers can place it inside its owning list.

#### Scenario: List item with trailing nested fenced code block

- **WHEN** a list item's text is followed by a fenced code block indented to nest inside that same item
- **THEN** the parsed block stream contains the list item block before the code block
- **AND** the list item's source range ends no later than the nested code block's source range start
- **AND** the code block records a list nesting depth of 1

#### Scenario: Multiple list items each with nested code

- **WHEN** several sibling list items each contain a nested fenced code block
- **THEN** the block stream alternates item, code block, item, code block in source order
- **AND** reading mode renders each code block below the bullet it belongs to, indented to that item's content column

#### Scenario: Indented code block inside a nested ordered list

- **WHEN** a nested ordered list item's marker is followed by five or more spaces, so CommonMark parses the item's content as an indented code block (e.g. `2. Item 2` followed by `    1.     Sub item 1` and `    2. sub item 2`)
- **THEN** the block stream contains the outer item, the empty nested item, the indented code block recording a list nesting depth of 2, and the following nested item, in source order
- **AND** reading mode renders the code block indented to the nested item's content column, between the nested items it belongs to

#### Scenario: List items without nested blocks are unaffected

- **WHEN** a list contains only plain or inline-formatted items
- **THEN** block variants, content, ordering, and source ranges are unchanged from CommonMark event order

### Requirement: Visual Edit list items with nested fenced code blocks

In Visual Edit mode, a list item containing a nested fenced code block SHALL render the item's text as one normal, directly editable list row and the nested code block as one source-backed code editor row, in source order. The nested code editor row SHALL be indented to the owning list item's content column (the same inset the item's own content sits at, one marker column past the item's indent step), so the code block stays visually inside its list. Neither the item text nor the code content SHALL appear twice on screen, and neither SHALL fall back to a conservative raw-source box solely because of the nesting structure.

#### Scenario: Nested fence renders item row plus code editor row

- **WHEN** Visual Edit displays a list item whose indented continuation contains a fenced code block
- **THEN** the item's bullet and inline content render as a normal editable list row
- **AND** the fenced code block renders below it as the code editor row used for top-level fences, indented to the item's content column
- **AND** no raw `- `, link syntax, or literal fence markers are shown for either row

#### Scenario: Indented code block nested two list levels deep

- **WHEN** Visual Edit displays a nested list whose item content is an indented code block (marker followed by five or more spaces)
- **THEN** the nested item's marker renders as a list row at its list level
- **AND** the indented code block renders as the code editor row indented to that nested item's content column
- **AND** editing the code payload remains source-backed with unchanged caret mapping

#### Scenario: No duplicated content

- **WHEN** Visual Edit displays the document region spanning such a list item and its nested code block
- **THEN** every source byte of the region is owned by exactly one visual row
- **AND** no row is force-marked as an unsupported source island due to range overlap

#### Scenario: Editing either row stays source-backed

- **WHEN** the user edits the item text row or the nested code payload in Visual Edit
- **THEN** the edit applies to the canonical Markdown source through the existing mutation paths
- **AND** the other row's rendered content remains intact
