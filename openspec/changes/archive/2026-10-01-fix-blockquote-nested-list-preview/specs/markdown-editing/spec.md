## ADDED Requirements

### Requirement: Blockquote nested content stays inside the quote in preview
The derived preview model SHALL keep content nested inside a blockquote attached to that blockquote. List items (ordered, unordered, and task list items) authored inside a blockquote SHALL be derived as children of the blockquote's preview block, never as top-level preview blocks. The rendered preview SHALL display those nested list items inside the blockquote container (within its left border and quote styling), with ordered items numbered according to the list's start index and successive items, unordered items shown with bullet markers, and task list items shown with their checked state. Nested list levels inside a blockquote SHALL retain their relative indentation level. Text extraction, statistics, export, math collection, and preview selection/copy SHALL include the text of list items nested inside blockquotes.

#### Scenario: Ordered list inside a blockquote renders inside the quote
- **WHEN** the document contains a blockquote with an ordered list (e.g. `> 1. first` / `> 2. second`)
- **THEN** the preview renders the list items inside the blockquote container, numbered 1, 2, ...
- **AND** no top-level list block for those items appears outside the quote

#### Scenario: Ordered list start index is honored inside a blockquote
- **WHEN** the document contains a blockquote whose ordered list starts at a number other than 1 (e.g. `> 3. third`)
- **THEN** the preview numbers the nested items starting from that number

#### Scenario: Unordered and task lists inside a blockquote
- **WHEN** the document contains a blockquote with unordered or task list items
- **THEN** the preview renders them inside the blockquote container with bullet markers or the correct checked state

#### Scenario: Nested list levels inside a blockquote
- **WHEN** the document contains a blockquote with a list that itself contains a nested list
- **THEN** the preview renders all levels inside the blockquote container with relative indentation preserved

#### Scenario: Quoted list text reaches derived consumers
- **WHEN** the document contains a blockquote with list items and the user views stats, selects and copies preview text spanning the quote, exports the document, or the quote contains inline math
- **THEN** the list item text is included in the statistics, copied text, exported output, and math rendering respectively

#### Scenario: Blockquote without nested blocks is unchanged
- **WHEN** the document contains a blockquote with only paragraph text
- **THEN** the derived preview block and its rendering are unchanged from previous behavior
