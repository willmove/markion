## MODIFIED Requirements

### Requirement: Document outline navigation
The editor SHALL provide a toggleable outline panel that lists the document's heading hierarchy, supports click-to-jump navigation, highlights the heading for the section containing the cursor, and updates as headings change. The outline SHALL render compact rows with no extra inter-row margin and no more than 2px total vertical padding for a single-line row. When the heading list exceeds the visible panel height, the outline SHALL scroll vertically so every heading remains reachable by mouse-wheel or trackpad input. The outline is a flat indented list; collapse/expand of subsections is **not** supported.

#### Scenario: Outline lists headings and tracks the document
- **WHEN** the outline panel is visible
- **THEN** it lists all headings with their nesting indentation and updates when headings are added, removed, or changed

#### Scenario: Click to jump
- **WHEN** the user clicks a heading in the outline
- **THEN** the editor scrolls to that heading's source position

#### Scenario: Active section highlight
- **WHEN** the cursor is within a given section
- **THEN** the outline highlights the heading corresponding to that section

#### Scenario: Outline rows use compact vertical spacing
- **WHEN** the outline contains consecutive single-line headings
- **THEN** each row has no extra inter-row margin and no more than 2px total vertical padding
- **AND** hierarchy indentation, readable labels, hover feedback, active highlighting, and the row click target remain intact

#### Scenario: Overflowing outline is vertically scrollable
- **WHEN** the document has more outline headings than fit in the visible sidebar height
- **THEN** mouse-wheel or trackpad input over the outline scrolls its heading list vertically
- **AND** every heading can be brought into view and activated
