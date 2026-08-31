## ADDED Requirements

### Requirement: GFM tables render at their authored source position

The editor SHALL render every GFM pipe table that the CommonMark+GFM parser emits as a visual table at the table’s authored source position in Split Preview, Read mode, and Visual Edit. This includes one-column tables (a single header cell and delimiter row, with or without body rows) mixed in the same document with later multi-column tables. A table SHALL NOT appear earlier in the rendered stream than its source offset, and later tables SHALL NOT inherit source ranges that belong to earlier tables. Two-or-more-column tables SHALL keep their existing cell-editing and toolbar behavior; one-column tables MAY remain non-editable at the cell/toolbar layer when exact cell bounds cannot be proven.

#### Scenario: One-column command tables stay in place

- **WHEN** the document contains a one-column GFM table such as `| command |\n| --- |` between surrounding prose or headings
- **THEN** Split Preview, Read mode, and Visual Edit render that table as a visual grid at that source location
- **AND** the table does not appear at the start of the document unless it is authored there

#### Scenario: Later multi-column result tables are not hoisted

- **WHEN** a document begins with headings separated only by blank lines and contains `Dies | Throughput` (or other multi-column) tables much later, after earlier one-column GFM tables
- **THEN** those later tables render at their authored offsets
- **AND** they do not appear between the leading headings whose source gap contains only whitespace

#### Scenario: Two-column cell editing still targets the caret’s table

- **WHEN** the user edits a cell or uses the Visual Edit table toolbar on a two-or-more-column GFM table after preview table ranges are taken from parser events
- **THEN** the mutation still replaces that table’s source bytes
- **AND** it does not edit a different table in the document
