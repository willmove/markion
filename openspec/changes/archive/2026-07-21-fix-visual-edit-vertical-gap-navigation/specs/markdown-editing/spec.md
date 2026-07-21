## MODIFIED Requirements

### Requirement: Layout-aware Visual Edit navigation
When Visual Edit is active, vertical and line-boundary navigation SHALL follow the painted visual layout rather than only logical Markdown source lines. Up/Down and their selection variants SHALL retain a preferred horizontal coordinate across wrapped lines and adjacent visual blocks, while Home/End SHALL target the active painted line in rendered content. Vertical navigation SHALL be symmetric across blank-line (`Whitespace`) gap rows: moving Up from the lower rendered block and moving Down from the upper rendered block SHALL both land on the gap row's source offset so the user can type into an existing blank line from either direction, then a subsequent vertical move SHALL continue into the rendered block on the far side while preserving the preferred horizontal coordinate.

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

#### Scenario: Vertical navigation is symmetric across a blank-line gap row
- **WHEN** the user presses Up from a paragraph whose rendered block above is separated by a blank-line `Whitespace` gap row (for example a heading above, paragraph below)
- **OR** the user presses Down from a heading whose rendered block below is separated by a blank-line gap row
- **THEN** the caret lands on the gap row's source offset (`Whitespace.source_range.start`), which is the same source offset the gap row's `VisualProjection` anchors at
- **AND** the gap row becomes the caret-owning row and accepts subsequent typed text at that source position through the standard source-backed input path
- **AND** the resolved target does not land on the start offset of the lower rendered block, which would otherwise look like the caret did not move

#### Scenario: A second vertical move continues past the gap row
- **WHEN** the caret already owns a blank-line gap row and the user presses Up (or Down) again
- **THEN** the caret moves into the rendered block on the far side of the gap
- **AND** the preferred horizontal coordinate is retained across the gap-row crossing

#### Scenario: Up from the start of a paragraph whose line above is a heading
- **WHEN** the caret is at the first source offset of a paragraph (paragraph start) and the user presses Up
- **AND** the block immediately above is a blank-line gap row
- **THEN** the caret moves onto the gap row instead of staying at the paragraph start
- **AND** subsequent typed text inserts at the gap row's source position

#### Scenario: Selection navigation uses visual targets
- **WHEN** the user invokes Select Up or Select Down in Visual Edit
- **THEN** the selection head uses the same layout-aware target as ordinary vertical movement
- **AND** the canonical source selection remains normalized and UTF-8 safe

#### Scenario: Home and End use the painted line in rendered content
- **WHEN** the Visual Edit caret is in a wrapped rendered line and the user presses Home or End
- **THEN** the caret moves to the first or last valid source-backed position of that painted line
- **AND** explicit source islands retain source-line Home/End behavior
