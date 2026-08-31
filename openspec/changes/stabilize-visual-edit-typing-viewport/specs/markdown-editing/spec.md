## MODIFIED Requirements

### Requirement: Visual Edit caret placement preserves the viewport
When Visual Edit is active, moving the source caret SHALL change the virtualized list scroll offset only when the caret would otherwise sit outside the current viewport plus a small inset margin. A pointer click or in-viewport drag that hit-tests an already painted Visual Edit row, and whose resulting caret remains inside that inset, SHALL leave `visual_list` scroll state unchanged so the caret appears at the click location without moving the rendered text. Keyboard navigation, search navigation, mode entry, and caret-moving edits SHALL still reveal an off-screen caret, but they SHALL use the same geometry test: if the target caret or its owning painted row is already inside the inset, they SHALL NOT pin that row to the viewport top or otherwise jump the document. Pinning a later list item to the top is reserved for rows that have no usable caret or item geometry and that sit below the previously measured window; a row that was in the prior visible range and is only temporarily unmeasured because it was spliced SHALL NOT be pinned. When item bounds are missing, the geometry test SHALL use the last painted caret rectangle for that tab if it is still a positive-height rect. Pixel-follow after paint SHALL apply only the minimum delta needed to bring a clipped caret into the inset. Caret geometry, reveal flags, and scroll adjustments SHALL remain per-tab interaction state and SHALL NOT increment `MarkdownDocument.version()` or invalidate derived Markdown caches.

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

#### Scenario: Typing in a visible mid-document row does not scroll
- **WHEN** the Visual Edit viewport is scrolled so several rows are visible
- **AND** the source caret is in a later painted row that is fully inside the viewport inset
- **AND** the user types or replaces text in that row without moving the caret outside the inset
- **THEN** the Visual Edit list `logical_scroll_top` is unchanged
- **AND** that row is not pinned to the viewport top

#### Scenario: IME replacement in a visible row does not pin
- **WHEN** an IME composition or other in-viewport source replacement updates the document while the caret remains inside the viewport inset
- **THEN** the Visual Edit list does not pin the owning row to the viewport top
- **AND** `logical_scroll_top` is unchanged except for the minimum pixel-follow if the painted caret would clip

#### Scenario: In-viewport Enter does not pin a still-visible successor
- **WHEN** the user presses Enter in Visual Edit and the resulting caret remains inside the viewport inset
- **THEN** the list does not pin the new or split row to the viewport top solely because it was spliced and briefly unmeasured
- **AND** already-visible rendered text above the caret does not jump away

#### Scenario: Last-line typing that would clip follows by a minimum delta
- **WHEN** a caret-moving edit at the document tail would place the painted caret below the viewport inset
- **THEN** the list scrolls just enough to keep the caret inside the inset
- **AND** it does not pin the tail row to the viewport top if that row is already measured or was already in the visible range

#### Scenario: Unmeasured tail rows can still be pinned to become measurable
- **WHEN** a caret-moving edit creates or targets a Visual Edit row that has no usable caret or item geometry and sits below the previously measured window
- **THEN** the list may pin that item so it can be laid out
- **AND** a subsequent pixel-follow keeps the painted caret inside the inset

#### Scenario: Pointer placement does not reparse
- **WHEN** the user clicks or drag-selects in Visual Edit without changing document text
- **THEN** the document version, dirty flag, undo history, and derived Markdown caches remain unchanged

### Requirement: Stable source-mapped visual block identity
Every derived Visual Edit block SHALL carry an opaque, non-persisted identity that remains stable across document versions when the block is proven to descend from the same source block either unchanged or as a 1:1 in-place successor of the same kind. Identity SHALL be independent from the block's current byte range and SHALL NOT replace canonical source ranges for editing. Identity SHALL NOT be reused across splits, merges, kind changes, or ambiguous reparses.

Cached per-row layout measurements (such as virtualized-list row heights and scroll anchoring) MAY be reused for identity-preserved rows. A row whose rendered geometry depends on source content that can change without changing the block's identity — a whitespace row whose height follows its covered blank-line count — SHALL have its cached measurements invalidated and re-measured whenever that height-relevant signature changes, even though the block identity remains stable.

#### Scenario: Prefix edit preserves shifted suffix identity
- **WHEN** a localized edit changes one block and shifts later unchanged blocks by a byte delta
- **THEN** each proven unchanged suffix block retains its prior visual block identity
- **AND** its source ranges are shifted to the exact current canonical offsets

#### Scenario: In-place same-kind edit preserves identity
- **WHEN** a localized edit rewrites text inside one visual block without splitting it, merging it, or changing its kind
- **THEN** that successor retains its prior visual block identity
- **AND** the virtualized Visual Edit list does not splice that row solely because its source bytes changed
- **AND** later unchanged blocks still retain their identities

#### Scenario: Split, merge, or kind change receives new identity
- **WHEN** an edit splits, merges, or changes the kind of a visual block, or ambiguously reparses it
- **THEN** every affected resulting block receives a new identity
- **AND** stale row layout, navigation, or widget state is not attached to it

#### Scenario: Repeated equal blocks remain occurrence-safe
- **WHEN** a document contains multiple textually equal blocks and an edit affects only one occurrence
- **THEN** identity reuse follows source-edit lineage and occurrence order
- **AND** an unchanged occurrence is not confused with the edited occurrence solely because their text hashes match

#### Scenario: Local edit invalidates only affected visual rows
- **WHEN** stable identities prove that visual rows outside an edited region are unchanged
- **THEN** the virtualized Visual Edit list splices only rows whose identity or height signature actually changed
- **AND** cached row heights and scroll anchoring state remain reusable for identity-preserved rows whose geometry is determined by their proven-unchanged content

#### Scenario: Height-mutable row does not reuse a stale cached height
- **WHEN** a whitespace row keeps its visual block identity while the blank-line count it covers changes — for example repeated Enter at the document tail
- **THEN** the virtualized list re-measures that row instead of reusing its previously cached height
- **AND** the list's total content height and scroll extent reflect the whitespace row's current true height, including when the row is outside the currently visible range

#### Scenario: Identity and incremental cache remain ephemeral
- **WHEN** a document is saved, reopened, recovered, cloned for undo, or replaced wholesale
- **THEN** visual identities and incremental region caches are rebuilt rather than persisted
- **AND** Markdown file contents and undo snapshot formats remain unchanged
