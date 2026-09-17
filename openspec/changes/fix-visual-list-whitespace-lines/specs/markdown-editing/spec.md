## ADDED Requirements

### Requirement: Visual Edit preserves whitespace-only list tail lines
Visual Edit SHALL preserve individually addressable lines in a list item's trailing whitespace, including lines containing spaces or tabs. Source-to-display mapping SHALL retain their line breaks without moving horizontal whitespace onto the preceding content line. Canonical source bytes SHALL remain unchanged by rendering, and per-version derived caches SHALL remain reusable during caret-only interaction.

#### Scenario: Consecutive Enter after a nested item with whitespace-only following lines
- **WHEN** a user continues a nested list item, exits its empty continuation with Enter, and presses Enter repeatedly before a whitespace-only line containing spaces or tabs
- **THEN** the caret remains below the original item after exit and moves down for every subsequent newline
- **AND** this holds with LF or CRLF, at document end and before subsequent content

#### Scenario: Pointer placement and typing use the same blank-line position
- **WHEN** the user clicks a displayed blank line following a list item and types text
- **THEN** input is inserted at that line's canonical source offset
- **AND** Undo restores the exact source whitespace and caret position

#### Scenario: Inline syntax and blank lines preserve separate ownership
- **WHEN** a list item contains inline formatting or links followed by whitespace-only lines
- **THEN** its syntax remains correctly hidden or revealed and every trailing blank line keeps its own source-backed display position
- **AND** the final separator before the next block does not create a duplicate visual line
