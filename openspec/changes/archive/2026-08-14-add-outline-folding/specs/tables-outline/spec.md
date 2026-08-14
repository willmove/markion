## MODIFIED Requirements

### Requirement: Document outline navigation
The editor SHALL provide a toggleable outline panel that lists the document's heading hierarchy, supports context-aware click-to-jump navigation, highlights the heading for the section containing the canonical cursor, and updates as headings change. In Read mode, clicking an outline heading label SHALL move the canonical cursor to that heading's source position, highlight that outline item, and bring the corresponding rendered heading into view in the preview pane. In Edit, Visual Edit, and Split Preview modes, heading-label clicks SHALL retain their existing editable-surface source-position navigation.

The outline SHALL present the heading hierarchy as an indented, collapsible tree. A heading that owns one or more following headings at deeper levels before the next heading at its own or a shallower level SHALL expose a disclosure control. A newly opened document outline SHALL start fully expanded. Activating a disclosure control SHALL collapse or expand that heading's descendant rows without invoking heading navigation; re-expanding an ancestor SHALL preserve any independently collapsed nested sections. Folding state SHALL remain isolated per open document and session-only.

The outline SHALL render compact rows with no extra inter-row margin and no more than 2px total vertical padding for a single-line row. When the visible heading list exceeds the panel height, the outline SHALL scroll vertically so every currently visible heading remains reachable by mouse-wheel or trackpad input. Folding SHALL affect presentation only and MUST NOT mutate Markdown, document version, dirty state, selection, or undo/redo history, and MUST NOT require recomputing the document's derived outline for an unchanged document version.

#### Scenario: Outline lists headings and tracks the document
- **WHEN** the outline panel is visible
- **THEN** it lists the current document's headings with hierarchy indentation and updates when headings are added, removed, or changed
- **AND** obsolete folding identities do not hide unrelated headings after the hierarchy changes

#### Scenario: Outline starts fully expanded
- **WHEN** a document is newly opened or created and its outline is shown
- **THEN** every heading is visible regardless of depth
- **AND** every heading with descendants shows an expanded disclosure state

#### Scenario: Collapse a heading subtree
- **WHEN** the user activates the expanded disclosure control for a heading with descendants
- **THEN** every consecutive descendant heading up to the next heading at the same or a shallower level is hidden
- **AND** the collapsed heading remains visible with a collapsed disclosure state
- **AND** no heading navigation occurs

#### Scenario: Expand a heading subtree
- **WHEN** the user activates the collapsed disclosure control for a heading
- **THEN** its descendant rows become visible again
- **AND** nested headings that the user independently collapsed remain collapsed

#### Scenario: Leaf headings have no disclosure action
- **WHEN** a heading has no descendant heading in the outline hierarchy
- **THEN** its row has no actionable disclosure control
- **AND** its label remains aligned with sibling heading labels

#### Scenario: Folding state is isolated per document
- **WHEN** the user collapses a section in one document and switches between open document tabs
- **THEN** each document retains its own outline folding state for the current session
- **AND** collapsing one document does not hide headings in another document

#### Scenario: Click to jump outside Read mode
- **WHEN** the user clicks a heading label in the outline while Edit, Visual Edit, or Split Preview mode is active
- **THEN** the active editable surface navigates to that heading's source position as it did before this change
- **AND** the click does not change the heading's folding state

#### Scenario: Click to jump in Read mode
- **WHEN** the user clicks a heading label in the outline while Read mode is active
- **THEN** the preview pane brings the rendered heading for that outline item into view
- **AND** the canonical cursor moves to the clicked heading's source position
- **AND** the clicked heading becomes the active outline item
- **AND** the click does not change the heading's folding state

#### Scenario: Active section is inside a collapsed subtree
- **WHEN** the canonical cursor's active heading is hidden beneath a collapsed ancestor
- **THEN** the nearest visible collapsed ancestor is highlighted as containing the active section
- **AND** cursor movement alone does not discard the user's collapsed state

#### Scenario: Outline interactions are non-mutating
- **WHEN** the user navigates, collapses, or expands headings through the outline
- **THEN** the document text, version, dirty state, selection, and undo/redo history remain unchanged

#### Scenario: Outline rows use compact vertical spacing
- **WHEN** the outline contains consecutive visible single-line headings
- **THEN** each row has no extra inter-row margin and no more than 2px total vertical padding
- **AND** hierarchy indentation, readable labels, disclosure affordances, hover feedback, active highlighting, and click targets remain intact

#### Scenario: Overflowing outline is vertically scrollable
- **WHEN** the expanded portions of the outline contain more headings than fit in the visible sidebar height
- **THEN** mouse-wheel or trackpad input over the outline scrolls its visible heading rows vertically
- **AND** every currently visible heading can be brought into view and activated
