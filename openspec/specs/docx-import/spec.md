# docx-import Specification

## Purpose
Preserve interoperable Markdown semantics when importing supported Word document content, including exact boundaries around strong emphasis.

## Requirements

### Requirement: Imported strong emphasis SHALL have an interoperable right boundary

When DOCX conversion ends bold formatting and the next source-visible character is a Unicode letter or number, the importer SHALL place exactly one U+0020 SPACE after the complete closing Markdown delimiter sequence if no source whitespace already separates the content. The rule SHALL use source run semantics rather than rewriting serialized Markdown, SHALL apply to CJK and Latin letters and digits, and SHALL preserve the authored boundary when the following content starts with whitespace, a line break, punctuation, a Markdown structural delimiter, another non-word symbol, or the end of the paragraph. Content that remains bold across adjacent source runs SHALL NOT receive a synthetic separating space.

#### Scenario: Chinese prose follows a bold run without source whitespace

- **WHEN** a DOCX paragraph contains bold `粗体` immediately followed by non-bold `文字`
- **THEN** the imported Markdown contains `**粗体** 文字`
- **AND** the following text remains outside the bold span

#### Scenario: Latin or numeric prose follows a bold run

- **WHEN** a bold run is immediately followed by a non-bold run whose first visible character is a Latin letter or digit
- **THEN** exactly one ASCII space separates the closing strong-emphasis delimiter sequence from that character

#### Scenario: Punctuation follows a bold run

- **WHEN** a bold run is immediately followed by ASCII or CJK punctuation such as `,`, `.`, `，`, `。`, `：`, or `）`
- **THEN** the importer preserves the adjacent punctuation without inserting a space

#### Scenario: Source separation already exists

- **WHEN** the content after a bold run begins with whitespace or an explicit line break
- **THEN** the importer preserves that separation without adding another space

#### Scenario: Bold formatting continues across source runs

- **WHEN** adjacent DOCX runs both resolve to bold formatting through direct or inherited styles
- **THEN** the importer does not insert a synthetic space between their visible content

#### Scenario: Bold is combined with another inline style

- **WHEN** a bold run also carries italic, underline, strike, superscript, or subscript formatting and is followed immediately by non-bold word-like text
- **THEN** the importer inserts the compatibility space after the complete closing delimiter or safe-HTML sequence
- **AND** the emitted Markdown retains the intended combined formatting and parses the following text outside the bold span

### Requirement: Imported inline formatting SHALL preserve interoperable edge whitespace

When styled DOCX text begins or ends with whitespace, the importer SHALL preserve that whitespace in source order outside the complete Markdown emphasis delimiter or safe-HTML wrapper. The non-whitespace content SHALL retain all resolved inline formatting, interior whitespace SHALL remain unchanged, and whitespace-only content SHALL NOT create empty formatting markers. Moving source edge whitespace SHALL NOT trigger or suppress the existing synthetic bold compatibility space except as determined by the original source run boundaries.

#### Scenario: Bold text has spaces on both run edges

- **WHEN** a DOCX paragraph contains `Before`, bold ` focus `, and `after` in consecutive runs
- **THEN** the imported Markdown contains `Before **focus** after`
- **AND** CommonMark parses `focus` as strong emphasis rather than literal asterisks

#### Scenario: Styled text has whitespace on only one run edge

- **WHEN** bold or italic DOCX text begins or ends with a source space
- **THEN** that space remains outside the corresponding emphasis delimiters
- **AND** the non-whitespace text parses with the authored emphasis

#### Scenario: Combined formatting has edge whitespace

- **WHEN** bold-italic text or text with another supported inline wrapper begins and ends with whitespace
- **THEN** the whitespace is emitted outside the complete combined wrapper
- **AND** the enclosed non-whitespace text retains every authored inline style

#### Scenario: Whitespace-only styled content

- **WHEN** a styled DOCX text segment contains only whitespace
- **THEN** the importer preserves the whitespace without emitting an empty emphasis or safe-HTML wrapper

#### Scenario: Existing bold right-boundary compatibility

- **WHEN** bold text with a leading source space and no trailing source space is followed immediately by word-like non-bold text
- **THEN** the source leading space remains outside the opening delimiter
- **AND** the importer still inserts exactly one synthetic space after the complete closing delimiter sequence

### Requirement: Word imports SHALL preserve the order of text and embedded resources
Supported embedded drawings SHALL appear in authored order among text and other drawings, including multiple drawings in one run. Rectangular table cells SHALL preserve supported links, images, styling and separation between paragraphs. The importer SHALL report content loss when an unsupported cell would otherwise omit recoverable resources or link targets.

#### Scenario: Several drawings share a text run
- **WHEN** a run interleaves text with two embedded drawings
- **THEN** the converted content includes both drawings among the text in source order

#### Scenario: A rectangular cell contains rich content
- **WHEN** a table cell contains paragraphs, styled text, a hyperlink and an embedded image
- **THEN** the rendered table preserves those contents and the paragraph separation
