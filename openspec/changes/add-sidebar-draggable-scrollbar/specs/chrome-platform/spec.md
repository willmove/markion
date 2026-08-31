## MODIFIED Requirements

### Requirement: Dense pane chrome with draggable scrollbars
The application chrome SHALL provide visible, right-side vertical scrollbars for the source editor pane, Visual Edit surface, and rendered preview pane when their content exceeds the visible area. The Visual Edit scrollbar SHALL match the Read-mode preview overlay in placement and drag behavior. The Preferences panel SHALL provide the same draggable, right-side vertical scrollbars for each of its scrollable regions — the General tab body, the Shortcuts category sidebar, the Shortcuts action list, and the Export tab body — whenever a region's content exceeds its visible area; wheel and trackpad scrolling SHALL continue to work unchanged. The visible left sidebar SHALL provide the same draggable, right-side vertical scrollbar for the Files tree list and the Outline heading list whenever that list exceeds its visible height; wheel and trackpad scrolling SHALL continue to work unchanged, and dragging a sidebar scrollbar SHALL NOT drive Sync scroll. The editor SHALL keep main pane gaps, outer padding, and visible separator chrome compact so the source and preview content occupy substantially more of the available window area than the prior spacious layout. Resize handles SHALL remain draggable even when their visible separator is compact.

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

#### Scenario: Overflowing Preferences panel region exposes a scrollbar
- **WHEN** the Preferences panel is open
- **AND** a scrollable panel region (General tab body, Shortcuts category sidebar, Shortcuts action list, or Export tab body) contains more content than fits its visible area
- **THEN** that region shows a right-side vertical scrollbar thumb
- **AND** dragging the thumb with the left mouse button scrolls that region up and down
- **AND** the thumb position reflects the region's scroll offset

#### Scenario: Fitting Preferences panel content hides the scrollbar
- **WHEN** the Preferences panel is open
- **AND** a scrollable panel region's content fits within its visible area
- **THEN** no vertical scrollbar thumb is shown for that region

#### Scenario: Preferences panel wheel scrolling is preserved
- **WHEN** the Preferences panel is open
- **AND** the user scrolls a scrollable panel region with the mouse wheel or trackpad
- **THEN** the region scrolls exactly as before the draggable scrollbar was added
- **AND** the scrollbar thumb moves to reflect the new scroll offset

#### Scenario: Overflowing Files sidebar exposes a scrollbar
- **WHEN** the sidebar is visible on the Files tab
- **AND** the file-tree list contains more rows than fit in the visible list height
- **THEN** the Files list shows a right-side vertical scrollbar thumb
- **AND** dragging that thumb with the left mouse button scrolls the file-tree rows up and down
- **AND** the thumb position reflects the list's scroll offset

#### Scenario: Fitting Files sidebar hides the scrollbar
- **WHEN** the sidebar is visible on the Files tab
- **AND** the file-tree list fits in the visible list height
- **THEN** no vertical scrollbar thumb is shown for the Files list

#### Scenario: Overflowing Outline sidebar exposes a scrollbar
- **WHEN** the sidebar is visible on the Outline tab for a Markdown document
- **AND** the visible outline rows exceed the panel height
- **THEN** the Outline list shows a right-side vertical scrollbar thumb
- **AND** dragging that thumb with the left mouse button scrolls the heading rows up and down
- **AND** the thumb position reflects the list's scroll offset

#### Scenario: Fitting Outline sidebar hides the scrollbar
- **WHEN** the sidebar is visible on the Outline tab
- **AND** the visible outline rows fit in the panel height, or the active tab is an image
- **THEN** no vertical scrollbar thumb is shown for the Outline list

#### Scenario: Sidebar wheel scrolling is preserved
- **WHEN** the sidebar is visible on Files or Outline
- **AND** the user scrolls that list with the mouse wheel or trackpad
- **THEN** the list scrolls as it did before the draggable scrollbar was added
- **AND** the scrollbar thumb, when shown, moves to reflect the new scroll offset

#### Scenario: Sidebar scrollbar does not drive Sync scroll
- **WHEN** Sync scroll is enabled and the active view mode is Split Preview
- **AND** the user drags the Files or Outline scrollbar
- **THEN** the source editor and rendered preview keep their current independent scroll positions
- **AND** no Sync-scroll driver is recorded from the sidebar drag

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
