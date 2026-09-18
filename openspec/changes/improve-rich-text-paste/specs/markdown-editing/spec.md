## ADDED Requirements

### Requirement: Spreadsheet clipboard paste becomes a GFM table
When Edit → Paste runs on the document editing surface (not a focused text field, not an image-only clipboard), a rectangular tab-separated plain-text payload SHALL be converted to a GitHub Flavored Markdown table and inserted as one atomic undo step. A payload is rectangular TSV when it has at least two lines, every line contains a tab, and every line has the same number of columns after splitting on tabs (trailing empty columns allowed). The first row SHALL become the header row. A single cell (one line with no tab, or a 1×1 table) SHALL paste as ordinary text, not a one-cell table. Pipe characters and newlines inside cells SHALL be escaped so they do not break the GFM table. Image paste and focused text-field paste SHALL remain unchanged.

#### Scenario: Excel TSV pastes as a GFM table
- **WHEN** the clipboard plain text is `姓名\t年龄\n张三\t18` with no usable HTML, and the user pastes into a document
- **THEN** the inserted Markdown is a GFM table whose header is `姓名` / `年龄` and whose body row is `张三` / `18`
- **AND** undoing once removes the entire paste

#### Scenario: Single cell does not become a table
- **WHEN** the clipboard plain text is a single value with no tab
- **THEN** paste inserts that value as ordinary text

#### Scenario: Non-tabular plain text stays verbatim
- **WHEN** the clipboard contains multiple lines of prose without tabs
- **THEN** paste inserts the text byte-for-byte

### Requirement: Headerless HTML tables use the first row as header
When clipboard HTML converts to a table whose rows are all `<td>` cells (no header cells), the converter SHALL emit a GFM table that uses the first row as the header rather than inserting an empty header row.

#### Scenario: Excel HTML table without th
- **WHEN** the clipboard HTML is a `<table>` of `<td>` cells with at least two rows and no `<th>`
- **THEN** the inserted GFM table uses the first data row as its header row
- **AND** it does not contain a blank header row of empty cells

### Requirement: Merged HTML tables flatten to GFM
When clipboard HTML contains a table with `colspan` or `rowspan`, paste SHALL insert a rectangular GFM table: the spanned value occupies the origin cell and the remaining covered cells are empty. Nested tables inside a cell SHALL still fall back to one raw HTML table. The result SHALL remain a single atomic undo step.

#### Scenario: Colspan flattens instead of raw HTML
- **WHEN** the user pastes an HTML table whose first row is one cell with `colspan="2"` and whose second row has two cells
- **THEN** the inserted text is a GFM table, not a serialized `<table>` element
- **AND** the first header cell holds the spanned text and the second header cell is empty

#### Scenario: Nested table still falls back to raw HTML
- **WHEN** the clipboard HTML is a table whose cell contains another table
- **THEN** paste inserts one raw HTML table for that subtree

### Requirement: Word-style clipboard lists become Markdown lists
When clipboard HTML represents a list as Word `MsoListParagraph` paragraphs or `mso-list` styled runs rather than `<ul>`/`<ol>`/`<li>`, paste SHALL reconstruct nested unordered or ordered Markdown lists from those markers and indent levels. Real `<ul>`/`<ol>` lists SHALL continue to convert as they do today.

#### Scenario: Word fake numbered list
- **WHEN** the clipboard HTML is two `MsoListParagraph` paragraphs with `mso-list` level-1 numbering
- **THEN** the inserted Markdown is an ordered list with those two items

#### Scenario: Word nested bullet list
- **WHEN** the clipboard HTML is a level-1 bullet paragraph followed by a level-2 bullet paragraph
- **THEN** the inserted Markdown is a nested unordered list

### Requirement: Invalid clipboard HTML falls back to Unicode text
When the clipboard offers an HTML flavor that cannot be decoded as UTF-8, Edit → Paste SHALL ignore that HTML flavor and insert using the Unicode plain-text payload, including TSV-to-table conversion when that payload is rectangular TSV.

#### Scenario: Non-UTF-8 HTML still pastes Unicode text
- **WHEN** the clipboard carries invalid-UTF-8 HTML alongside Unicode tab-separated text of a two-by-two grid
- **THEN** paste inserts a GFM table built from the Unicode text rather than reporting an empty clipboard or inserting replacement characters from the HTML

### Requirement: Paste as plain text inserts raw clipboard text
The Edit menu SHALL offer Paste as Plain Text, bound by default to `Ctrl+Alt+V` on Windows and Linux and `Cmd+Option+V` on macOS, customizable through the existing shortcut preferences. Invoking it on the document editing surface SHALL insert the clipboard's Unicode plain text with no HTML conversion and no TSV-to-table conversion, as one atomic undo step. Focused text fields SHALL still receive raw text. Image-only clipboards SHALL follow the existing image import flow. Toggle View Mode SHALL keep `Ctrl+Shift+V` / `Cmd+Shift+V`. The action label SHALL be routed through the i18n layer in every supported interface language and SHALL appear in the keyboard shortcut reference.

#### Scenario: Paste as plain text keeps tabs
- **WHEN** the clipboard holds HTML of a table and Unicode TSV, and the user invokes Paste as Plain Text
- **THEN** the document receives the Unicode TSV (tabs and newlines) rather than a GFM table or converted HTML

#### Scenario: Ordinary paste still converts
- **WHEN** the same clipboard is pasted with Edit → Paste
- **THEN** the document receives converted Markdown (GFM table or formatted HTML conversion)

#### Scenario: Shortcut does not steal toggle view
- **WHEN** the user presses `Ctrl+Shift+V` / `Cmd+Shift+V`
- **THEN** the view mode cycles as before
- **AND** Paste as Plain Text is not invoked
