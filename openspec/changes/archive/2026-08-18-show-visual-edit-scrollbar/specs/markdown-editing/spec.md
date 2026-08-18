## MODIFIED Requirements

### Requirement: Pane scroll state with visible scrollbars
The editor SHALL preserve each tab's source editor, Visual Edit, and rendered preview scroll positions while exposing visible scrollbar controls for those surfaces. Using a scrollbar, mouse wheel, or trackpad SHALL update the same per-tab scroll state for the visible surface without modifying document text or derived Markdown state. Visual Edit SHALL keep its own per-tab virtualized-list scroll state, independent of the rendered preview list, even though both may represent the same document. When the persisted Sync scroll preference is enabled and the active view mode is Split Preview, scrolling either pane SHALL additionally update the other pane's per-tab scroll position so both viewport anchors represent the same source-backed document location, using rendered preview blocks' source ranges and within-block progress instead of matching whole-document scroll fractions. This coupling SHALL NOT merge the two panes' scroll states into a shared scroll: each pane SHALL retain its own scroll handle or list state, driver/follower observations SHALL remain isolated per tab, and a programmatic follower update SHALL NOT be mistaken for new user input. Synchronization SHALL NOT reset the preview list, reparse the document, mutate document text, or invalidate derived Markdown caches. When Sync scroll is disabled, when the active view mode is not Split Preview, or when no current source mapping is available, the two panes SHALL not be coupled. Scrolling Visual Edit, including by dragging its scrollbar, SHALL NOT establish a Split Preview sync-scroll driver.

#### Scenario: Editor scrollbar preserves tab scroll state
- **WHEN** the user scrolls the source editor pane by dragging its scrollbar and then switches away from and back to the tab
- **THEN** the source editor pane returns to the same scroll position

#### Scenario: Preview scrollbar preserves tab scroll state
- **WHEN** the user scrolls the rendered preview pane by dragging its scrollbar and then switches away from and back to the tab
- **THEN** the rendered preview pane returns to the same scroll position

#### Scenario: Visual Edit scrollbar preserves tab scroll state
- **WHEN** the user scrolls the Visual Edit surface by dragging its scrollbar and then switches away from and back to the tab
- **THEN** the Visual Edit surface returns to the same scroll position
- **AND** the rendered preview scroll position for that tab is unchanged

#### Scenario: Scrollbar navigation does not mutate document state
- **WHEN** the user drags the editor, Visual Edit, or preview scrollbar
- **THEN** the document text, dirty flag, undo/redo history, preview blocks, outline, stats, syntax highlighting cache, and cached text handle remain governed by the existing document-version rules

#### Scenario: Visual Edit scrollbar does not drive Sync scroll
- **WHEN** Sync scroll is enabled
- **AND** the user drags the Visual Edit scrollbar or otherwise scrolls Visual Edit
- **THEN** no Split Preview follower pane is moved
- **AND** later entering Split Preview does not treat that Visual Edit scroll as a preview-driven sync update

#### Scenario: Sync scroll couples panes by document location without merging state
- **WHEN** Sync scroll is enabled and the active view mode is Split Preview
- **AND** the user scrolls one of the two panes
- **THEN** the other pane moves to the source-backed document location represented by the driving pane's viewport anchor
- **AND** each pane still holds its own scroll handle or list state, and switching tabs still restores each tab's independent scroll positions
- **AND** no preview list reset, document mutation, cache invalidation, or Markdown reparse occurs

#### Scenario: Local height differences do not select an unrelated block
- **WHEN** the source and rendered representations have non-uniform local height ratios
- **AND** Sync scroll follows a scroll across those regions
- **THEN** the follower remains aligned to the driving pane's source-backed block and relative position rather than to the same fraction of its total scrollable range

#### Scenario: Programmatic follower movement does not reverse the driver
- **WHEN** Sync scroll writes a mapped target to the follower pane
- **THEN** the next reconciliation treats that movement as the expected follower result
- **AND** it does not move the original driving pane back toward the follower's previous position

#### Scenario: Independent scroll resumes when Sync scroll is disabled
- **WHEN** Sync scroll is disabled or the view mode is not Split Preview
- **THEN** scrolling one pane does not move the other pane
