## ADDED Requirements

### Requirement: Pane scroll state with visible scrollbars
The editor SHALL preserve each tab's source editor and rendered preview scroll positions while exposing visible scrollbar controls for those panes. Using a scrollbar, mouse wheel, or trackpad SHALL update the same per-tab scroll state without modifying document text or derived Markdown state.

#### Scenario: Editor scrollbar preserves tab scroll state
- **WHEN** the user scrolls the source editor pane by dragging its scrollbar and then switches away from and back to the tab
- **THEN** the source editor pane returns to the same scroll position

#### Scenario: Preview scrollbar preserves tab scroll state
- **WHEN** the user scrolls the rendered preview pane by dragging its scrollbar and then switches away from and back to the tab
- **THEN** the rendered preview pane returns to the same scroll position

#### Scenario: Scrollbar navigation does not mutate document state
- **WHEN** the user drags the editor or preview scrollbar
- **THEN** the document text, dirty flag, undo/redo history, preview blocks, outline, stats, syntax highlighting cache, and cached text handle remain governed by the existing document-version rules
