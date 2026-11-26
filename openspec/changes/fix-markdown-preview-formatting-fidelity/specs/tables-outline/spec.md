## ADDED Requirements

### Requirement: GFM Markdown tables render header emphasis, column alignments, and cell footnotes
In Split Preview, Read mode, and Visual Edit, GFM Markdown tables SHALL render the header row (`row_index == 0`) with semibold font weight (`FontWeight::SEMIBOLD`), matching HTML table headers, and SHALL apply parsed column alignments (`Left`, `Center`, `Right`) from the GFM delimiter row to each cell's horizontal text/content alignment. In Split Preview and Read mode, table cells containing footnote references SHALL route through the mixed inline renderer so footnote superscript geometry and hover tooltips are preserved.

#### Scenario: GFM table header renders semibold and respects column alignment
- **WHEN** a document contains a GFM table with `| :--- | :---: | ---: |` alignments
- **THEN** Split Preview, Read mode, and Visual Edit render row 0 with semibold weight and align column 0 left, column 1 centered, and column 2 right

#### Scenario: Footnote inside a preview table cell keeps superscript and tooltip
- **WHEN** a GFM table cell contains a footnote reference `[^1]`
- **THEN** Split Preview and Read mode render the footnote marker as a superscript with its hover tooltip rather than plain text
