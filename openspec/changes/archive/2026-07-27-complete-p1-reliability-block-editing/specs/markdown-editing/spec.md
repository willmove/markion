## ADDED Requirements

### Requirement: Visual Edit SHALL provide a complete slash-command block palette
When a collapsed Visual Edit caret is on a line containing only optional indentation and a slash query, Markion SHALL show a localized, filtered command palette. The palette SHALL provide Text, Heading 1 through Heading 6, Bulleted List, Numbered List, Task List, Quote, Code Block, Divider, and Table commands. Up and Down SHALL change the active result, Enter SHALL apply it, Escape SHALL close the palette without changing source, and pointer selection SHALL apply the same command. Confirmation SHALL replace the slash query through one canonical UTF-8-safe source edit.

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

### Requirement: Visual Edit SHALL support exact block transformations and operations
A supported focused Visual Edit block SHALL expose contextual operations to turn it into Text, Heading 1 through Heading 6, Bulleted List, Numbered List, Task List, Quote, or Code Block, and to Duplicate or Delete it. Each operation SHALL validate current document version, block identity, and exact source ownership; it SHALL perform one canonical source mutation with one undo entry and preserve unrelated bytes, line endings, dirty state, autosave/recovery behavior, tab isolation, and cache invariants.

#### Scenario: Heading turns into a task item
- **WHEN** the user transforms an exactly mapped heading into a Task List block
- **THEN** only the proven structural marker is replaced with canonical unchecked-task Markdown
- **AND** inline authored content and UTF-8 text remain byte-identical

#### Scenario: Code block turns into text
- **WHEN** a closed exactly mapped fenced code block is transformed to Text
- **THEN** its payload becomes paragraph source and the fence metadata is removed in one edit
- **AND** an unclosed or ambiguous fence is not transformed speculatively

#### Scenario: Duplicate and delete are atomic
- **WHEN** the user duplicates or deletes a supported block
- **THEN** the complete exact block source and deterministic separator whitespace are duplicated or removed
- **AND** one Undo restores the prior source and selection

#### Scenario: Stale or ambiguous transform is rejected
- **WHEN** a block event carries a stale version/identity/range or the source ownership overlaps an ambiguous nested structure
- **THEN** no source, history, document version, or cache identity changes
- **AND** complete source editing remains available

### Requirement: Visual Edit SHALL support source-safe block reordering
Supported non-overlapping Visual Edit blocks SHALL be reorderable through Move Up, Move Down, and a drag grip with before/after drop targets. All reorder paths SHALL use the same exact source-unit operation, SHALL preserve the moved block bytes and deterministic separator whitespace, and SHALL create one undo entry. Nested list items, quote-group leaves, overlapping ranges, and stale targets SHALL not expose or accept guessed reordering.

#### Scenario: Block moves with button action
- **WHEN** the user invokes Move Down on a supported paragraph before another supported block
- **THEN** the two source units exchange order without altering either block's authored bytes
- **AND** selection follows the moved block and one Undo restores the previous order

#### Scenario: Drag uses the same reorder semantics
- **WHEN** the user drags a supported block grip to a valid before or after target
- **THEN** the same canonical source result is produced as the corresponding button moves
- **AND** drag movement before drop does not mutate source or document version

#### Scenario: Unsafe reorder is unavailable
- **WHEN** the focused row is nested, part of an overlapping quote group, or lacks a complete exact source unit
- **THEN** reorder controls are disabled or absent and drops are ignored
- **AND** source mode remains the lossless fallback
