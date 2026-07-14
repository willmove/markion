## MODIFIED Requirements

### Requirement: Document outline navigation
The editor SHALL provide a toggleable outline panel that lists the document's heading hierarchy, supports context-aware click-to-jump navigation, highlights the heading for the section containing the cursor, and updates as headings change. In Edit mode, outline clicks SHALL jump to the heading's source position. In Read mode, where the editor pane is hidden, outline clicks SHALL scroll the preview pane to the corresponding rendered heading content. The outline is a flat indented list; collapse/expand of subsections is **not** supported.

#### Scenario: Outline lists headings and tracks the document
- **WHEN** the outline panel is visible
- **THEN** it lists all headings with their nesting indentation and updates when headings are added, removed, or changed

#### Scenario: Click to jump in Edit mode
- **WHEN** the user clicks a heading in the outline while Edit mode is active
- **THEN** the editor scrolls to that heading's source position

#### Scenario: Click to jump in Read mode
- **WHEN** the user clicks a heading in the outline while Read mode is active
- **THEN** the preview pane scrolls to the rendered heading content for that outline item

#### Scenario: Active section highlight
- **WHEN** the cursor is within a given section
- **THEN** the outline highlights the heading corresponding to that section
