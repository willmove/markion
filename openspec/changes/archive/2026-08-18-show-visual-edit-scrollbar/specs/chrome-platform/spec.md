## MODIFIED Requirements

### Requirement: Dense pane chrome with draggable scrollbars
The application chrome SHALL provide visible, right-side vertical scrollbars for the source editor pane, Visual Edit surface, and rendered preview pane when their content exceeds the visible area. The Visual Edit scrollbar SHALL match the Read-mode preview overlay in placement and drag behavior. The editor SHALL keep main pane gaps, outer padding, and visible separator chrome compact so the source and preview content occupy substantially more of the available window area than the prior spacious layout. Resize handles SHALL remain draggable even when their visible separator is compact.

#### Scenario: Large source document exposes editor scrollbar
- **WHEN** the active document has more source lines than fit in the editor pane
- **THEN** the editor pane shows a right-side vertical scrollbar
- **AND** dragging that scrollbar changes the visible source text

#### Scenario: Large rendered document exposes preview scrollbar
- **WHEN** the active document renders more preview content than fits in the preview pane
- **THEN** the preview pane shows a right-side vertical scrollbar
- **AND** dragging that scrollbar changes the visible rendered content

#### Scenario: Large Visual Edit document exposes a scrollbar
- **WHEN** the active view mode is Visual Edit
- **AND** the visual document renders more content than fits in the visible surface
- **THEN** the Visual Edit surface shows a right-side vertical scrollbar
- **AND** dragging that scrollbar changes the visible visual document content
- **AND** the thumb placement and drag behavior match the Read-mode preview scrollbar

#### Scenario: Short or empty Visual Edit documents hide the scrollbar
- **WHEN** the active view mode is Visual Edit
- **AND** the visual document fits in the visible surface or the document is empty
- **THEN** no vertical scrollbar thumb is shown

#### Scenario: Main pane chrome is compact
- **WHEN** the editor renders the main content area
- **THEN** the visual gaps between the sidebar, editor pane, split divider, and preview pane are reduced to approximately 15% of the previous spacious padding
- **AND** source and preview content occupy the reclaimed space

#### Scenario: Resize handles remain usable
- **WHEN** the visible sidebar or editor/preview separator is compact
- **THEN** the user can still drag the separator handle to resize the corresponding panes

#### Scenario: Single-pane modes remain full-width
- **WHEN** the active view mode is Edit or Read
- **THEN** the visible editor or preview pane fills the remaining main workspace instead of retaining split-mode width
