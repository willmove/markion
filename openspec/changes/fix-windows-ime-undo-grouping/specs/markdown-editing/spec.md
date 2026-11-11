## MODIFIED Requirements

### Requirement: Visual Edit IME composition fidelity
Visual Edit SHALL treat the active IME marked range as first-class projection and rendering state. The marked source SHALL remain visibly identified, precisely mapped, and correctly positioned for the platform candidate window throughout composition, including UTF-16 input containing CJK text, emoji, or combining characters. A live composition SHALL retain its semantic undo capture across candidate-selection pauses, independently of the ordinary typing coalescing timeout.

#### Scenario: Marked text is visible in the mixed projection
- **WHEN** an IME composition creates or updates a non-empty marked range inside rendered inline content
- **THEN** Visual Edit reveals any exact containing syntax needed to identity-map the marked source
- **AND** the painted marked range uses the platform composition underline without losing its inline content

#### Scenario: Candidate geometry follows the active marked range
- **WHEN** GPUI requests bounds for the active composition after the owning visual row has been laid out
- **THEN** Visual Edit returns geometry derived from the requested projected range
- **AND** the surface-level fallback is used only while exact row geometry is unavailable

#### Scenario: One IME composition is one undoable action
- **WHEN** an IME session produces multiple intermediate marked-text replacements and then commits
- **THEN** one Undo restores the source and selection from before that composition began
- **AND** one Redo reapplies the committed composition result

#### Scenario: Candidate-selection pauses preserve the same undo entry
- **WHEN** the user pauses longer than the ordinary typing capture timeout while choosing candidates or between marked-text updates, then commits
- **THEN** the whole live composition remains one undo entry
- **AND** Undo restores pre-composition source rather than an intermediate Pinyin or marked-text value

#### Scenario: UTF-16 composition remains UTF-8 safe
- **WHEN** IME replacement or selection ranges include CJK text, emoji, or combining characters
- **THEN** boundary conversion, projection, and marked-range painting resolve to valid canonical UTF-8 boundaries
- **AND** no partial code point is inserted, selected, or underlined

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

#### Scenario: Remeasuring a visible table does not pin it to the top
- **WHEN** a cell edit or its Undo remeasures a source-owned table that was fully inside the viewport inset
- **THEN** the pre-layout reveal does not pin that table to the viewport top
- **AND** post-paint caret follow applies only the minimum overflow delta if the new content needs it
