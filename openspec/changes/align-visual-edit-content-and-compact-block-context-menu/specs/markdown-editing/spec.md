## ADDED Requirements

### Requirement: Visual Edit block chrome SHALL preserve document content geometry
Visual Edit SHALL lay out equivalent top-level rendered content on the same document content axis and with the same available column width as its Read-mode presentation, except for intentional Markdown-semantic indentation or component-internal editing chrome. Block-operation and drag affordances SHALL remain outside normal content flow and SHALL NOT change measured content bounds, row height, or line wrapping when they appear, disappear, gain focus, or become unavailable.

#### Scenario: Top-level prose and media share the document axis
- **WHEN** an unfocused Visual Edit document contains top-level headings, paragraphs, images, formulas, code, or other rendered blocks without semantic indentation
- **THEN** their outer content presentations use the shared document column rather than separate transformable and non-transformable leading gutters
- **AND** equivalent Read-mode content uses the same leading axis and available column width

#### Scenario: Hover and focus chrome do not reflow content
- **WHEN** a transformable block gains or loses hover, caret ownership, menu availability, or drag-grip visibility
- **THEN** the block's content bounds, row height, and wrapped-line breaks remain unchanged solely because of that presentation state
- **AND** showing or hiding the chrome does not change document or derived-cache state

#### Scenario: Semantic indentation remains intentional
- **WHEN** a Visual Edit block is a nested list item, blockquote, source-backed editor, table, image field editor, or another construct with semantic or component-internal indentation
- **THEN** that indentation remains part of the construct's presentation
- **AND** no additional block-operation gutter is added to its normal-flow width

## MODIFIED Requirements

### Requirement: Visual Edit SHALL support exact block transformations and operations
A supported Visual Edit block SHALL expose a compact contextual menu to turn it into Text, Heading 1 through Heading 6, Bulleted List, Numbered List, Task List, Quote, Code Block, Divider, or Table, and to Duplicate or Delete it. Right-clicking an eligible block SHALL target the clicked block without requiring caret ownership or collapsing the current exact text selection; the keyboard context-menu action SHALL target the caret-owning eligible block. The menu SHALL group Text and Heading transforms and List transforms into localized submenus no deeper than one level, identify the current block type, separate destructive Delete from non-destructive actions, and keep every enabled command reachable by pointer and keyboard. The contextual block-operation menu SHALL render in an overlay above all Visual Edit document rows and media, SHALL remain anchored near the invoking pointer or caret within the usable viewport, and SHALL keep every command and submenu reachable when space is constrained. Showing, positioning, navigating, scrolling within, or dismissing the menu SHALL NOT change canonical source, document version, selection, history, dirty state, or derived-cache identity. Each operation SHALL validate current document version, block identity, and exact source ownership; it SHALL perform one canonical source mutation with one undo entry and preserve unrelated bytes, line endings, dirty state, autosave/recovery behavior, tab isolation, and cache invariants.

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

#### Scenario: Block menu overlays later visual content
- **WHEN** the user opens a supported block's operation menu where its bounds overlap following headings, formatted prose, an image, or another Visual Edit row
- **THEN** the complete menu background and commands paint above the overlapping document content
- **AND** underlying document content cannot visually obscure the menu or receive pointer actions within its bounds

#### Scenario: Block menu stays reachable near viewport edges
- **WHEN** the user opens the block-operation menu or one of its submenus with insufficient space below or beside its invoking control
- **THEN** each panel flips or is constrained within the usable viewport
- **AND** overflow commands remain reachable through menu-local scrolling without scrolling the document

#### Scenario: Block menu dismissal is presentation-only
- **WHEN** the user dismisses an open block-operation menu with Escape, an outside action, document scrolling, a tab or mode change, or stale-target invalidation
- **THEN** the menu closes without changing canonical Markdown, document version, selection, history, dirty state, or derived-cache identity

#### Scenario: Right-click targets a non-caret block without collapsing selection
- **WHEN** Visual Edit owns an exact non-empty text selection and the user right-clicks another eligible block
- **THEN** the compact block menu targets the clicked block while preserving the existing canonical selection until a command is invoked
- **AND** a more specific child interaction that consumes the right-click is not replaced by the generic block menu

#### Scenario: Keyboard context action opens an operable block menu
- **WHEN** the caret owns an eligible Visual Edit block and the user invokes the platform keyboard context-menu action
- **THEN** the compact menu opens near the painted caret or a bounded surface fallback and targets that exact block
- **AND** Up, Down, Left, Right, Enter, and Escape navigate, confirm, return from submenus, or dismiss without mutating source before confirmation

#### Scenario: Compact transform groups expose every current block type
- **WHEN** the user opens the Text and Headings or Lists submenu
- **THEN** Text and Heading 1 through Heading 6, or Bulleted, Numbered, and Task List respectively, are reachable with the current type identified
- **AND** Quote, Code Block, Divider, Table, Duplicate, Move Up, Move Down, and separated Delete remain reachable from the root menu according to their availability

### Requirement: Visual Edit SHALL support source-safe block reordering
Supported non-overlapping Visual Edit blocks SHALL be reorderable through Move Up, Move Down, and a hover/focus-only drag grip with before/after drop targets. The drag grip SHALL be positioned outside normal document content flow and SHALL NOT alter content width, row height, or wrapping when shown, hidden, or dragged. All reorder paths SHALL use the same exact source-unit operation, SHALL preserve the moved block bytes and deterministic separator whitespace, and SHALL create one undo entry. Nested list items, quote-group leaves, overlapping ranges, and stale targets SHALL not expose or accept guessed reordering.

#### Scenario: Block moves with button action
- **WHEN** the user invokes Move Down on a supported paragraph before another supported block
- **THEN** the two source units exchange order without altering either block's authored bytes
- **AND** selection follows the moved block and one Undo restores the previous order

#### Scenario: Drag uses the same reorder semantics
- **WHEN** the user drags a supported block grip to a valid before or after target
- **THEN** the same canonical source result is produced as the corresponding button moves
- **AND** drag movement before drop does not mutate source or document version

#### Scenario: Drag grip is flow-neutral
- **WHEN** an eligible block becomes hovered, focused, or actively dragged
- **THEN** its grip appears in the leading interaction area without shifting or narrowing the block content
- **AND** hiding the grip restores no layout because the content geometry never changed

#### Scenario: Unsafe reorder is unavailable
- **WHEN** the focused row is nested, part of an overlapping quote group, or lacks a complete exact source unit
- **THEN** reorder controls are disabled or absent and drops are ignored
- **AND** source mode remains the lossless fallback
