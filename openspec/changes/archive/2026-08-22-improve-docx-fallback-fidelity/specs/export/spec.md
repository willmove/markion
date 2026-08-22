## ADDED Requirements

### Requirement: Built-in DOCX fallback package completeness
The built-in DOCX writer SHALL emit a structurally complete OOXML package in which every style, numbering, and relationship reference resolves. The package SHALL include `word/styles.xml` (with `docDefaults` and paragraph/character styles for Normal, Title, Heading1 through Heading6, Quote, and code), `word/numbering.xml`, `word/theme/theme1.xml`, `word/settings.xml`, and `word/fontTable.xml`, in addition to the existing content-types, relationships, core-properties, and document parts. Headings at every Markdown level (H1–H6) SHALL map to their own distinct heading style.

#### Scenario: All style references resolve
- **WHEN** a document containing headings H1–H6 is exported via the built-in DOCX writer
- **THEN** the package contains `word/styles.xml` defining Heading1 through Heading6 and Normal
- **AND** every `w:pStyle` value used in `word/document.xml` is defined in `word/styles.xml`

#### Scenario: Six distinct heading levels
- **WHEN** a document contains headings from `#` through `######`
- **THEN** `word/document.xml` references six distinct styles Heading1–Heading6 with no collapse of deeper levels

#### Scenario: Front matter title uses a Title style
- **WHEN** the document front matter provides a title
- **THEN** the exported document opens with a Title-styled paragraph and the title is still written to `docProps/core.xml`

### Requirement: Built-in DOCX fallback inline fidelity
The built-in DOCX writer SHALL preserve resolved inline styling by consuming the rich-text spans already computed for preview. Bold, italic, strikethrough, highlight, superscript, subscript, and inline code SHALL map to the corresponding `w:rPr` properties (with `w:highlight` for highlight and `w:vertAlign` for super/subscript), and links SHALL be emitted as real `w:hyperlink` elements backed by external relationships in `word/_rels/document.xml.rels` so the target URL survives. Inline code SHALL use a monospace font declaration.

#### Scenario: Inline styles are preserved
- **WHEN** a paragraph contains bold, italic, strikethrough, highlight, superscript, subscript, and inline code spans
- **THEN** `word/document.xml` carries `w:b`, `w:i`, `w:strike`, `w:highlight`, `w:vertAlign` (superscript/subscript), and a monospace `w:rFonts` run respectively, rather than flattening to plain text

#### Scenario: Links keep their targets
- **WHEN** a paragraph contains `[label](https://example.com)`
- **THEN** the document contains a `w:hyperlink` for `label` whose relationship in `word/_rels/document.xml.rels` targets `https://example.com`

#### Scenario: Nested inline styles compose
- **WHEN** a span carries multiple styles (e.g. bold italic)
- **THEN** the run carries all corresponding `w:rPr` properties simultaneously

### Requirement: Built-in DOCX fallback structural lists
The built-in DOCX writer SHALL render unordered, ordered, and task lists as real Word list paragraphs. `word/numbering.xml` SHALL define abstract bullet and decimal numbering formats; list items SHALL reference them via `w:numPr` with `w:ilvl` reflecting the item's nesting depth, so nested lists keep their hierarchy and remain editable in Word. Task-list items SHALL keep a checked/unchecked marker prefix until Word checkbox content controls are adopted.

#### Scenario: Nested lists keep depth
- **WHEN** a document contains a bullet list nested two levels deep
- **THEN** items at each depth reference the numbering definition with the matching `w:ilvl` and render with increasing indentation in Word

#### Scenario: Ordered lists are auto-numbered
- **WHEN** a document contains an ordered list
- **THEN** items use a decimal `w:numFmt` and no literal `1. ` text is emitted for the marker

### Requirement: Built-in DOCX fallback typography and page setup
The built-in DOCX writer SHALL declare east-asian fonts for CJK text. `docDefaults` and the heading styles SHALL carry `w:rFonts` with an `w:eastAsia` attribute (body, heading, and code styles each with a sensible default), so Chinese text renders with a controlled font instead of Word fallback heuristics. The default page setup SHALL be A4 with 2.54 cm margins instead of hard-coded US Letter.

#### Scenario: CJK fonts are declared
- **WHEN** any document is exported via the built-in DOCX writer
- **THEN** `word/styles.xml` `docDefaults` and heading styles carry `w:rFonts` `w:eastAsia` declarations

#### Scenario: Default page is A4
- **WHEN** a document is exported with default options
- **THEN** `w:sectPr` in `word/document.xml` specifies A4 dimensions (11906×16838 twips) and 1440-twip margins
