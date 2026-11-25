## ADDED Requirements

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
