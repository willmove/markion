## ADDED Requirements

### Requirement: Heading bookmarks SHALL become Markdown heading identifiers
The importer SHALL represent the first supported bookmark on a nonempty heading as the heading's authored identifier. Additional bookmarks and bookmarks outside headings SHALL retain standalone targets. Targets and internal hyperlinks SHALL use the same unique identifiers.

#### Scenario: A heading contains a bookmark
- **WHEN** an imported heading contains one bookmark
- **THEN** the bookmark becomes the heading identifier
- **AND** Visual Edit renders the heading without a standalone bookmark row

### Requirement: Word emphasis SHALL survive run fragmentation
Adjacent text with the same effective formatting SHALL retain continuous styling across Word run boundaries. Where adjacent formatting or punctuation would make Markdown emphasis delimiters ambiguous, the importer SHALL preserve the intended styling and source spacing with supported inline markup.

#### Scenario: Bold Chinese text surrounds a separately stored Latin word
- **WHEN** consecutive bold Word runs contain Chinese text, a space-delimited Latin word, and more Chinese text
- **THEN** the resulting Markdown renders the entire phrase in bold with the authored spaces

### Requirement: Supported Word equations SHALL retain their mathematical structure
The importer SHALL emit editable inline or display LaTeX for supported Office Math expressions, preserving subscripts, superscripts, fractions, roots, delimiters, n-ary operators, limits, accents, mathematical alphabet styles, and textual annotations. Unsupported structures SHALL produce the existing explicit content-loss diagnostic and readable fallback.

#### Scenario: A display equation combines structured notation
- **WHEN** a Word display equation contains a sum with a lower limit, a bracketed expression, and an accented variable
- **THEN** Markion can typeset the imported expression with the authored operators, brackets, limit, and accent

### Requirement: Word structure and endnote diagnostics SHALL reflect authored content
The importer SHALL preserve built-in Word quote paragraph styles as Markdown blockquotes and standalone Word horizontal rules as thematic breaks. A rectangular Word table column whose paragraphs share an explicit center or right alignment SHALL retain it in the GFM table. An endnotes relationship without an authored endnote reference SHALL NOT generate a content-loss diagnostic.

#### Scenario: A body paragraph uses Word's Block Text style
- **WHEN** a paragraph uses the built-in Block Text style or inherits it
- **THEN** the imported document renders the paragraph as a blockquote

#### Scenario: A standalone Word horizontal rule separates sections
- **WHEN** a paragraph contains a Word horizontal-rule shape
- **THEN** the imported document renders a Markdown thematic break

#### Scenario: A table column is consistently centered
- **WHEN** the header and body paragraphs of a rectangular Word table column are centered
- **THEN** the resulting GFM column uses center alignment

#### Scenario: Word packages only endnote separator metadata
- **WHEN** a DOCX includes an endnotes part and no endnote references
- **THEN** the import report does not claim endnote content was lost
