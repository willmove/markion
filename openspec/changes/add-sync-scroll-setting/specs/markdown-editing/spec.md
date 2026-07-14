## MODIFIED Requirements

### Requirement: Pane scroll state with visible scrollbars
The editor SHALL preserve each tab's source editor and rendered preview scroll positions while exposing visible scrollbar controls for those panes. Using a scrollbar, mouse wheel, or trackpad SHALL update the same per-tab scroll state without modifying document text or derived Markdown state. When the persisted Sync scroll preference is enabled and the active view mode is Split Preview, scrolling either pane SHALL additionally update the other pane's per-tab scroll position to the same fraction of its scrollable range, clamped to its bounds; this coupling SHALL NOT merge the two panes' scroll states into a shared scroll (each pane retains its own scroll handle and the preview retains its own list state) and SHALL NOT reset the preview list or reparse the document. When Sync scroll is disabled, or the active view mode is not Split Preview, the two panes SHALL scroll independently.

#### Scenario: Editor scrollbar preserves tab scroll state
- **WHEN** the user scrolls the source editor pane by dragging its scrollbar and then switches away from and back to the tab
- **THEN** the source editor pane returns to the same scroll position

#### Scenario: Preview scrollbar preserves tab scroll state
- **WHEN** the user scrolls the rendered preview pane by dragging its scrollbar and then switches away from and back to the tab
- **THEN** the rendered preview pane returns to the same scroll position

#### Scenario: Scrollbar navigation does not mutate document state
- **WHEN** the user drags the editor or preview scrollbar
- **THEN** the document text, dirty flag, undo/redo history, preview blocks, outline, stats, syntax highlighting cache, and cached text handle remain governed by the existing document-version rules

#### Scenario: Sync scroll couples the panes without merging scroll state
- **WHEN** Sync scroll is enabled and the active view mode is Split Preview
- **AND** the user scrolls one of the two panes
- **THEN** the other pane's scroll position moves to the matching fraction of its scrollable range
- **AND** each pane still holds its own scroll handle/list state, and switching tabs still restores each tab's independent scroll positions
- **AND** no preview list reset or Markdown reparse occurs

#### Scenario: Independent scroll resumes when Sync scroll is disabled
- **WHEN** Sync scroll is disabled or the view mode is not Split Preview
- **THEN** scrolling one pane does not move the other pane
