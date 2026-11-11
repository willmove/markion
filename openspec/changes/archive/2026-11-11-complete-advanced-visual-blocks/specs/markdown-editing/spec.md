## MODIFIED Requirements

### Requirement: Visual Edit SHALL support source-safe block reordering
Supported non-overlapping Visual Edit source units SHALL be reorderable through Move Up, Move Down, and a hover/focus-only drag grip with before/after drop targets. A complete list parent SHALL carry its nested descendants; a quote group and a contiguous block selection SHALL move as complete source units. The drag grip SHALL be positioned outside normal document content flow and SHALL NOT alter content width, row height, or wrapping when shown, hidden, or dragged. All reorder paths SHALL use the same exact source-unit operation, SHALL preserve moved source bytes and deterministic separator whitespace, and SHALL create one undo entry. Drops inside the moved subtree, overlapping ownership and stale targets SHALL be rejected. Safe list indentation changes SHALL apply to the complete selected subtree.

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

#### Scenario: Parent list item moves with descendants
- **WHEN** a list parent with nested items or nested code is moved
- **THEN** its complete subtree moves without leaving descendants behind or changing their internal indentation

#### Scenario: Unsafe reorder is unavailable
- **WHEN** the target lacks a complete exact source unit, is inside the moving subtree, or is stale
- **THEN** reorder controls are disabled or the drop is ignored
- **AND** source mode remains the lossless fallback

## ADDED Requirements

### Requirement: Exact container splitting preserves surrounding content
Turning an exactly mapped quoted leaf into ordinary text SHALL remove only its own quote prefixes, separate preceding and following quoted content into valid containers, and preserve unrelated authored text and line endings. Complete nested list indentation/outdent commands SHALL update every owned source line together. Each successful operation SHALL be one checked undoable edit; ambiguous or stale mappings SHALL remain unchanged.

#### Scenario: Extract a middle quote paragraph
- **WHEN** the user turns the middle paragraph of a multi-paragraph quote into text
- **THEN** the preceding quote, unquoted paragraph and following quote are separate containers with their original payloads
- **AND** one Undo restores the exact original quote source

#### Scenario: Outdent a list subtree
- **WHEN** the user outdents a nested list parent with descendants
- **THEN** the whole subtree loses one legal indentation level and retains its internal nesting
