## MODIFIED Requirements

### Requirement: Visual Edit whitespace activation
The system SHALL keep source-backed whitespace ranges available for exact caret mapping. In Visual Edit, a `Whitespace` row SHALL behave as a first-class empty line: it occupies the rendered body paragraph line height (one painted line per covered newline, floored at one line and capped at the existing pathological bound), presents an I-beam pointer, and accepts pointer placement onto an existing offset inside its source range. Clicking a whitespace row SHALL move the caret into that range and MUST NOT insert a newline or otherwise mutate the document text, version, dirty state, undo history, or derived Markdown caches. When the source caret owns a whitespace row — because the user clicked it, pressed Enter onto a new insertion line, or moved into it with keyboard navigation — Visual Edit SHALL present the same empty-paragraph-height layout plus a thin insertion caret line visually consistent with the caret in a paragraph or heading, and SHALL accept subsequent typed text at the exact source caret position. Visual Edit SHALL NOT wrap a whitespace row in a source-island box (border, padding, monospace styling, or differentiated background). Source islands SHALL remain reserved for blocks whose source has no rendered visual form (frontmatter, code, HTML, unsupported constructs) or for inline runs whose source/display mapping is ambiguous. Landing offsets SHALL lie inside the whitespace source range. For a single-newline gap between two rendered blocks, the caret SHALL land at `Whitespace.source_range.start` (the authored separator newline), not the first content byte of the following block.

#### Scenario: Clicking a blank line between headings places the caret without mutation
- **WHEN** the Visual Edit caret belongs to a rendered heading and the user clicks the blank-line `Whitespace` row between that heading and another heading
- **THEN** the caret moves onto an existing offset inside that whitespace range (`source_range.start` for a single-newline gap)
- **AND** the document text, version, dirty state, undo history, and derived Markdown cache identity remain unchanged
- **AND** the gap row presents an insertion caret

#### Scenario: Clicking a blank line between a heading and a paragraph places the caret without mutation
- **WHEN** the Visual Edit caret belongs to a rendered block and the user clicks the blank-line `Whitespace` row between a heading and a paragraph
- **THEN** the caret moves onto an existing offset inside that whitespace range
- **AND** the document text, version, dirty state, undo history, and derived Markdown cache identity remain unchanged
- **AND** the gap row becomes the caret-owning typing surface

#### Scenario: Typing after a gap click inserts at the existing newline
- **WHEN** the user clicks the blank-line row between `## [Unreleased]` and `## [16.1.7]` in a changelog-like document and types text
- **THEN** the typed bytes insert at the existing separator newline so a paragraph appears between the two headings
- **AND** the following heading’s first content byte is not consumed
- **AND** the edit does not insert an extra blank line beyond the newline that was already authored

#### Scenario: Structural Enter activates an insertion line
- **WHEN** the user presses Enter from a heading in Visual Edit and the structural edit creates a new source-backed insertion line
- **THEN** the owning visual row presents the caret and accepts subsequent typed text at the exact source position regardless of whether the parser retains the newline in the heading range

#### Scenario: Intentional source caret movement preserves whitespace editing
- **WHEN** keyboard navigation or reveal logic moves the source caret into an existing whitespace-only range
- **THEN** the owning whitespace row provides the source-backed editing affordance without recomputing the document's cached Markdown-derived state

#### Scenario: Whitespace row owning the caret renders a caret line, not a source island
- **WHEN** the source caret owns a whitespace row in Visual Edit — for example after clicking it, after creating a blank line by pressing Enter, or after pressing Down or Up onto an existing blank line
- **THEN** the row is rendered at empty-paragraph height with a thin insertion caret line and no border, padding, monospace styling, or differentiated background
- **AND** typed text is inserted into the canonical Markdown source at the caret position through the same dirty-state, undo/redo, autosave, and per-tab isolation paths as any other edit

#### Scenario: Whitespace row not owning the caret stays an empty line
- **WHEN** a whitespace row does not own the source caret
- **THEN** it still occupies empty-paragraph height and remains pointer-editable
- **AND** it does not paint an insertion caret until it owns the caret

### Requirement: Layout-aware Visual Edit navigation
When Visual Edit is active, vertical and line-boundary navigation SHALL follow the painted visual layout rather than only logical Markdown source lines. Up/Down and their selection variants SHALL retain a preferred horizontal coordinate across wrapped lines and adjacent visual blocks, while Home/End SHALL target the active painted line in rendered content. Vertical navigation SHALL treat a blank-line (`Whitespace`) row as a navigation stop: moving Up from the lower rendered block and moving Down from the upper rendered block SHALL both land on an existing offset inside the gap row so the user can type into that authored blank line from either direction. A subsequent vertical move SHALL continue into the rendered block on the far side, or walk additional painted lines inside a multi-line whitespace row, while preserving the preferred horizontal coordinate. Leading and trailing blank lines at the document edge SHALL remain reachable the same way instead of becoming a dead no-op.

#### Scenario: Up and Down traverse wrapped visual lines
- **WHEN** a rendered paragraph or other editable visual block wraps onto multiple painted lines
- **AND** the user presses Up or Down
- **THEN** the caret moves to the closest valid source-backed position on the adjacent painted line
- **AND** it does not skip directly to the previous or next logical Markdown line

#### Scenario: Vertical navigation retains preferred horizontal position
- **WHEN** the user presses Up or Down repeatedly across painted lines with different lengths
- **THEN** Visual Edit retains the initial preferred horizontal coordinate
- **AND** each target is the closest valid caret position on that line

#### Scenario: Vertical navigation crosses visual blocks
- **WHEN** Up or Down moves past the first or last painted line of the active visual block
- **THEN** the caret moves to the closest source-backed position in the adjacent visual block
- **AND** a virtualized target row is revealed before the pending movement is completed

#### Scenario: Vertical navigation lands on a blank-line row between content blocks
- **WHEN** the user presses Up from a paragraph whose rendered block above is separated by a blank-line `Whitespace` gap row (for example a heading above, paragraph below)
- **OR** the user presses Down from a heading whose rendered block below is separated by a blank-line gap row
- **THEN** the caret lands on an existing offset inside the gap row (`Whitespace.source_range.start` for a single-newline gap)
- **AND** the gap row becomes the caret-owning row and accepts subsequent typed text at that source position through the standard source-backed input path
- **AND** the resolved target does not land on the start offset of the lower rendered block when that byte is outside the whitespace range
- **AND** the preferred horizontal coordinate is retained across the gap-row crossing

#### Scenario: A second vertical move continues past the gap row
- **WHEN** the caret already owns a blank-line gap row and the user presses Up (or Down) again
- **THEN** the caret moves into the rendered block on the far side of the gap, or onto the next painted line if the whitespace row covers multiple newlines
- **AND** the preferred horizontal coordinate is retained across the crossing

#### Scenario: Up from the start of a paragraph whose line above is a heading
- **WHEN** the caret is at the first source offset of a paragraph (paragraph start) and the user presses Up
- **AND** the block immediately above is a blank-line gap row
- **THEN** the caret moves onto the gap row instead of staying at the paragraph start or jumping into the heading
- **AND** subsequent typed text inserts at the gap row's source position

#### Scenario: A blank line is reachable by arrows as well as click and Enter
- **WHEN** the user wants to type into an existing blank line between two rendered blocks
- **THEN** the caret reaches that `Whitespace` row by clicking it, by pressing Up or Down onto it, or by pressing Enter, and the row becomes the caret-owning row that accepts typed text at its source position
- **AND** a single Up or Down from an adjacent content block parks on the blank line instead of skipping it

#### Scenario: A vertical move reaches a leading or trailing blank line at the document edge
- **WHEN** the only rows beyond the active block in the move direction are blank-line gap rows up to the start or end of the document
- **THEN** the move lands on an existing offset inside the gap row (`Whitespace.source_range.start` for a single-newline gap) instead of becoming a dead no-op

#### Scenario: Selection navigation uses visual targets
- **WHEN** the user invokes Select Up or Select Down in Visual Edit
- **THEN** the selection head uses the same layout-aware target as ordinary vertical movement
- **AND** the canonical source selection remains normalized and UTF-8 safe

#### Scenario: Home and End use the painted line in rendered content
- **WHEN** the Visual Edit caret is in a wrapped rendered line and the user presses Home or End
- **THEN** the caret moves to the first or last valid source-backed position of that painted line
- **AND** explicit source islands retain source-line Home/End behavior
