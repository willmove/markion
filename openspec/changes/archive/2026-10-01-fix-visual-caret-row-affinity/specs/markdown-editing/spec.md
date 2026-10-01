## ADDED Requirements

### Requirement: Visual caret and blank rows agree with source lines
Visual Edit SHALL paint the caret on the source-backed line that receives subsequent input. A source position before an authored line ending SHALL remain on the preceding content row; a position after that ending SHALL belong to the following line. Blank-row pointer placement and vertical navigation SHALL resolve to positions on their displayed source lines, including EOF, LF, and CRLF. Interaction alone SHALL preserve document text, version, undo history, and derived caches.

#### Scenario: Bare heading before a terminal newline
- **WHEN** the source caret is immediately after `#` in a heading followed by a newline
- **THEN** the visual caret is beside the revealed marker on the same line
- **AND** typing or switching to Source mode agrees with that position

#### Scenario: Clicking below a heading or paragraph
- **WHEN** the user clicks a terminal blank line below a heading or paragraph
- **THEN** the caret resolves after the preceding line ending
- **AND** typing inserts on the clicked line without appending to the preceding content

#### Scenario: Distinct consecutive blank lines
- **WHEN** a document has consecutive blank lines, including whitespace-only lines and CRLF endings
- **THEN** each displayed blank line has a distinct valid source-line target
- **AND** Up/Down navigation, pointer placement, and the caret share that mapping

#### Scenario: Structural and formatted neighboring rows
- **WHEN** the caret crosses boundaries of headings, paragraphs with inline formatting, lists, or quotes
- **THEN** its painted line agrees with the line receiving Unicode typing or IME composition
- **AND** undo restores the exact original source
