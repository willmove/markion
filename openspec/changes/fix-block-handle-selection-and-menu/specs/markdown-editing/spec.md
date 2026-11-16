## MODIFIED Requirements

### Requirement: Visual Edit SHALL support exact block transformations and operations
A supported Visual Edit block SHALL expose a compact contextual menu to turn it into Text, Heading 1 through Heading 6, Bulleted List, Numbered List, Task List, Quote, Code Block, Divider, or Table, and to Duplicate or Delete it. Right-clicking an eligible block SHALL target the clicked block without requiring caret ownership or collapsing the current exact text selection; the keyboard context-menu action SHALL target the caret-owning eligible block. These explicit context menus SHALL group Text and Heading transforms and List transforms into localized submenus no deeper than one level. A grip-invoked popup SHALL expose the same transforms as compact direct controls as specified by the block-grip quick-settings requirement. All presentations SHALL identify the current block type, separate destructive Delete from non-destructive actions, and keep every enabled command reachable by pointer and keyboard. The contextual block-operation menu SHALL render in an overlay above all Visual Edit document rows and media, SHALL remain anchored near the invoking pointer or caret within the usable viewport, and SHALL keep every command and submenu reachable when space is constrained. Showing, positioning, navigating, scrolling within, or dismissing the menu SHALL NOT change canonical source, document version, selection, history, dirty state, or derived-cache identity; a grip click's explicit block selection occurs before opening its popup. Each operation SHALL validate current document version, block identity, and exact source ownership; each mutating operation SHALL perform one canonical source mutation with one undo entry and preserve unrelated bytes, line endings, dirty state, autosave/recovery behavior, tab isolation, and cache invariants.

#### Scenario: Heading turns into a task item
- **WHEN** the user transforms an exactly mapped heading into a Task List block
- **THEN** only the proven structural marker is replaced with canonical unchecked-task Markdown
- **AND** inline authored content and UTF-8 text remain byte-identical

#### Scenario: Code block turns into text
- **WHEN** the user transforms a closed exactly mapped fenced code block to Text
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
- **WHEN** the user opens the Text and Headings or Lists submenu of an explicit context menu
- **THEN** Text and Heading 1 through Heading 6, or Bulleted, Numbered, and Task List respectively, are reachable with the current type identified
- **AND** Quote, Code Block, Divider, Table, Duplicate, Move Up, Move Down, and separated Delete remain reachable from the root menu according to their availability

## ADDED Requirements

### Requirement: Exclusive block-selection ends SHALL leave the following block inactive

A selected block range SHALL own complete canonical source units and their deterministic separators, excluding the next unselected block's structural prefix. A non-empty selection whose exclusive end equals the next block's start SHALL NOT reveal or highlight that next block's heading, list or quote markers or their following spaces solely because of that endpoint. The selected block's own markers and descendants SHALL remain part of its complete source unit for copy, delete and move. Collapsed caret editing, selection that actually intersects a marker, and IME composition SHALL retain their documented syntax-reveal behavior. Selecting or projecting a range SHALL NOT mutate source, version, history or shared derived caches.

#### Scenario: Paragraph before a heading
- **WHEN** a paragraph immediately before an H2 or H3 is selected through its grip
- **THEN** the selection ends before the following heading's `## ` or `### ` prefix
- **AND** that prefix stays hidden and unhighlighted until an independent interaction targets it

#### Scenario: Adjacent headings retain separate ownership
- **WHEN** the user selects the first of two neighboring headings
- **THEN** its own authored heading prefix remains part of the selected canonical source unit
- **AND** the following heading's prefix is excluded and is not revealed by the exclusive selection end

#### Scenario: Extended and reversed selections stop at the same boundary
- **WHEN** the user extends or reverses a multi-block selection ending immediately before an unselected heading, list or quote
- **THEN** every selected complete unit is included and the following structural prefix remains inactive
- **AND** LF/CRLF separators and multibyte text retain exact UTF-8-safe ownership

#### Scenario: A real caret or intersecting selection still reveals syntax
- **WHEN** a collapsed caret enters a heading prefix or its first content position, a non-empty selection actually overlaps its prefix, or an IME marked range intersects it
- **THEN** the corresponding prefix is revealed according to the existing editing behavior
- **AND** an empty heading or list item remains directly editable

### Requirement: Block grips SHALL provide optional compact quick settings

When automatic grip-click popups are enabled, completing an ordinary non-drag click on an eligible Visual Edit block grip SHALL select that single complete source unit and open a nearby compact quick-settings popup. It SHALL offer direct Text, H1-H6, bulleted/numbered/task list, quote, code, divider and table controls, the existing Duplicate, Move Up, Move Down and separated Delete operations, plus Copy, Cut and Add Below. It SHALL identify the current type, use localized labels or tooltips and active-theme colors, and keep disabled operations visibly unavailable. Every enabled command SHALL be reachable by pointer and keyboard. Shift-click SHALL extend a block range without opening the popup. Dragging SHALL retain complete selected-unit reordering and SHALL NOT open a click popup. Disabled automatic popups SHALL retain selection-only clicks. Opening/dismissing the popup after selection SHALL leave source, version, history, dirty state, the established selection and derived caches unchanged.

#### Scenario: Completed click opens nearby quick settings
- **WHEN** automatic popups are enabled and the user clicks and releases an eligible block grip without dragging or extending selection
- **THEN** only that complete block unit is selected and a compact popup opens near the grip within the viewport
- **AND** its enabled type controls and editing actions are reachable without being obscured by later document content

#### Scenario: Shift and drag remain selection gestures
- **WHEN** the user Shift-clicks another block grip or drags an already selected block range
- **THEN** the range extends or moves through the existing structured-selection/reordering behavior
- **AND** no automatic click popup opens during or after the gesture

#### Scenario: Copy and Cut target the clicked block
- **WHEN** the user invokes Copy or Cut from a valid grip popup
- **THEN** portable plain-text and HTML clipboard representations describe that complete source unit without content or markers from the following block
- **AND** Copy leaves the document unchanged, while Cut removes that exact unit in one checked edit and one undo step

#### Scenario: Add Below preserves the next heading
- **WHEN** the user invokes Add Below on a block immediately before a heading
- **THEN** one editable blank paragraph is inserted below the complete target unit and the caret moves there
- **AND** the following heading's prefix, unrelated source and LF/CRLF style are preserved
- **AND** one Undo restores the exact source and prior selection

#### Scenario: Dismissal and stale targets are safe
- **WHEN** the user dismisses a grip popup or its captured document version or block identity becomes stale
- **THEN** it closes without guessing a source mutation or changing the established selection merely to dismiss it
- **AND** explicit right-click and keyboard context-menu entry points remain usable
