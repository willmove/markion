## ADDED Requirements

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
