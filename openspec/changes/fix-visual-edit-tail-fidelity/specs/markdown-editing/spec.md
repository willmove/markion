## MODIFIED Requirements

### Requirement: Visual Edit whitespace activation
The system SHALL keep source-backed whitespace ranges available for exact caret mapping while treating whitespace between rendered blocks as passive layout until the source caret intentionally enters that range. When the source caret owns a whitespace row — whether because the user pressed Enter at the end of a paragraph (whose source range excludes the trailing newline) or because keyboard navigation moved the caret into a whitespace-only range — Visual Edit SHALL present the row as the same passive-height layout it uses when unfocused, plus a thin insertion caret line visually consistent with the caret in a paragraph or heading, and SHALL accept subsequent typed text at the exact source caret position. Visual Edit SHALL NOT wrap a whitespace row that owns the caret in a source-island box (border, padding, monospace styling, or differentiated background), because such chrome misrepresents ordinary inter-paragraph spacing as a code-like block. Source islands SHALL remain reserved for blocks whose source has no rendered visual form (frontmatter, code, HTML, unsupported constructs) or for inline runs whose source/display mapping is ambiguous and therefore requires a conservative source-editing fallback.

A whitespace row's passive height SHALL correspond to the amount of blank source it represents: at least one line height for any non-empty whitespace range, and proportionally more as the covered blank-line count grows. The height SHALL NOT be clamped to a small fixed maximum that hides document growth; only a generous sanity bound (a large multiple of the viewport height) MAY apply to pathological documents. Inserting or removing blank lines inside a whitespace range SHALL therefore produce a visible change in the rendered surface even though the row contains no text.

#### Scenario: Clicking a passive gap between headings does not activate editing
- **WHEN** the Visual Edit caret belongs to a rendered heading and the user clicks the whitespace gap between that heading and another heading
- **THEN** the source selection and document content remain unchanged and the gap does not present an insertion caret

#### Scenario: Clicking a passive gap before a paragraph does not activate editing
- **WHEN** the Visual Edit caret belongs to a rendered block and the user clicks the whitespace gap between a heading and a paragraph
- **THEN** the source selection and document content remain unchanged and the gap does not become an editable typing area

#### Scenario: Structural Enter activates an insertion line
- **WHEN** the user presses Enter from a heading in Visual Edit and the structural edit creates a new source-backed insertion line
- **THEN** the owning visual row presents the caret and accepts subsequent typed text at the exact source position regardless of whether the parser retains the newline in the heading range

#### Scenario: Intentional source caret movement preserves whitespace editing
- **WHEN** keyboard navigation or reveal logic moves the source caret into an existing whitespace-only range
- **THEN** the owning whitespace row provides the source-backed editing affordance without recomputing the document's cached Markdown-derived state

#### Scenario: Whitespace row owning the caret renders a caret line, not a source island
- **WHEN** the source caret owns a whitespace row in Visual Edit — for example after creating a blank line by pressing Enter (so a second newline lands outside any paragraph range), or after pressing Down arrow across an existing blank line
- **THEN** the row is rendered as passive-height layout with a thin insertion caret line and no border, padding, monospace styling, or differentiated background
- **AND** typed text is inserted into the canonical Markdown source at the caret position through the same dirty-state, undo/redo, autosave, and per-tab isolation paths as any other edit

#### Scenario: Whitespace row not owning the caret remains passive
- **WHEN** a whitespace row does not own the source caret
- **THEN** it renders as passive layout without a caret, exactly as before, regardless of whether it owns the caret on other frames

#### Scenario: Repeated Enter at the document tail keeps growing the visible blank region
- **WHEN** the caret is in the trailing whitespace of the document and the user presses Enter several times in a row
- **THEN** each press inserts one newline (plus continuation prefix) into the canonical source
- **AND** the trailing whitespace row's rendered height grows visibly with each press instead of stopping after a fixed small height
- **AND** the insertion caret moves down the grown row with each press instead of remaining painted at the row origin
- **AND** no press is silently swallowed or hidden by the rendering

#### Scenario: Typing at the last visible line keeps the caret and new text in view
- **WHEN** the Visual Edit caret is on the last rendered line of a document whose content is taller than the pane
- **AND** the user types characters or presses Enter so the last row grows past the current viewport bottom
- **THEN** the list scrolls enough that the painted caret and the newly inserted text remain fully visible
- **AND** the caret does not appear stuck at its previous screen position

#### Scenario: Whitespace row height tracks blank-line count in both directions
- **WHEN** blank lines covered by a whitespace row are added or removed — including via undo, redo, or external reload
- **THEN** the row's rendered height reflects the new blank-line count on the next rendered frame
- **AND** a document whose tail whitespace shrinks back to a single blank line renders that region at a single-line passive height

### Requirement: Stable source-mapped visual block identity
Every derived Visual Edit block SHALL carry an opaque, non-persisted identity that remains stable across document versions only when the block is proven to descend unchanged from the same source block. Identity SHALL be independent from the block's current byte range and SHALL NOT replace canonical source ranges for editing.

Cached per-row layout measurements (such as virtualized-list row heights and scroll anchoring) MAY be reused for identity-preserved rows only when the row's rendered geometry is determined by its proven-unchanged content. A row whose rendered geometry depends on source content that can change without changing the block's identity — a whitespace row whose height follows its covered blank-line count — SHALL have its cached measurements invalidated and re-measured whenever that height-relevant signature changes, even though the block identity remains stable.

#### Scenario: Prefix edit preserves shifted suffix identity
- **WHEN** a localized edit changes one block and shifts later unchanged blocks by a byte delta
- **THEN** each proven unchanged suffix block retains its prior visual block identity
- **AND** its source ranges are shifted to the exact current canonical offsets

#### Scenario: Changed block receives new identity
- **WHEN** an edit changes, splits, merges, or ambiguously reparses a visual block
- **THEN** every affected resulting block receives a new identity
- **AND** stale row layout, navigation, or widget state is not attached to it

#### Scenario: Repeated equal blocks remain occurrence-safe
- **WHEN** a document contains multiple textually equal blocks and an edit affects only one occurrence
- **THEN** identity reuse follows source-edit lineage and occurrence order
- **AND** an unchanged occurrence is not confused with the edited occurrence solely because their text hashes match

#### Scenario: Local edit invalidates only affected visual rows
- **WHEN** stable identities prove that visual rows outside an edited region are unchanged
- **THEN** the virtualized Visual Edit list splices only the affected middle rows
- **AND** cached row heights and scroll anchoring state remain reusable for rows whose geometry is determined by their proven-unchanged content

#### Scenario: Height-mutable row does not reuse a stale cached height
- **WHEN** a whitespace row keeps its visual block identity while the blank-line count it covers changes — for example repeated Enter at the document tail
- **THEN** the virtualized list re-measures that row instead of reusing its previously cached height
- **AND** the list's total content height and scroll extent reflect the whitespace row's current true height, including when the row is outside the currently visible range

#### Scenario: Identity and incremental cache remain ephemeral
- **WHEN** a document is saved, reopened, recovered, cloned for undo, or replaced wholesale
- **THEN** visual identities and incremental region caches are rebuilt rather than persisted
- **AND** Markdown file contents and undo snapshot formats remain unchanged

### Requirement: Pane scroll state with visible scrollbars
The editor SHALL preserve each tab's source editor, Visual Edit, and rendered preview scroll positions while exposing visible scrollbar controls for those surfaces. Using a scrollbar, mouse wheel, or trackpad SHALL update the same per-tab scroll state for the visible surface without modifying document text or derived Markdown state. Visual Edit SHALL keep its own per-tab virtualized-list scroll state, independent of the rendered preview list, even though both may represent the same document. When the persisted Sync scroll preference is enabled and the active view mode is Split Preview, scrolling either pane SHALL additionally update the other pane's per-tab scroll position so both viewport anchors represent the same source-backed document location, using rendered preview blocks' source ranges and within-block progress instead of matching whole-document scroll fractions. This coupling SHALL NOT merge the two panes' scroll states into a shared scroll: each pane SHALL retain its own scroll handle or list state, driver/follower observations SHALL remain isolated per tab, and a programmatic follower update SHALL NOT be mistaken for new user input. Synchronization SHALL NOT reset the preview list, reparse the document, mutate document text, or invalidate derived Markdown caches. When Sync scroll is disabled, when the active view mode is not Split Preview, or when no current source mapping is available, the two panes SHALL not be coupled. Scrolling Visual Edit, including by dragging its scrollbar, SHALL NOT establish a Split Preview sync-scroll driver.

Each virtualized pane (Visual Edit and rendered preview) SHALL expose a scroll range that covers its complete rendered content, including the vertical padding applied around the list, such that the user can scroll the final rendered row fully into view. Scroll extents and scrollbar geometry SHALL be derived from the pane's current true content height — accounting for rows whose heights change while identity-preserved — rather than from stale per-row measurements.

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

#### Scenario: Last rendered line is fully scrollable into view
- **WHEN** a document's Visual Edit content is taller than the pane and the user scrolls to the bottom of the surface by any means (scrollbar, wheel, or caret reveal)
- **THEN** the final rendered row — including the last text line and any trailing whitespace row — can be brought completely into view, unclipped by the pane's bottom edge or reserved scrollbar area

#### Scenario: Growth of off-screen tail whitespace extends the scroll range
- **WHEN** trailing blank lines are added while the tail whitespace row is outside the visible range
- **THEN** the surface's scrollable range and scrollbar extent grow to include the whitespace row's new true height
- **AND** scrolling to the new bottom reveals the full grown whitespace region

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
