# html-table-rendering

## Purpose

Render raw HTML `<table>` blocks embedded in Markdown as visual tables in preview, Read mode, and Visual Edit, including cells that span rows or columns, instead of flattening cell text into a single run.

## Requirements

### Requirement: CRLF cover-sheet HTML tables still render as one grid

A raw HTML `<table>` that occupies one CommonMark HTML block in the source (no blank line between tags) SHALL parse and render as a single visual table grid in Split Preview, Read mode, and Visual Edit even when the file uses CRLF line endings. Cell text from distinct `<td>`/`<th>` elements SHALL NOT concatenate into one run. Visual Edit SHALL map that table to one HTML visual block, not one card per source line.

#### Scenario: Cover-sheet label and value stay split on CRLF source

- **WHEN** the document is the cover-sheet table with a left `rowspan` strut, a right `rowspan` strut, and a body row `<td>文档版本</td><td colspan="2">01</td>`, stored with CRLF between `<tr>` lines and no blank lines
- **THEN** Split Preview, Read mode, and Visual Edit render `文档版本` and `01` in two distinct columns of the same row
- **AND** the strings are not presented as a concatenated run such as `文档版本01`
- **AND** Visual Edit does not show a separate bordered HTML card per table row

#### Scenario: Cover-sheet grid on LF source is unchanged

- **WHEN** the same cover-sheet markup uses LF line endings
- **THEN** it still renders as one table grid with `文档版本` and `01` in different columns
