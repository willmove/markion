## ADDED Requirements

### Requirement: Quoted GFM tables retain exact authored cell editing

When a GFM table belongs to a block quote, Markion SHALL render it inside the quote. Visual Edit SHALL map each provable authored cell to its exact source bytes and allow cell-local edits without altering quote prefixes, neighboring content, table alignment, or line endings.

#### Scenario: Quote a table from its grip

- **WHEN** the user opens a GFM table's block grip menu in Visual Edit and activates Quote
- **THEN** Visual Edit paints a quoted table grid with editable authored cells
- **AND** one Undo restores the original table and its pre-command selection

#### Scenario: Edit a quoted Unicode table

- **WHEN** the user types inside a cell of a quoted GFM table containing multibyte text with CRLF line endings
- **THEN** the edit targets that cell's canonical source bytes and preserves its quote markers, separator row, neighboring cells, and line endings
