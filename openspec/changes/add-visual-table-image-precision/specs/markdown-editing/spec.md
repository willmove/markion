## MODIFIED Requirements

### Requirement: Visual Edit SHALL provide selection-contextual formatting controls
When Visual Edit owns a non-empty, exactly source-mapped text selection, Markion SHALL present contextual controls for strong emphasis, emphasis, inline code, and link editing. A selection that lies entirely inside one Visual Edit table cell’s exact source range SHALL be treated as exactly source-mapped for those controls, even though the table block’s `editable_runs` are empty. Invoking a control SHALL use the existing canonical Markdown mutation, semantic undo, selection, autosave, and exact UTF-8 source paths. Merely showing, moving, or dismissing the controls SHALL NOT change document state or invalidate derived caches. A selection that crosses a table cell boundary or a `|` delimiter SHALL NOT present an unsafe contextual mutation.

#### Scenario: Selection toolbar formats visual text
- **WHEN** the user selects exactly mapped prose in Visual Edit and invokes Bold, Italic, or Inline Code from the contextual controls
- **THEN** the corresponding canonical Markdown markers are changed through one semantic command
- **AND** one Undo restores the prior source and selection

#### Scenario: Table cell selection is format-safe
- **WHEN** the user selects a non-empty range fully inside one Visual Edit table cell and invokes Bold from the contextual controls
- **THEN** only that cell’s selected bytes are wrapped
- **AND** one Undo restores the prior source and selection

#### Scenario: Ambiguous selection stays conservative
- **WHEN** a selection crosses an ambiguous or source-island boundary, or crosses a GFM table cell boundary
- **THEN** Markion does not present an unsafe contextual mutation for that range
- **AND** raw source editing remains available

### Requirement: GFM table preview blocks carry parser event source ranges
The parser SHALL assign each `PreviewBlock::Table` the source range of the pulldown-cmark table event that produced that block’s rows. The range SHALL be non-empty and SHALL cover the authored GFM table bytes. The parser SHALL NOT assign an empty `0..0` placeholder range, and SHALL NOT zip table cell content to source ranges produced by a separate document scan that can skip tables the CommonMark+GFM parser emits. After derivation, GFM tables SHALL appear in the preview block stream in document (source) order. Nested-in-list table ordering MAY still be restored by sorting blocks on those event source-range starts; that sort SHALL NOT be used to repair invented placeholder ranges. Table cell-editing lookup MAY continue to use a dedicated scan of two-or-more-column tables and is not required to be 1:1 with preview table blocks.

When an HTML comment whose payload is exactly a Markion column-width comment (`<!-- markion-cols:… -->`) immediately precedes a GFM table at the same block nesting, derivation SHALL extend that table’s source range to include the comment and SHALL NOT emit the comment as a separate Html preview block. Cell-editing lookup SHALL continue to use the pipe-table slice inside that range.

#### Scenario: One-column GFM tables keep their event ranges
- **WHEN** the document contains a one-column GFM table (`| header |\n| --- |` with or without body rows) that the CommonMark+GFM parser emits as a table
- **THEN** the corresponding preview table block’s source range is non-empty
- **AND** the source slice for that range contains the table’s header line at its authored offset

#### Scenario: Mixed one-column and multi-column tables stay in document order
- **WHEN** a document has ordinary multi-column GFM tables, then one or more one-column `| command |\n| --- |` tables, then later multi-column result tables
- **THEN** the preview block stream lists those tables in authored source order
- **AND** no result-table block is placed at source offset `0` unless that table is actually authored at the start of the document
- **AND** no table block is inserted between an H2 and the immediately following H3 when the source between them is only blank lines

#### Scenario: Empty placeholder ranges are not used for tables
- **WHEN** the parser emits a GFM table with at least one row
- **THEN** that table’s preview `source_range` is non-empty
- **AND** `source_range.start` equals the start of the pulldown-cmark table event for that table (adjusted for any front-matter body offset), or the start of an absorbed immediately-preceding column-width comment

#### Scenario: Nested list tables still follow document order
- **WHEN** a list item contains a nested GFM table
- **THEN** the preview stream places the list item block before that table
- **AND** the list item’s source range ends no later than the nested table’s source range start

#### Scenario: Column-width comments are absorbed into the following table
- **WHEN** the document contains `<!-- markion-cols:30,70 -->` immediately followed by a two-column GFM table
- **THEN** preview derivation emits one Table block whose source range includes the comment and the table
- **AND** no Html preview block is emitted for that comment
