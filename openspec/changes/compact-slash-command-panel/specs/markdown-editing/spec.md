## MODIFIED Requirements

### Requirement: Visual Edit SHALL provide a complete slash-command block palette
When a collapsed Visual Edit caret follows a valid slash query in an eligible editable text block or blank line, Markion SHALL show a localized, filtered command palette. The palette SHALL provide Text, Heading 1 through Heading 5, Bulleted List, Numbered List, Task List, Quote, Code Block, Divider, Table, Link, Formula, and Image commands. Heading 6 SHALL NOT appear in slash completion, including filtered results; other heading-formatting entry points SHALL retain their existing supported levels. Formatting commands SHALL appear as compact buttons in a grid with localized tooltips and an active-command caption. Button spacing SHALL be tighter than the initial compact palette. Table, Link, Formula, and Image SHALL appear as separated labeled insertion actions. The palette SHALL use the active theme and remain anchored near the source without inserting document layout space. Up and Down SHALL change the active result, Enter SHALL apply it, Escape SHALL close the palette without changing source, and pointer selection SHALL apply the same command. Confirmation SHALL replace only the slash marker and query through one canonical UTF-8-safe source edit, preserving following paragraph content and line endings. Standalone block templates SHALL separate remaining same-line content from their closing delimiter.

#### Scenario: Slash query filters commands
- **WHEN** the user types `/hea` on an otherwise empty Visual Edit block
- **THEN** the palette shows the matching localized heading commands
- **AND** typing or navigating the palette does not create a parallel document value

#### Scenario: Keyboard confirmation is one edit
- **WHEN** the user selects Heading 2 with the keyboard and presses Enter
- **THEN** the slash query becomes an H2 Markdown block with the caret at its editable content position
- **AND** one Undo restores the exact slash query and selection

#### Scenario: Escape preserves canonical source
- **WHEN** the slash palette is open and the user presses Escape
- **THEN** the palette closes without changing document version, source, selection, history, or derived-cache identity

#### Scenario: Stale slash target is rejected
- **WHEN** the document version or query range changes before a palette command is confirmed
- **THEN** the palette closes without guessing a mutation

#### Scenario: Compact pointer controls retain every command
- **WHEN** an unfiltered slash palette opens
- **THEN** every listed palette action is reachable through its compact button or labeled insertion action
- **AND** pointer confirmation uses the same canonical edit as keyboard confirmation

#### Scenario: Slash before existing paragraph content
- **WHEN** the user types `/` at the start of a paragraph containing spaces, CJK text, or inline Markdown after the caret
- **THEN** the palette opens with an empty query
- **AND** selecting Heading 2 preserves that existing content after the new heading prefix
- **AND** one Undo restores the exact slash, existing text, and caret

#### Scenario: Slash opens insertion settings without editing
- **WHEN** the user chooses Link, Formula, or Image from slash completion
- **THEN** the corresponding settings editor opens against the marker/query range
- **AND** source, selection, version, undo history, dirty state, and derived caches are unchanged
- **AND** canceling leaves the original slash query intact without immediately reopening completion

#### Scenario: Confirm insertion settings once
- **WHEN** the user enters a link label/URL/title, a LaTeX formula, or image settings and confirms
- **THEN** the confirmed content replaces only the captured marker/query in one undoable edit
- **AND** existing paragraph content and authored line endings remain intact
- **AND** a stale captured document version is rejected

#### Scenario: Local image file uses the slash insertion target
- **WHEN** the user chooses a local image file from slash Image settings
- **THEN** the existing configured resource policy imports or references it at the captured query range
- **AND** canceling the file chooser does not delete the query or modify source

### Requirement: Slash trigger recognizes ASCII and Chinese markers
Slash-command recognition SHALL accept `/` or `、` after line start or whitespace, require a whitespace-free query bounded by the caret, and reject URL-like or otherwise non-command prefixes and path syntax in the query. Content after the caret and LF or CRLF line endings SHALL NOT participate in query validation. Eligible Visual Edit contexts SHALL include editable prose, heading, list, quote, and blank-line blocks. Code, diagram, math, table-cell, HTML, image-source, front-matter, and other structured source fields SHALL retain literal slash input without opening this palette. Recognition SHALL remain disabled during IME composition and non-empty selections. Confirmation SHALL replace the complete marker and query through the existing exact block edit path.

#### Scenario: Chinese marker opens an equivalent query
- **WHEN** the user types `、hea` after line start or whitespace in Visual Edit
- **THEN** the same filtered heading commands and stale-target validation apply as for `/hea`

#### Scenario: URL is not a slash query
- **WHEN** the caret follows a URL-like prefix containing `://`
- **THEN** no slash query is created

#### Scenario: Windows line endings do not block commands
- **WHEN** the user types `/` or `、h2` on an eligible line terminated by CRLF
- **THEN** the palette opens and filters just as it does with LF
- **AND** confirmation preserves the CRLF bytes

#### Scenario: Structured payload keeps literal slash input
- **WHEN** the user types `/` at the beginning of a code, math, diagram, HTML, image, front-matter, or table-cell source field
- **THEN** the slash remains literal and the block palette stays closed
