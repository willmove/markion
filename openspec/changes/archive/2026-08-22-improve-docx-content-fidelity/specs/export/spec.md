## ADDED Requirements

### Requirement: Built-in DOCX fallback embeds local images
The built-in DOCX writer SHALL embed local image files into the package. A Markdown image whose source resolves to an existing local file (relative paths resolved against the document's directory) SHALL be copied into `word/media/` with a unique name, declared in `[Content_Types].xml`, and referenced from `word/document.xml` as a `w:drawing` sized in EMUs so that images wider than the text column are scaled down to fit while narrower images keep their natural pixel size (at 96 DPI). The image's alt text SHALL be preserved as the drawing's description. Remote (`http(s)`) and data-URI images SHALL keep the existing text fallback.

#### Scenario: Local image is embedded
- **WHEN** a document references `![diagram](images/diagram.png)` and the file exists relative to the document
- **THEN** the package contains `word/media/` with the image bytes, a relationship, a content-type entry for the extension, and a `w:drawing` in the document flow

#### Scenario: Oversized image is scaled to the column
- **WHEN** an embedded image's natural width at 96 DPI exceeds the text column width
- **THEN** the `wp:extent` scales it down proportionally to fit the column

#### Scenario: Missing or remote images keep the text fallback
- **WHEN** an image source is remote, a data URI, or a local path that does not exist
- **THEN** the writer emits the existing `alt: url` text paragraph and the export still succeeds

### Requirement: Built-in DOCX fallback table fidelity
The built-in DOCX writer SHALL render tables with a bold header row marked `w:tblHeader` (repeat as header on page breaks), per-column horizontal alignment (`w:jc`) derived from the parsed Markdown separator row, preserved inline styles within cells, and a table width fitted to the text column with proportional column widths. Raw HTML `<table>` blocks SHALL be parsed into the same table structure rather than flattening each cell into a scattered paragraph.

#### Scenario: Header row repeats and is bold
- **WHEN** a table is exported via the built-in writer
- **THEN** the first `w:tr` carries `w:tblHeader` and its runs carry `w:b`

#### Scenario: Column alignment follows the separator row
- **WHEN** a table declares `|:--|:-:|--:|`
- **THEN** the columns render left/center/right aligned via `w:jc` in their cells

#### Scenario: HTML table keeps its structure
- **WHEN** the document contains a raw `<table>` block
- **THEN** the export contains a real `w:tbl` with the parsed rows and cells instead of one paragraph per cell

### Requirement: Built-in DOCX fallback renders math as OMML
The built-in DOCX writer SHALL convert inline and display math into OMML (`m:oMath` / `m:oMathPara`) so equations open as native editable Word equations. When a formula cannot be converted, the math zone SHALL contain the authored LaTeX source as its text (not a Unicode approximation, and without a `Math: ` prefix), and the export SHALL still succeed.

#### Scenario: Display math becomes a native equation
- **WHEN** a document contains a `$$`-fenced formula
- **THEN** `word/document.xml` contains an `m:oMathPara` representing the formula

#### Scenario: Unconvertible math preserves its source
- **WHEN** a formula uses constructs the converter does not support
- **THEN** the math zone text is the byte-identical authored LaTeX and the export succeeds

### Requirement: Built-in DOCX fallback footnotes, rules, and alerts
The built-in DOCX writer SHALL emit real footnotes: `word/footnotes.xml` carries the definitions and the body carries `w:footnoteReference` marks, so references and definitions stay linked in Word. Horizontal rules SHALL render as a paragraph with a bottom border rather than literal `----------` text. GFM alert blockquotes SHALL render as styled callout paragraphs (accented left border with a bold kind label) rather than `> `-prefixed text.

#### Scenario: Footnote reference links to its definition
- **WHEN** a document contains `[^a]` and its definition
- **THEN** the body carries a `w:footnoteReference w:id` and `word/footnotes.xml` contains the matching `w:footnote`

#### Scenario: Horizontal rule is a border
- **WHEN** the document contains `---` as a thematic break
- **THEN** the export contains a paragraph with `w:pBdr` bottom border and no literal dash text

#### Scenario: GFM alert renders as a callout
- **WHEN** the document contains `> [!WARNING]` with a body
- **THEN** the export renders a bold kind label and the body with callout-style borders/indentation

### Requirement: DOCX package is deflate-compressed
The built-in DOCX writer SHALL compress package entries with deflate instead of storing them uncompressed.

#### Scenario: Package entries are compressed
- **WHEN** any document is exported via the built-in writer
- **THEN** the ZIP local file headers use compression method 8 (deflate) and the package opens correctly in Word
