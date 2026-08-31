## ADDED Requirements

### Requirement: GFM table preview blocks carry parser event source ranges

The parser SHALL assign each `PreviewBlock::Table` the source range of the pulldown-cmark table event that produced that block’s rows. The range SHALL be non-empty and SHALL cover the authored GFM table bytes. The parser SHALL NOT assign an empty `0..0` placeholder range, and SHALL NOT zip table cell content to source ranges produced by a separate document scan that can skip tables the CommonMark+GFM parser emits. After derivation, GFM tables SHALL appear in the preview block stream in document (source) order. Nested-in-list table ordering MAY still be restored by sorting blocks on those event source-range starts; that sort SHALL NOT be used to repair invented placeholder ranges. Table cell-editing lookup MAY continue to use a dedicated scan of two-or-more-column tables and is not required to be 1:1 with preview table blocks.

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
- **AND** `source_range.start` equals the start of the pulldown-cmark table event for that table (adjusted for any front-matter body offset)

#### Scenario: Nested list tables still follow document order

- **WHEN** a list item contains a nested GFM table
- **THEN** the preview stream places the list item block before that table
- **AND** the list item’s source range ends no later than the nested table’s source range start
