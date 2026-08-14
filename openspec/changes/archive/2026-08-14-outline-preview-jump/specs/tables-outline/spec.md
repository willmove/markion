## MODIFIED Requirements

### Requirement: Document outline navigation
The editor SHALL provide a toggleable outline panel that lists the document's heading hierarchy, supports context-aware click-to-jump navigation, highlights the heading for the section containing the canonical cursor, and updates as headings change. In Read mode, clicking an outline heading SHALL move the canonical cursor to that heading's source position, highlight that outline item, and bring the corresponding rendered heading into view in the preview pane. In Edit, Visual Edit, and Split Preview modes, outline clicks SHALL retain their existing editable-surface source-position navigation. The outline SHALL render compact rows with no extra inter-row margin and no more than 2px total vertical padding for a single-line row. When the heading list exceeds the visible panel height, the outline SHALL scroll vertically so every heading remains reachable by mouse-wheel or trackpad input. The outline is a flat indented list; collapse/expand of subsections is **not** supported.

#### Scenario: Outline lists headings and tracks the document
- **WHEN** the outline panel is visible
- **THEN** it lists all headings with their nesting indentation and updates when headings are added, removed, or changed

#### Scenario: Click to jump outside Read mode
- **WHEN** the user clicks a heading in the outline while Edit, Visual Edit, or Split Preview mode is active
- **THEN** the active editable surface navigates to that heading's source position as it did before this change

#### Scenario: Click to jump in Read mode
- **WHEN** the user clicks a heading in the outline while Read mode is active
- **THEN** the preview pane brings the rendered heading for that outline item into view
- **AND** the canonical cursor moves to the clicked heading's source position
- **AND** the clicked heading becomes the active outline item

#### Scenario: Read-mode navigation is non-mutating
- **WHEN** the user navigates through the outline while Read mode is active
- **THEN** the document text, version, dirty state, and undo/redo history remain unchanged

#### Scenario: Active section highlight
- **WHEN** the canonical cursor is within a given section
- **THEN** the outline highlights the heading corresponding to that section

#### Scenario: Outline rows use compact vertical spacing
- **WHEN** the outline contains consecutive single-line headings
- **THEN** each row has no extra inter-row margin and no more than 2px total vertical padding
- **AND** hierarchy indentation, readable labels, hover feedback, active highlighting, and the row click target remain intact

#### Scenario: Overflowing outline is vertically scrollable
- **WHEN** the document has more outline headings than fit in the visible sidebar height
- **THEN** mouse-wheel or trackpad input over the outline scrolls its heading list vertically
- **AND** every heading can be brought into view and activated
