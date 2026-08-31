## ADDED Requirements

### Requirement: Visual Edit caret placement preserves the viewport
When Visual Edit is active, moving the source caret SHALL change the virtualized list scroll offset only when the caret would otherwise sit outside the current viewport plus a small inset margin. A pointer click or in-viewport drag that hit-tests an already painted Visual Edit row, and whose resulting caret remains inside that inset, SHALL leave `visual_list` scroll state unchanged so the caret appears at the click location without moving the rendered text. Keyboard navigation, search navigation, mode entry, and caret-moving edits SHALL still reveal an off-screen caret, but they SHALL use the same geometry test: if the target caret or its owning painted row is already inside the inset, they SHALL NOT pin that row to the viewport top or otherwise jump the document. Pinning a later list item to the top is reserved for unmeasured rows that cannot yet be revealed by bounds. Pixel-follow after paint SHALL apply only the minimum delta needed to bring a clipped caret into the inset. Caret geometry, reveal flags, and scroll adjustments SHALL remain per-tab interaction state and SHALL NOT increment `MarkdownDocument.version()` or invalidate derived Markdown caches.

#### Scenario: Clicking a visible mid-document row does not scroll
- **WHEN** the user clicks painted Visual Edit text that is already fully inside the viewport and is not the last content line sitting on the clip
- **THEN** the source caret moves to the clicked source offset
- **AND** the Visual Edit list `logical_scroll_top` is unchanged
- **AND** the painted caret remains at the click location

#### Scenario: Clicking a visible lower row does not pin it to the top
- **WHEN** the Visual Edit viewport is scrolled so several rows are visible
- **AND** the user clicks a later painted row that is still fully inside the viewport inset
- **THEN** that row is not scrolled to the viewport top
- **AND** already-visible rendered text does not jump

#### Scenario: In-viewport drag selection does not jump the document
- **WHEN** the user drag-selects Visual Edit text that stays inside the viewport inset
- **THEN** the source selection updates
- **AND** the Visual Edit list scroll offset is unchanged

#### Scenario: Last-line click stays put when the caret remains in view
- **WHEN** the last rendered content line is already fully inside the viewport inset
- **AND** the user clicks that line
- **THEN** the caret is placed at the click location
- **AND** the viewport does not jump

#### Scenario: Off-screen keyboard or search navigation still reveals the caret
- **WHEN** keyboard navigation, search navigation, or mode entry moves the source caret to a visual row outside the current viewport inset
- **THEN** the Visual Edit list scrolls the minimum amount needed to bring that caret into the inset
- **AND** a later manual wheel or scrollbar movement is not forced back to the caret unless another off-inset caret move occurs

#### Scenario: Last-line typing that would clip follows by a minimum delta
- **WHEN** a caret-moving edit at the document tail would place the painted caret below the viewport inset
- **THEN** the list scrolls just enough to keep the caret inside the inset
- **AND** it does not pin the tail row to the viewport top if that row is already measured

#### Scenario: Unmeasured tail rows can still be pinned to become measurable
- **WHEN** a caret-moving edit creates or targets a Visual Edit row that has no measured height and sits below the measured window
- **THEN** the list may pin that item so it can be laid out
- **AND** a subsequent pixel-follow keeps the painted caret inside the inset

#### Scenario: Pointer placement does not reparse
- **WHEN** the user clicks or drag-selects in Visual Edit without changing document text
- **THEN** the document version, dirty flag, undo history, and derived Markdown caches remain unchanged

## MODIFIED Requirements

### Requirement: Pane scroll state with visible scrollbars
The editor SHALL preserve each tab's source editor, Visual Edit, and rendered preview scroll positions while exposing visible scrollbar controls for those surfaces. Using a scrollbar, mouse wheel, or trackpad SHALL update the same per-tab scroll state for the visible surface without modifying document text or derived Markdown state. Visual Edit SHALL keep its own per-tab virtualized-list scroll state, independent of the rendered preview list, even though both may represent the same document. Visual Edit SHALL include a trailing document-end padding band in its scrollable extent, sized from the current Visual Edit viewport (about half the viewport height), so the last rendered content line can be scrolled away from the pane clip and last-line pointer placement does not have to jump already-visible text. That padding is presentation-only: it SHALL NOT appear in `MarkdownDocument.text`, in cached `VisualBlock` slices, or in other derived Markdown state. When the persisted Sync scroll preference is enabled and the active view mode is Split Preview, scrolling either pane SHALL additionally update the other pane's per-tab scroll position so both viewport anchors represent the same source-backed document location, using rendered preview blocks' source ranges and within-block progress instead of matching whole-document scroll fractions. This coupling SHALL NOT merge the two panes' scroll states into a shared scroll: each pane SHALL retain its own scroll handle or list state, driver/follower observations SHALL remain isolated per tab, and a programmatic follower update SHALL NOT be mistaken for new user input. Synchronization SHALL NOT reset the preview list, reparse the document, mutate document text, or invalidate derived Markdown caches. When Sync scroll is disabled, when the active view mode is not Split Preview, or when no current source mapping is available, the two panes SHALL not be coupled. Scrolling Visual Edit, including by dragging its scrollbar, SHALL NOT establish a Split Preview sync-scroll driver.

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

#### Scenario: Visual Edit end padding lets the last line leave the clip
- **WHEN** a Visual Edit document has at least one rendered content row and the pane is taller than that row
- **THEN** the scrollable extent includes a trailing padding band of about half the current Visual Edit viewport
- **AND** the user can scroll until the last rendered content line sits away from the pane bottom clip

#### Scenario: Visual Edit end padding does not enter the document
- **WHEN** the Visual Edit list shows its document-end padding band
- **THEN** `MarkdownDocument.text`, the cached visual-block slice, dirty state, and derived Markdown caches are unchanged by the presence of that padding

#### Scenario: Clicking the end padding places the caret at the document end
- **WHEN** the user clicks the Visual Edit document-end padding band
- **THEN** the source caret moves to the document end
- **AND** the list does not pin a content row to the viewport top unless the resulting caret would sit outside the viewport inset
