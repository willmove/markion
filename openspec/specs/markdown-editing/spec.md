# markdown-editing

## Purpose

Covers canonical Markdown source editing, parsing and formatting together with the source-backed, WYSIWYG-oriented Visual Edit surface. Edit and Split modes retain complete raw-source access; Visual Edit keeps exactly mapped constructs rendered or directly editable, progressively reveals only necessary syntax, and preserves complete source islands whenever a lossless mutation cannot be proven.
## Requirements
### Requirement: Markdown parsing via CommonMark + GFM
The parser SHALL parse Markdown using `pulldown-cmark` configured for CommonMark conformance plus the GitHub Flavored Markdown extensions in use (tables, task lists, strikethrough, footnotes, superscript/subscript, highlight, autolinks). Parsing SHALL produce structured data consumed by the preview, Visual Edit, outline, stats, and search subsystems. Source-mapped Visual Edit derivation SHALL incrementally reuse independently parseable top-level regions after a localized source edit and SHALL fall back to a full-document parse whenever global Markdown context, region boundaries, or exact source ranges are uncertain. Incremental and fallback output SHALL be semantically and byte-range equivalent to a full parse of the current canonical source.

#### Scenario: Local edit reparses affected safe regions
- **WHEN** a source edit is confined to an independently parseable top-level region
- **THEN** source-mapped derivation reparses that region and bounded boundary context
- **AND** text-identical unaffected regions are reused without reparsing

#### Scenario: Globally scoped syntax uses full fallback
- **WHEN** an edit can affect reference definitions, footnotes, front matter, an unclosed fence, HTML block boundaries, or another cross-region parse dependency
- **THEN** the editor derives the source-mapped model through the full-document parser
- **AND** it does not publish a speculative incremental mapping

#### Scenario: Incremental output equals full parse
- **WHEN** incremental derivation accepts an edit sequence containing insertions, deletions, replacements, UTF-8 text, block splits, or block merges
- **THEN** its block variants, content, ordering, outline, and every source range equal a full parse of the same canonical source after each edit

#### Scenario: Extended inline syntax is recognized
- **WHEN** the document contains `==highlight==`, `^superscript^`, `~subscript~`, task list items, or footnote references
- **THEN** the parser recognizes these constructs and the preview renders them with their respective styles

#### Scenario: Nested Markdown constructs are preserved
- **WHEN** a block construct contains inline or nested constructs (e.g. a list with nested code, a blockquote with a table)
- **THEN** the parser handles the nesting per CommonMark precedence rules

### Requirement: Source-text editing with formatting actions
The editor SHALL provide a source-text editing surface where the user types raw Markdown, plus a set of formatting actions (bold, italic, inline code, links, images, headings, lists, task lists, blockquotes, fenced code blocks) that wrap or transform the selected text into the corresponding Markdown syntax. Heading actions exposed in the Format menu and via keyboard shortcuts SHALL include every level from H1 through the configured Heading menu depth maximum (default H1–H5, optionally H1–H6).

#### Scenario: Formatting actions wrap the selection
- **WHEN** the user triggers a formatting action (e.g. bold) with a selection
- **THEN** the selected text is wrapped in the corresponding Markdown markers and the document updates

#### Scenario: Common editing operations
- **WHEN** the user performs insert/delete, selection, copy/cut/paste, undo/redo, or select all
- **THEN** the editor applies the operation and reports status feedback (including for empty-clipboard or no-selection cases)

#### Scenario: Heading level switching
- **WHEN** the user switches the heading level of a line via the Format menu
- **THEN** the line's heading markers are updated without crashing

### Requirement: Keyboard shortcut system
The editor SHALL bind common formatting, file, view, and navigation operations to keyboard shortcuts, with platform-appropriate modifier conventions, and SHALL surface the full shortcut list in-app.

#### Scenario: Shortcuts follow platform conventions
- **WHEN** the editor runs on macOS vs Windows/Linux
- **THEN** shortcuts use the platform-appropriate modifier key convention (Cmd vs Ctrl)

#### Scenario: Full shortcut reference is available in-app
- **WHEN** the user opens the keyboard shortcut reference from the Help menu
- **THEN** the editor displays the complete, localized list of shortcuts

#### Scenario: Menu items display their shortcuts
- **WHEN** a menu item has an associated shortcut
- **THEN** the shortcut is shown alongside the item label

### Requirement: Multi-document tab model
The editor SHALL hold zero or more open content tabs within a single window (`tabs` plus an `active_tab` index), rather than a single document per window. A Markdown or curated text tab SHALL carry its own isolated document, cursor/selection, scroll position, undo/redo history, IME composition state, layout caches, dirty flag, and autosave/recovery tracking; a read-only image tab SHALL carry only the state required to identify, load, present, scroll, and close that image. Switching tabs SHALL NOT disturb another tab's state. Tabs for filesystem-backed content SHALL be unique by file path within a window: when an open request targets a file that is already open in another tab, the editor SHALL focus that existing tab instead of opening a duplicate tab. A tab bar SHALL be rendered only when more than one tab is open; with a single tab the active content surface SHALL use the same space as the pre-tab layout. Tabs are session-only: they are not persisted across launches (restarting returns to a single untitled document).

#### Scenario: Opening files creates switchable tabs with isolated state
- **WHEN** the user opens a second supported file via the file tree or the Open in New Tab action
- **THEN** a new document or image tab is appended and activated
- **AND** switching back to the first tab restores that tab's exact content-specific state

#### Scenario: Opening an already-open file focuses its existing tab
- **WHEN** the user opens a supported file by path and that same file is already open in a tab
- **THEN** the existing tab is activated
- **AND** no duplicate tab is appended or replaced
- **AND** an existing document tab preserves its text, dirty flag, cursor/selection, undo/redo history, editor scroll position, preview scroll position, and derived Markdown caches
- **AND** an existing image tab preserves its load result and presentation state

#### Scenario: File→Open replaces the active tab
- **WHEN** the user invokes File → Open and picks a supported file that is not already open
- **THEN** the active tab's content is replaced after applying a dirty guard when that tab contains an editable document, rather than spawning a new tab
- **AND** replacing a read-only image tab does not require a dirty confirmation

#### Scenario: Tab navigation and closing
- **WHEN** the user presses the next/previous tab shortcut (Ctrl+Tab / Ctrl+Shift+Tab) or clicks a tab / its close button
- **THEN** the active tab switches in opening order, or the targeted tab closes; closing the last tab creates a fresh untitled document rather than closing the window

#### Scenario: Closing an unsaved tab prompts for confirmation
- **WHEN** the user closes a document tab whose content has unsaved changes
- **THEN** the editor prompts for confirmation before discarding those changes
- **AND** closing a read-only image tab never presents an unsaved-changes prompt

#### Scenario: Quitting with multiple unsaved tabs
- **WHEN** the user quits or closes the window while two or more document tabs have unsaved changes and any number of image tabs are open
- **THEN** the editor detects the unsaved document tabs and prompts before discarding them
- **AND** image tabs do not contribute to the dirty-tab count

#### Scenario: Autosave targets the tab that was active when scheduled
- **WHEN** an autosave timer fires after the user has switched tabs
- **THEN** the autosave writes the document tab whose generation was captured at schedule time, not whichever tab is now active
- **AND** an image tab never becomes an autosave target

#### Scenario: Single-tab layout is unchanged
- **WHEN** only one content tab is open
- **THEN** no tab bar is rendered and the active document or image surface occupies the normal document-workspace area

### Requirement: Editor view modes
The editor SHALL provide four mutually exclusive view modes: Edit (also surfaced as "Source"), Visual Edit, Split Preview, and Read. Source mode SHALL show the Markdown source editing surface without the rendered preview pane. Visual Edit mode SHALL show a single WYSIWYG editing surface where Markdown constructs are presented as close to their rendered result as the editor can edit through an exact, lossless source mutation, with constructs that cannot yet be rendered tracked as WYSIWYG coverage gaps under the `WYSIWYG coverage roadmap` requirement. Split Preview mode SHALL show the Markdown source editing surface and rendered preview pane together, preserving the current live-preview workflow. Read mode SHALL show the rendered Markdown preview without the source editing pane and SHALL NOT allow editing through the rendered preview.

#### Scenario: Edit mode shows only source editing
- **WHEN** the active view mode is Edit (also surfaced as "Source")
- **THEN** the source editing surface is visible and accepts normal editing operations
- **AND** the rendered preview pane is not visible

#### Scenario: Visual Edit mode shows one editable visual surface
- **WHEN** the active view mode is Visual Edit
- **THEN** the editor shows a single WYSIWYG editing surface where Markdown constructs render close to their preview appearance while remaining editable
- **AND** constructs that cannot yet be rendered are tracked as WYSIWYG coverage gaps under the `WYSIWYG coverage roadmap` requirement

#### Scenario: Split Preview mode shows both panes
- **WHEN** the active view mode is Split Preview
- **THEN** the source editing surface and rendered preview pane are both visible
- **AND** edits in the source surface continue to update the preview through the existing derived Markdown state

#### Scenario: Read mode shows only rendered Markdown
- **WHEN** the active view mode is Read
- **THEN** the rendered preview pane is visible without a source editing pane
- **AND** editing through the rendered preview is not permitted

#### Scenario: Mode switching preserves document state
- **WHEN** the user switches between Edit, Visual Edit, Split Preview, and Read for an open document
- **THEN** the document text, dirty flag, cursor/selection, undo/redo history, editor scroll position, preview scroll position, and tab identity are preserved
- **AND** derived preview blocks, outline, stats, syntax highlighting, visual edit blocks, and cached text handles continue to follow the existing per-document-version cache rules

### Requirement: View mode switching shortcuts
The editor SHALL provide keyboard shortcuts for switching to each view mode directly, using platform-appropriate modifier conventions. The editor MAY also retain an existing shortcut that cycles through the view modes.

#### Scenario: Direct shortcut enters Edit mode
- **WHEN** the user presses the Edit mode shortcut
- **THEN** the active view mode becomes Edit
- **AND** status feedback identifies Edit mode

#### Scenario: Direct shortcut enters Visual Edit mode
- **WHEN** the user presses the Visual Edit mode shortcut
- **THEN** the active view mode becomes Visual Edit
- **AND** status feedback identifies Visual Edit mode

#### Scenario: Direct shortcut enters Split Preview mode
- **WHEN** the user presses the Split Preview mode shortcut
- **THEN** the active view mode becomes Split Preview
- **AND** status feedback identifies Split Preview mode

#### Scenario: Direct shortcut enters Read mode
- **WHEN** the user presses the Read mode shortcut
- **THEN** the active view mode becomes Read
- **AND** status feedback identifies Read mode

#### Scenario: Mode shortcuts follow platform conventions
- **WHEN** the editor runs on macOS versus Windows/Linux
- **THEN** the view mode shortcuts use the same `secondary` modifier convention as other application shortcuts

### Requirement: Source-backed Visual Edit mode
The editor SHALL provide a Visual Edit mode whose default presentation contract is WYSIWYG (what you see is what you get): every Markdown construct SHALL be presented as close to its rendered result as the editor can edit through an exact, lossless source mutation. `MarkdownDocument.text` SHALL remain the single canonical editable representation — Visual Edit is a presentation and editing contract over that text, not a parallel rendered document model. Every Visual Edit mutation SHALL flow through the existing source-mutation path (dirty-state, undo/redo, autosave, recovery, per-tab isolation), and SHALL NOT edit an inferred rendered tree. Constructs that the editor currently cannot present in rendered form are classified as **WYSIWYG coverage gaps** under the `WYSIWYG coverage roadmap` requirement, not as accepted end states; each gap SHALL show raw source only as a transitional measure until a future change closes it. Math SHALL be rendered while unfocused and SHALL reveal its complete authored delimiter group when focused; it SHALL NOT be mutated through an inferred rendered formula tree.

#### Scenario: Visual prose editing updates Markdown source
- **WHEN** the user edits visible prose inside a paragraph, heading, blockquote, or list item in Visual Edit mode
- **THEN** the corresponding Markdown source text is updated
- **AND** the document dirty flag and undo history are updated through the existing document mutation path

#### Scenario: Visual formatting actions remain source-backed
- **WHEN** the user applies bold, italic, inline code, link, image, heading, list, task list, blockquote, or fenced-code formatting in Visual Edit mode
- **THEN** the editor updates the underlying Markdown markers in `MarkdownDocument.text`
- **AND** switching to Source mode shows Markdown source that represents the visual result

#### Scenario: Focused syntax can be exposed for editing
- **WHEN** the cursor enters visually formatted inline content whose hidden Markdown syntax is needed for precise editing
- **THEN** the editor SHALL reveal the smallest complete source syntax group for that focused content (progressive-reveal WYSIWYG)
- **AND** the construct SHALL NOT be mutated through an ambiguous rendered-tree edit

#### Scenario: Unfocused math is rendered in Visual Edit
- **WHEN** valid inline, display, or fenced math is visible in Visual Edit and neither its source range nor delimiter group is focused
- **THEN** inline math appears as a baseline-aligned formula atom and display math appears as a typeset block
- **AND** the authored Markdown remains the canonical content

#### Scenario: Focused inline math reveals one complete source group
- **WHEN** the caret or a selection endpoint enters an inline math source range in Visual Edit
- **THEN** the complete byte-exact delimiter group is revealed as one editable source range
- **AND** unrelated prose in the same block remains rendered

#### Scenario: Focused display math uses a source edit island
- **WHEN** the user focuses `$$...$$` or fenced `math` content in Visual Edit
- **THEN** that formula presents an editable payload containing its exact authored syntax alongside the rendered formula
- **AND** moving focus away restores formula rendering without changing the document version

#### Scenario: Complex constructs use conservative edit islands
- **WHEN** the user focuses a construct that the `WYSIWYG coverage roadmap` classifies as an open gap (for example a front-matter region, an indented code block, an unclosed code fence, or a paragraph containing decoded HTML entities)
- **THEN** the editor SHALL show the authored source as a transitional editing affordance and SHALL classify the construct against the roadmap
- **AND** the construct SHALL NOT be mutated through an ambiguous rendered-tree edit
- **AND** the gap SHALL be tracked for closure by a future change that moves the construct into rendered or progressive-reveal WYSIWYG

#### Scenario: Visual-only interaction does not reparse unnecessarily
- **WHEN** the user moves the cursor, changes selection, hovers text, or focuses a rendered editor or transitional source view without changing document text
- **THEN** the document version SHALL remain unchanged
- **AND** derived Markdown caches SHALL NOT be invalidated

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

### Requirement: Preview pane free-range text selection and copy
When the rendered preview pane is visible (Split Preview or Read mode), the editor SHALL allow the user to select textual content with the pointer across one or more contiguous preview blocks in document order (for example a heading together with following paragraphs, or multiple list items) and copy the selected plain text to the system clipboard via Copy (menu or shortcut). Selection and copy in the preview SHALL NOT mutate the document text, dirty flag, undo/redo history, or derived Markdown caches. The preview SHALL remain non-editable: cut, paste, and typing MUST NOT apply to preview content. A non-empty preview selection SHALL take copy precedence over the source editor selection.

#### Scenario: Drag-select across multiple preview blocks
- **WHEN** the rendered preview pane is visible and the user drag-selects from text in one preview block into text in a later or earlier block in document order
- **THEN** the selection covers the contiguous textual content between the drag start and end (partial first and last runs, full runs in between)
- **AND** the selection is highlighted across those runs
- **AND** the document text and derived Markdown state are unchanged

#### Scenario: Drag-select within a single preview text run
- **WHEN** the rendered preview pane is visible and the user drag-selects text within a single preview text run
- **THEN** the selected range is highlighted in that run
- **AND** the document text and derived Markdown state are unchanged

#### Scenario: Copy free-range selection as plain text
- **WHEN** a non-empty multi-block or single-run preview selection exists and the user invokes Copy (menu or shortcut)
- **THEN** the selected plain text (joined across covered runs in document order) is written to the system clipboard
- **AND** the document text, dirty flag, and undo/redo history are unchanged

#### Scenario: Preview selection takes copy precedence
- **WHEN** a non-empty preview text selection exists and the source editor also has a selection
- **THEN** Copy uses the preview selection's plain text rather than the source editor selection

#### Scenario: Read mode allows free-range copy but not edit
- **WHEN** the active view mode is Read and the user selects preview text spanning multiple blocks and copies it
- **THEN** the clipboard receives the selected plain text
- **AND** interacting with the preview still does not mutate the document text

#### Scenario: Link click still works alongside free-range selection
- **WHEN** the user clicks a preview link without creating a meaningful text selection
- **THEN** the link opens as before
- **AND** a drag that creates a non-empty selection does not open the link

### Requirement: Preview pane context menu with multi-format copy
When the rendered preview pane is visible, the editor SHALL provide a right-click context menu on the preview with actions to copy the current preview selection as plain text, as Markdown source, and as an HTML fragment. The menu SHALL also offer Select All for preview textual content, and Copy Link Address when the right-click resolves to a link URL. Context-menu actions SHALL NOT mutate the document text, dirty flag, undo/redo history, or derived Markdown caches.

#### Scenario: Right-click opens the preview context menu
- **WHEN** the preview pane is visible and the user right-clicks inside it
- **THEN** a context menu appears at the pointer with the localized copy and selection actions

#### Scenario: Copy as Markdown from a multi-block selection
- **WHEN** a non-empty preview selection covering one or more blocks exists and the user chooses Copy as Markdown
- **THEN** the clipboard receives Markdown source corresponding to the selected region (derived from document source ranges for the covered blocks)
- **AND** the document remains unmodified

#### Scenario: Copy as HTML from a preview selection
- **WHEN** a non-empty preview selection exists and the user chooses Copy as HTML
- **THEN** the clipboard receives an HTML fragment for that selection
- **AND** the document remains unmodified

#### Scenario: Copy as Plain Text from the context menu
- **WHEN** a non-empty preview selection exists and the user chooses Copy as Plain Text
- **THEN** the clipboard receives the same plain text that Edit→Copy would produce for that selection

#### Scenario: Copy actions disabled without a selection
- **WHEN** the preview context menu is open and there is no non-empty preview selection
- **THEN** Copy as Plain Text, Copy as Markdown, and Copy as HTML are unavailable (disabled or omitted)
- **AND** Select All remains available

#### Scenario: Select All selects the full preview text
- **WHEN** the user chooses Select All from the preview context menu
- **THEN** the preview selection covers all textual preview content for the active document from the first run to the last

#### Scenario: Copy Link Address when right-clicking a link
- **WHEN** the user right-clicks a preview link and chooses Copy Link Address
- **THEN** the clipboard receives that link's URL
- **AND** the document remains unmodified

### Requirement: Preview pane text selection and copy
When the rendered preview pane is visible (Split Preview or Read mode), the editor SHALL allow the user to select textual content in the preview with the pointer and copy the selected plain text to the system clipboard. Selection and copy in the preview SHALL NOT mutate the document text, dirty flag, undo/redo history, or derived Markdown caches. The preview SHALL remain non-editable: cut, paste, and typing MUST NOT apply to preview content.

#### Scenario: Drag-select preview text
- **WHEN** the rendered preview pane is visible and the user drag-selects text within a preview text run (heading, paragraph, list item body, blockquote, code block body, table cell, or other textual preview content)
- **THEN** the selected range is highlighted in the preview
- **AND** the document text and derived Markdown state are unchanged

#### Scenario: Copy selected preview text
- **WHEN** a non-empty preview text selection exists and the user invokes Copy (menu or shortcut)
- **THEN** the selected plain text is written to the system clipboard
- **AND** the document text, dirty flag, and undo/redo history are unchanged

#### Scenario: Preview selection takes copy precedence
- **WHEN** a non-empty preview text selection exists and the source editor also has a selection
- **THEN** Copy uses the preview selection's plain text rather than the source editor selection

#### Scenario: Read mode allows copy but not edit
- **WHEN** the active view mode is Read and the user selects preview text and copies it
- **THEN** the clipboard receives the selected plain text
- **AND** interacting with the preview still does not mutate the document text

#### Scenario: Link click still works alongside selection
- **WHEN** the user clicks a preview link without creating a meaningful text selection
- **THEN** the link opens as before
- **AND** a drag that creates a non-empty selection does not open the link

### Requirement: Format menu heading depth follows preference
The editor SHALL expose heading formatting entries in the Format menu (in-window dropdown and native OS menu) from H1 through the configured maximum level. The default maximum level SHALL be 5 so H4 and H5 are visible without extra setup. When the maximum level is 6, H6 SHALL also appear alongside H1–H5 with the same behavior as existing heading actions.

#### Scenario: Default menus show H1 through H5
- **WHEN** Heading menu depth is H1–H5 (default)
- **THEN** the Format menu lists heading actions for H1, H2, H3, H4, and H5

#### Scenario: Extended menus show H1 through H6
- **WHEN** Heading menu depth is H1–H6
- **THEN** the Format menu lists heading actions for H1, H2, H3, H4, H5, and H6

#### Scenario: Heading actions apply the selected level
- **WHEN** the user triggers a heading action for level N from the Format menu
- **THEN** the editor applies `MarkdownFormat::Heading(N)` to the current selection or line

### Requirement: Heading keyboard shortcuts respect configured depth
The editor SHALL bind `Ctrl+4` and `Ctrl+5` (platform `secondary-4/5`) to Heading 4 and 5 by default, in addition to the existing H1–H3 shortcuts. When Heading menu depth is H1–H6, the editor SHALL also bind `Ctrl+6` to Heading 6. The keyboard shortcut reference SHALL list H4 and H5 by default and H6 only when Heading menu depth is H1–H6.

#### Scenario: Default shortcuts apply H4 and H5
- **WHEN** Heading menu depth is H1–H5 and the user presses the Heading 4 shortcut
- **THEN** the editor applies a level-4 heading to the current selection or line

#### Scenario: Shortcut reference documents extended headings conditionally
- **WHEN** the user opens the keyboard shortcut reference and Heading menu depth is H1–H6
- **THEN** the reference includes Heading 4, 5, and 6 shortcuts

### Requirement: Visual Edit inline formatting fidelity
Visual Edit SHALL render byte-exact supported inline formatting in prose blocks without exposing its Markdown delimiters while the construct is unfocused. Supported formatting SHALL include emphasis, strong emphasis, safely nested strong/emphasis combinations, strikethrough, inline code, links, highlight, superscript, subscript, backslash-escaped ASCII punctuation, and exactly recognized inline HTML in the supported subset. A backslash followed by an ASCII punctuation character SHALL render as the literal punctuation character with the backslash hidden as a marker. The supported inline-HTML subset SHALL consist of the exact unattributed style pairs `<em>`/`<i>`, `<strong>`/`<b>`, `<s>`/`<del>`/`<strike>`, `<code>`, `<mark>`, `<sub>`, and `<sup>`, plus the void line-break forms `<br>`, `<br/>`, and `<br />`; their tags SHALL be hidden markers whose styling composes with Markdown formatting, and `<br>` SHALL render as an authored line break inside the inline flow. Supported links SHALL include reference-style links (full `[text][label]`, collapsed `[label][]`, and shortcut `[label]` forms) whose definitions appear elsewhere in the document: Visual Edit SHALL resolve them against the document's link reference definitions, while definitions inside fenced code blocks SHALL NOT create links. Resolving document-scoped definitions SHALL preserve exact in-block source ranges — rendering and reveal mappings for the block's own content remain byte-identical to a full-document parse. Moving the caret or a selection endpoint into a supported formatted construct — including an escaped-character group or a supported inline-HTML element — SHALL reveal one safe containing source group for precise editing without converting unrelated inline content in the same block to raw Markdown. Constructs whose source/display mapping is malformed, crossing, or otherwise ambiguous — including backslash sequences or inline HTML outside the proven subset, decoded HTML entities, and angle-bracket autolink sources the link reveal validator cannot yet classify — SHALL be classified as WYSIWYG coverage gaps under the `WYSIWYG coverage roadmap` requirement and SHALL show raw source only as a transitional editing affordance until a future change closes the gap with a byte-exact projection.

#### Scenario: Default inline formatting paragraph stays visual
- **WHEN** the default welcome document is opened in Visual Edit mode and its Inline formatting paragraph is not focused
- **THEN** supported Markdown delimiters in that paragraph are hidden
- **AND** italic, bold, combined bold-and-italic, strikethrough, inline code, link, highlight, superscript, and subscript content is rendered with its corresponding visual style

#### Scenario: Reference-style link resolves against a document-level definition
- **WHEN** a prose block contains a reference-style link whose definition line appears in a different block of the same document
- **THEN** Visual Edit renders the link label with link styling and hides the reference brackets while unfocused, exactly as Split Preview and Read modes do
- **AND** moving the caret into the link reveals the complete local `[text][label]` source group for editing
- **AND** all in-block source ranges (runs, reveal groups, markers) are identical to those of a full-document parse

#### Scenario: Reference-style link forms all resolve
- **WHEN** a document defines a link reference and uses it via the full `[text][label]`, collapsed `[label][]`, or shortcut `[label]` form in Visual Edit
- **THEN** each form renders as a link rather than literal bracketed text

#### Scenario: Bracketed text inside fenced code does not become a link
- **WHEN** a fenced code block contains a line shaped like a link reference definition
- **THEN** that line does not register as a definition
- **AND** matching `[text][label]` prose elsewhere in the document remains literal text in Visual Edit

#### Scenario: Undefined reference remains literal
- **WHEN** a prose block contains `[text][label]` with no matching definition anywhere in the document
- **THEN** Visual Edit renders it as literal text, matching CommonMark behavior

#### Scenario: Nested formatting reveals one safe containing group
- **WHEN** the caret or a selection endpoint enters byte-exact nested strong/emphasis content in Visual Edit
- **THEN** the editor reveals one outermost containing Markdown source range without duplicating text
- **AND** source/display mappings remain monotonic and UTF-8 safe
- **AND** unrelated inline content in the same block remains rendered

#### Scenario: Extended inline markers remain source-backed
- **WHEN** the caret enters a valid highlight, superscript, or subscript construct in Visual Edit
- **THEN** the complete local delimiters are revealed for editing
- **AND** moving the caret away hides those delimiters and restores the visual style
- **AND** cursor-only reveal does not change the document version or invalidate cached visual blocks

#### Scenario: Escaped punctuation renders as literal text
- **WHEN** an unfocused prose block contains backslash-escaped ASCII punctuation such as `\*` or `\.`
- **THEN** the paragraph renders as normal prose showing the literal punctuation character, not a whole-block source island
- **AND** the backslash stays hidden while the rest of the paragraph remains rendered
- **AND** the rendering matches Split Preview and Read mode visible text

#### Scenario: Escaped construct reveals its authored group
- **WHEN** the caret or a selection endpoint moves into an escaped-character group such as `\*` (including the escaped-backslash form `\\`)
- **THEN** the complete authored backslash-plus-character source group is revealed for editing
- **AND** moving the caret away hides the backslash again and restores the literal rendering without changing the document version

#### Scenario: Escapes compose with Markdown formatting
- **WHEN** a prose block contains an escape inside other supported formatting, such as `**a \* b**`
- **THEN** the escaped character renders literally inside the styled construct with the backslash hidden
- **AND** entering the construct reveals one safe containing source group

#### Scenario: Inline HTML style pair renders with hidden tags
- **WHEN** an unfocused prose block contains an exact unattributed pair such as `text <em>em</em> more` or `a <strong>b</strong> c`
- **THEN** the paragraph renders as normal prose with the tagged content carrying the corresponding visual style
- **AND** the tags stay hidden and the block does not collapse into an HTML source island

#### Scenario: Inline HTML element reveals its complete source
- **WHEN** the caret or a selection endpoint moves into content between a supported inline-HTML tag pair
- **THEN** the complete element source — opening tag, content, and closing tag — is revealed as one group for editing
- **AND** moving the caret away hides the tags and restores the rendered form without changing the document version

#### Scenario: Inline `<br>` renders an authored line break
- **WHEN** an unfocused prose block contains a void `<br>`, `<br/>`, or `<br />` form
- **THEN** the paragraph renders the same stacked line-break layout it uses for authored hard breaks, without collapsing into an HTML source island
- **AND** caret activation of the tag reveals its authored source with pointer and keyboard resolution limited to the tag's safe source boundaries

#### Scenario: Unsupported inline HTML remains conservative
- **WHEN** a prose block contains inline HTML outside the supported subset — an unknown tag, a tag carrying attributes, an unpaired or crossing tag pair, or an HTML entity such as `&amp;`
- **THEN** Visual Edit preserves the whole-block source-backed transitional editing affordance and classifies the construct as a WYSIWYG coverage gap under the roadmap
- **AND** the editor does not guess a rendered-tree mutation for that content
- **AND** inline `<img>` tags keep their existing image-atom rendering and mixed-path behavior

#### Scenario: Angle-bracket autolinks are a tracked WYSIWYG gap
- **WHEN** a prose block contains an angle-bracket autolink such as `<https://example.com>` or `<user@example.com>`
- **THEN** Visual Edit keeps the paragraph on the source-backed transitional editing path because the link reveal validator only accepts bracketed link sources
- **AND** the construct is classified as a WYSIWYG coverage gap under the roadmap for closure by extending the link reveal validator

#### Scenario: Ambiguous inline syntax remains conservative
- **WHEN** a prose block contains malformed, crossing, or byte-inexact inline syntax whose visible text cannot be reconstructed byte-exactly from the authored slice
- **THEN** Visual Edit preserves a source-backed transitional editing affordance and classifies the construct as a WYSIWYG coverage gap under the roadmap
- **AND** the editor does not guess a rendered-tree mutation for that construct

### Requirement: Visual Edit whitespace activation
The system SHALL keep source-backed whitespace ranges available for exact caret mapping. In Visual Edit, a `Whitespace` row SHALL behave as a first-class empty line: it occupies the rendered body paragraph line height (one painted line per covered newline, floored at one line and capped at the existing pathological bound), presents an I-beam pointer, and accepts pointer placement onto an existing offset inside its source range. Clicking a whitespace row SHALL move the caret into that range and MUST NOT insert a newline or otherwise mutate the document text, version, dirty state, undo history, or derived Markdown caches. When the source caret owns a whitespace row — because the user clicked it, pressed Enter onto a new insertion line, or moved into it with keyboard navigation — Visual Edit SHALL present the same empty-paragraph-height layout plus a thin insertion caret line visually consistent with the caret in a paragraph or heading, and SHALL accept subsequent typed text at the exact source caret position. Visual Edit SHALL NOT wrap a whitespace row in a source-island box (border, padding, monospace styling, or differentiated background). Source islands SHALL remain reserved for blocks whose source has no rendered visual form (frontmatter, code, HTML, unsupported constructs) or for inline runs whose source/display mapping is ambiguous. Landing offsets SHALL lie inside the whitespace source range. For a single-newline gap between two rendered blocks, the caret SHALL land at `Whitespace.source_range.start` (the authored separator newline), not the first content byte of the following block.

#### Scenario: Clicking a blank line between headings places the caret without mutation
- **WHEN** the Visual Edit caret belongs to a rendered heading and the user clicks the blank-line `Whitespace` row between that heading and another heading
- **THEN** the caret moves onto an existing offset inside that whitespace range (`source_range.start` for a single-newline gap)
- **AND** the document text, version, dirty state, undo history, and derived Markdown cache identity remain unchanged
- **AND** the gap row presents an insertion caret

#### Scenario: Clicking a blank line between a heading and a paragraph places the caret without mutation
- **WHEN** the Visual Edit caret belongs to a rendered block and the user clicks the blank-line `Whitespace` row between a heading and a paragraph
- **THEN** the caret moves onto an existing offset inside that whitespace range
- **AND** the document text, version, dirty state, undo history, and derived Markdown cache identity remain unchanged
- **AND** the gap row becomes the caret-owning typing surface

#### Scenario: Typing after a gap click inserts at the existing newline
- **WHEN** the user clicks the blank-line row between `## [Unreleased]` and `## [16.1.7]` in a changelog-like document and types text
- **THEN** the typed bytes insert at the existing separator newline so a paragraph appears between the two headings
- **AND** the following heading’s first content byte is not consumed
- **AND** the edit does not insert an extra blank line beyond the newline that was already authored

#### Scenario: Structural Enter activates an insertion line
- **WHEN** the user presses Enter from a heading in Visual Edit and the structural edit creates a new source-backed insertion line
- **THEN** the owning visual row presents the caret and accepts subsequent typed text at the exact source position regardless of whether the parser retains the newline in the heading range

#### Scenario: Intentional source caret movement preserves whitespace editing
- **WHEN** keyboard navigation or reveal logic moves the source caret into an existing whitespace-only range
- **THEN** the owning whitespace row provides the source-backed editing affordance without recomputing the document's cached Markdown-derived state

#### Scenario: Whitespace row owning the caret renders a caret line, not a source island
- **WHEN** the source caret owns a whitespace row in Visual Edit — for example after clicking it, after creating a blank line by pressing Enter, or after pressing Down or Up onto an existing blank line
- **THEN** the row is rendered at empty-paragraph height with a thin insertion caret line and no border, padding, monospace styling, or differentiated background
- **AND** typed text is inserted into the canonical Markdown source at the caret position through the same dirty-state, undo/redo, autosave, and per-tab isolation paths as any other edit

#### Scenario: Whitespace row not owning the caret stays an empty line
- **WHEN** a whitespace row does not own the source caret
- **THEN** it still occupies empty-paragraph height and remains pointer-editable
- **AND** it does not paint an insertion caret until it owns the caret

### Requirement: Progressive Markdown marker reveal in Visual Edit
Visual Edit SHALL keep supported paragraph, heading, list-item, and blockquote content visually rendered while it is focused. When precise editing requires Markdown syntax, the editor SHALL reveal only the smallest complete inline syntax group whose source mapping is proven exact, while `MarkdownDocument.text` remains the canonical representation. Display-to-source and source-to-display mappings SHALL remain UTF-8-safe and monotonic for pointer placement, selection, keyboard navigation, platform text input, and IME caret geometry. Syntax whose mapping is nested, overlapping, byte-inexact, or otherwise ambiguous MUST use a conservative source-backed edit island.

#### Scenario: Focusing plain prose preserves visual rendering
- **WHEN** the user places the caret in plain text inside a supported visual paragraph, heading, list item, or blockquote
- **THEN** the block remains in its rendered visual style
- **AND** the entire block is not replaced by raw Markdown source

#### Scenario: Active inline syntax is revealed locally
- **WHEN** the caret enters exactly mapped strong, emphasis, strikethrough, or inline-code content in a supported visual block
- **THEN** the complete markers for that active inline construct are revealed together with its content
- **AND** other supported content in the same block remains visually rendered

#### Scenario: Active link exposes its destination
- **WHEN** the caret enters an exactly mapped inline link label or its hidden source syntax
- **THEN** the local link syntax, including its destination and optional title, becomes visible and editable
- **AND** editing it mutates the corresponding canonical Markdown source range

#### Scenario: Leaving a reveal group hides its markers without mutation
- **WHEN** the caret or selection endpoints leave a locally revealed syntax group without editing document text
- **THEN** that group returns to its rendered representation
- **AND** the document version, dirty state, undo history, and derived Markdown caches remain unchanged

#### Scenario: Selection remains source-accurate across hidden markers
- **WHEN** a Visual Edit selection crosses rendered runs separated by hidden Markdown markers
- **THEN** the visual highlight represents the selected canonical source content across projected segments
- **AND** replacement, copy, cut, and formatting actions operate on the exact source selection

#### Scenario: Keyboard navigation into a hidden marker reveals it
- **WHEN** source-based keyboard navigation moves the caret into a currently hidden marker range
- **THEN** the next Visual Edit render reveals the owning syntax group
- **AND** subsequent caret geometry and input use an identity-mapped visible source position

#### Scenario: Ambiguous inline syntax remains conservative
- **WHEN** an inline construct is nested, overlapping, escaped, transformed, or otherwise lacks a proven byte-exact mapping
- **THEN** Visual Edit uses a source-backed edit island for the affected block or construct
- **AND** it does not guess a rendered-tree mutation

### Requirement: Structure-aware block editing in Visual Edit
When Visual Edit is active, Enter and Backspace SHALL apply Markdown-aware structural transitions for supported headings, blockquotes, ordered and unordered lists, and task lists. Each transition SHALL be one canonical source edit integrated with the existing selection, dirty-state, undo/redo, autosave, recovery, cache invalidation, and per-tab isolation paths. Edit, Split Preview, and Read mode behavior SHALL remain unchanged except where they already share the same source helper.

#### Scenario: Enter after heading content starts a paragraph
- **WHEN** the Visual Edit caret is in a heading and the user presses Enter
- **THEN** the source is split at the caret without copying the heading prefix to the new line
- **AND** the following line renders as a paragraph unless its source explicitly contains another block marker

#### Scenario: Enter continues a non-empty list item
- **WHEN** the caret is in a non-empty ordered, unordered, or task-list item and the user presses Enter
- **THEN** the new source line receives the appropriate list prefix
- **AND** ordered numbering advances while a new task-list item starts unchecked

#### Scenario: Enter continues or exits a blockquote
- **WHEN** the caret is in a non-empty blockquote line and the user presses Enter
- **THEN** the new source line continues the blockquote prefix
- **AND WHEN** the current blockquote line contains only its prefix and the user presses Enter
- **THEN** the empty prefix is removed and the caret exits the blockquote

#### Scenario: Enter on an empty list item exits the list
- **WHEN** a list or task-list line contains only its structural prefix and the user presses Enter
- **THEN** the empty prefix is removed instead of creating another empty item
- **AND** subsequent input produces a plain paragraph at that position

#### Scenario: Backspace at visible content start demotes the block
- **WHEN** the caret is collapsed at the first visible content position of a top-level heading, blockquote, list item, or task-list item and the user presses Backspace
- **THEN** the complete structural prefix is removed in one edit
- **AND** the remaining content becomes the corresponding less-structured or plain block without partial marker corruption

#### Scenario: Backspace at nested list start outdents first
- **WHEN** the caret is collapsed at the first visible content position of a nested list or task-list item and the user presses Backspace
- **THEN** one indentation level is removed while preserving the item prefix
- **AND** another Backspace at the resulting top-level boundary can remove the prefix

#### Scenario: Structural edit is one undoable mutation
- **WHEN** Visual Edit performs a structural Enter or Backspace transition
- **THEN** one Undo restores the prior Markdown source and selection
- **AND** Redo reapplies the same transition through the existing history path

### Requirement: Affinity-aware Visual Edit caret
Visual Edit SHALL preserve which canonical source side owns a collapsed caret when hidden Markdown syntax maps multiple source positions to one display boundary. Pointer placement, Left/Right navigation, local marker reveal, and subsequent text input SHALL resolve that boundary consistently without corrupting or silently crossing inline formatting.

#### Scenario: Pointer placement at a hidden marker boundary is deterministic
- **WHEN** the user clicks a display boundary shared by formatted content and hidden opening or closing syntax
- **THEN** Visual Edit records a deterministic upstream or downstream caret affinity together with the canonical source offset
- **AND** repainting the unchanged projection preserves the same visual caret side

#### Scenario: Arrow navigation traverses a revealed delimiter
- **WHEN** local Markdown delimiters are revealed and the user presses Left or Right across an opening or closing delimiter
- **THEN** the caret advances through the corresponding UTF-8-safe source boundaries in the requested direction
- **AND** the caret does not stall or jump to an unrelated inline run

#### Scenario: Typing at a formatted-span boundary respects affinity
- **WHEN** the caret is visually collapsed at the start or end boundary of formatted content and the user types
- **THEN** the insertion occurs at the canonical source side represented by the current affinity
- **AND** text is not unintentionally included in or excluded from the formatted span

#### Scenario: Unambiguous movement clears stale affinity
- **WHEN** the caret moves to a source/display position with one exact mapping or the document version changes
- **THEN** stale boundary affinity is cleared or revalidated against the new projection
- **AND** source offsets remain clamped to valid UTF-8 boundaries

### Requirement: Layout-aware Visual Edit navigation
When Visual Edit is active, vertical and line-boundary navigation SHALL follow the painted visual layout rather than only logical Markdown source lines. Up/Down and their selection variants SHALL retain a preferred horizontal coordinate across wrapped lines and adjacent visual blocks, while Home/End SHALL target the active painted line in rendered content. Vertical navigation SHALL treat a blank-line (`Whitespace`) row as a navigation stop: moving Up from the lower rendered block and moving Down from the upper rendered block SHALL both land on an existing offset inside the gap row so the user can type into that authored blank line from either direction. A subsequent vertical move SHALL continue into the rendered block on the far side, or walk additional painted lines inside a multi-line whitespace row, while preserving the preferred horizontal coordinate. Leading and trailing blank lines at the document edge SHALL remain reachable the same way instead of becoming a dead no-op.

#### Scenario: Up and Down traverse wrapped visual lines
- **WHEN** a rendered paragraph or other editable visual block wraps onto multiple painted lines
- **AND** the user presses Up or Down
- **THEN** the caret moves to the closest valid source-backed position on the adjacent painted line
- **AND** it does not skip directly to the previous or next logical Markdown line

#### Scenario: Vertical navigation retains preferred horizontal position
- **WHEN** the user presses Up or Down repeatedly across painted lines with different lengths
- **THEN** Visual Edit retains the initial preferred horizontal coordinate
- **AND** each target is the closest valid caret position on that line

#### Scenario: Vertical navigation crosses visual blocks
- **WHEN** Up or Down moves past the first or last painted line of the active visual block
- **THEN** the caret moves to the closest source-backed position in the adjacent visual block
- **AND** a virtualized target row is revealed before the pending movement is completed

#### Scenario: Vertical navigation lands on a blank-line row between content blocks
- **WHEN** the user presses Up from a paragraph whose rendered block above is separated by a blank-line `Whitespace` gap row (for example a heading above, paragraph below)
- **OR** the user presses Down from a heading whose rendered block below is separated by a blank-line gap row
- **THEN** the caret lands on an existing offset inside the gap row (`Whitespace.source_range.start` for a single-newline gap)
- **AND** the gap row becomes the caret-owning row and accepts subsequent typed text at that source position through the standard source-backed input path
- **AND** the resolved target does not land on the start offset of the lower rendered block when that byte is outside the whitespace range
- **AND** the preferred horizontal coordinate is retained across the gap-row crossing

#### Scenario: A second vertical move continues past the gap row
- **WHEN** the caret already owns a blank-line gap row and the user presses Up (or Down) again
- **THEN** the caret moves into the rendered block on the far side of the gap, or onto the next painted line if the whitespace row covers multiple newlines
- **AND** the preferred horizontal coordinate is retained across the crossing

#### Scenario: Up from the start of a paragraph whose line above is a heading
- **WHEN** the caret is at the first source offset of a paragraph (paragraph start) and the user presses Up
- **AND** the block immediately above is a blank-line gap row
- **THEN** the caret moves onto the gap row instead of staying at the paragraph start or jumping into the heading
- **AND** subsequent typed text inserts at the gap row's source position

#### Scenario: A blank line is reachable by arrows as well as click and Enter
- **WHEN** the user wants to type into an existing blank line between two rendered blocks
- **THEN** the caret reaches that `Whitespace` row by clicking it, by pressing Up or Down onto it, or by pressing Enter, and the row becomes the caret-owning row that accepts typed text at its source position
- **AND** a single Up or Down from an adjacent content block parks on the blank line instead of skipping it

#### Scenario: A vertical move reaches a leading or trailing blank line at the document edge
- **WHEN** the only rows beyond the active block in the move direction are blank-line gap rows up to the start or end of the document
- **THEN** the move lands on an existing offset inside the gap row (`Whitespace.source_range.start` for a single-newline gap) instead of becoming a dead no-op

#### Scenario: Selection navigation uses visual targets
- **WHEN** the user invokes Select Up or Select Down in Visual Edit
- **THEN** the selection head uses the same layout-aware target as ordinary vertical movement
- **AND** the canonical source selection remains normalized and UTF-8 safe

#### Scenario: Home and End use the painted line in rendered content
- **WHEN** the Visual Edit caret is in a wrapped rendered line and the user presses Home or End
- **THEN** the caret moves to the first or last valid source-backed position of that painted line
- **AND** explicit source islands retain source-line Home/End behavior

### Requirement: Visual Edit IME composition fidelity
Visual Edit SHALL treat the active IME marked range as first-class projection and rendering state. The marked source SHALL remain visibly identified, precisely mapped, and correctly positioned for the platform candidate window throughout composition, including UTF-16 input containing CJK text, emoji, or combining characters.

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

#### Scenario: UTF-16 composition remains UTF-8 safe
- **WHEN** IME replacement or selection ranges include CJK text, emoji, or combining characters
- **THEN** boundary conversion, projection, and marked-range painting resolve to valid canonical UTF-8 boundaries
- **AND** no partial code point is inserted, selected, or underlined

### Requirement: Semantic text-input undo grouping
The editor SHALL group compatible contiguous text input into semantic undo entries while preserving atomic boundaries for composition, selection replacement, paste, formatting, structural commands, table commands, mode/tab changes, and explicit undo/redo. Grouping SHALL remain isolated per document tab and SHALL preserve exact source and selection restoration.

#### Scenario: Contiguous typing coalesces within the capture window
- **WHEN** consecutive ordinary text insertions occur within the configured coalescing window at the preceding collapsed caret with no intervening boundary
- **THEN** one Undo removes the compatible typing group
- **AND** one Redo restores the complete group and its resulting selection

#### Scenario: Atomic command terminates a typing group
- **WHEN** paste, formatting, structural Enter/Backspace, a table command, selection replacement, mode/tab change, or another atomic command follows ordinary typing
- **THEN** the atomic command and preceding typing are separate undo entries

#### Scenario: Caret discontinuity terminates a typing group
- **WHEN** the caret or selection moves so the next insertion is not contiguous with the preceding text input
- **THEN** the next input starts a new undo group
- **AND** Undo restores each location independently

#### Scenario: Undo grouping is isolated per tab
- **WHEN** the user types in one document tab, switches tabs, and edits another document
- **THEN** each tab retains its own pending group and undo/redo history
- **AND** switching tabs cannot merge entries or restore source in the wrong document

### Requirement: Stable source-mapped visual block identity
Every derived Visual Edit block SHALL carry an opaque, non-persisted identity that remains stable across document versions only when the block is proven to descend unchanged from the same source block. Identity SHALL be independent from the block's current byte range and SHALL NOT replace canonical source ranges for editing.

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
- **AND** unchanged row height and scroll anchoring state remain reusable

#### Scenario: Identity and incremental cache remain ephemeral
- **WHEN** a document is saved, reopened, recovered, cloned for undo, or replaced wholesale
- **THEN** visual identities and incremental region caches are rebuilt rather than persisted
- **AND** Markdown file contents and undo snapshot formats remain unchanged

### Requirement: Mixed Markdown images stay inline with adjacent prose
When a paragraph, heading, quoted paragraph, or list item contains a Markdown image together with any other prose in the same construct, Visual Edit, Read mode, and Split Preview SHALL present the image as an inline atom on the same visual line as the adjacent text (wrapping only when the line does not fit). The authored `![alt](url)` bytes SHALL belong only to that atom. Those surfaces SHALL NOT stack the image on its own row above leftover prose, SHALL NOT paint the complete image syntax as a source island under the preview, and SHALL NOT leak alt text or the destination URL as ordinary copy. Image-only paragraphs and images separated by a blank line remain block-level image rows with the existing image presentation.

#### Scenario: Leading same-line image plus trailing prose

- **WHEN** the document contains a paragraph of the form `![alt](url)trailing text`
- **THEN** Read mode and Split Preview show the image and the trailing text on the same line
- **AND** Visual Edit shows the same inline layout in one paragraph row
- **AND** the complete authored `![alt](url)` syntax does not appear as a source island or as visible copy under the preview while the atom is unfocused

#### Scenario: Text surrounding an image on one line

- **WHEN** the document contains `text ![alt](url) more`
- **THEN** Visual Edit, Read mode, and Split Preview keep leading text, the image atom, and trailing text in one prose row
- **AND** no row is force-marked as an unsupported source island due to range overlap

#### Scenario: Heading, quote, and list item keep the parent construct

- **WHEN** a heading, a blockquote paragraph, or a list item starts with or contains a mixed Markdown image and trailing prose
- **THEN** the image stays an inline atom inside that heading, quoted paragraph, or list item
- **AND** a list item does not emit a second bullet or a continuation paragraph
- **AND** quoted rows keep the same quote boundary

#### Scenario: Image-only and blank-line-separated images stay block-level

- **WHEN** Visual Edit, Read mode, or Split Preview displays a paragraph that is only a Markdown image, or a prose paragraph separated from an image by a blank line
- **THEN** the image still renders as a block-level image row
- **AND** the prose paragraph (when present) remains a separate row whose source range does not overlap the image

### Requirement: Maintained Visual Edit support classification
The repository SHALL maintain a current Visual Edit WYSIWYG coverage matrix that classifies every user-visible Markdown construct into exactly one of three classes: **rendered WYSIWYG** (the construct is shown in its rendered form, including dedicated field/payload editors for code, math, diagrams, images, and tables whose editors ARE the rendered form), **progressive-reveal WYSIWYG** (the construct is rendered by default and reveals its smallest complete source syntax group when the caret enters it — inline formatting, links, inline math, structural prefixes), or **WYSIWYG coverage gap** (the construct currently shows raw source and is tracked under the `WYSIWYG coverage roadmap` for closure by a future change). The matrix SHALL name the canonical editable range and the verification evidence for each rendered/reveal class, and SHALL name the roadmap priority and implementation seam for each gap. The matrix SHALL agree with the stable requirements and the implemented `VisualBlock`/`VisualBlockEditor` behavior.

#### Scenario: Contributor evaluates current WYSIWYG coverage
- **WHEN** a contributor reads the Visual Edit WYSIWYG coverage matrix
- **THEN** it distinguishes rendered WYSIWYG constructs (prose, code, math, diagrams, images, tables, task lists, footnote definitions and references, blockquotes, alerts, rules, HTML blocks), progressive-reveal WYSIWYG constructs (inline formatting, links, inline math, escaped punctuation, supported inline HTML, structural prefixes, heading attributes), and open WYSIWYG gaps (decoded entities, front matter, indented code, unclosed fences, reference-style images, malformed tables, unsupported inline-HTML forms, autolinks, task-list checkbox interaction, definition lists, empty list items)
- **AND** it explains that canonical Markdown remains the single persisted representation and that no construct is edited through a parallel rendered tree

#### Scenario: A new visual block behavior is proposed
- **WHEN** a proposal changes how a Markdown construct is presented or edited in Visual Edit
- **THEN** the proposal selects one of the three coverage classes for the construct
- **AND** if the proposal moves a construct out of the gap class, it updates the matrix and the `WYSIWYG coverage roadmap`
- **AND** implementation and documentation cannot be considered complete until the matrix and invariant evidence are updated

### Requirement: Rendered math preserves selection, mapping, and copy
In Split Preview, Read, and Visual Edit, rendered inline math SHALL participate in prose layout as a single measured atom aligned to the surrounding text baseline, and display math SHALL participate as a source-mapped block. Pointer hit testing and selection SHALL resolve math to its byte-exact authored source boundaries rather than internal rendered glyphs. Copying a selection containing math as plain text or Markdown SHALL preserve the complete authored math syntax in document order; copying as HTML SHALL use the same safe static-math semantics as HTML export.

#### Scenario: Inline math aligns and wraps atomically
- **WHEN** a prose line contains text before and after inline math
- **THEN** the formula baseline aligns with the surrounding text and participates in line wrapping as one indivisible atom
- **AND** adjacent text retains its source mapping

#### Scenario: Drag selection crosses a formula
- **WHEN** the user drag-selects preview content from text before an inline formula to text after it
- **THEN** the selection covers the complete formula atom and never a partial internal glyph range
- **AND** no document or derived-cache state is mutated

#### Scenario: Source-preserving copy includes delimiters
- **WHEN** a preview or Visual Edit selection containing math is copied as plain text or Markdown
- **THEN** the clipboard includes the complete authored `$...$`, `$$...$$`, or fenced `math` syntax at that source position
- **AND** the payload is not replaced by a Unicode approximation

#### Scenario: Formula hit testing maps to safe boundaries
- **WHEN** the user clicks the leading or trailing half of an unfocused inline formula in Visual Edit
- **THEN** the caret resolves to the corresponding source boundary or activates the complete source-backed group
- **AND** it is never placed inside an unrepresented rendered glyph tree

#### Scenario: Read mode remains non-editable
- **WHEN** the user selects or copies a rendered formula in Read mode
- **THEN** source-preserving copy is available
- **AND** typing, cut, paste, or pointer interaction cannot mutate the document

### Requirement: Visual Edit SHALL provide selection-contextual formatting controls
When Visual Edit owns a non-empty, exactly source-mapped text selection, Markion SHALL present contextual controls for strong emphasis, emphasis, inline code, and link editing. Invoking a control SHALL use the existing canonical Markdown mutation, semantic undo, selection, autosave, and exact UTF-8 source paths. Merely showing, moving, or dismissing the controls SHALL NOT change document state or invalidate derived caches.

#### Scenario: Selection toolbar formats visual text
- **WHEN** the user selects exactly mapped prose in Visual Edit and invokes Bold, Italic, or Inline Code from the contextual controls
- **THEN** the corresponding canonical Markdown markers are changed through one semantic command
- **AND** one Undo restores the prior source and selection

#### Scenario: Ambiguous selection stays conservative
- **WHEN** a selection crosses an ambiguous or source-island boundary
- **THEN** Markion does not present an unsafe contextual mutation for that range
- **AND** raw source editing remains available

### Requirement: Links SHALL have an exact source-backed visual editor
Creating or focusing an exactly mapped inline link SHALL provide a visual editor for label, URL, and optional title. Submitting the editor SHALL serialize one valid inline Markdown link and apply it as one source mutation. Canceling or changing focus SHALL leave the authored source byte-for-byte unchanged. Reference-style, malformed, or crossing links SHALL retain conservative source editing.

#### Scenario: Selected text creates a link
- **WHEN** the user selects exactly mapped visual prose, opens the link editor, enters a URL and optional title, and confirms
- **THEN** the selected text becomes the link label in one canonical-source mutation
- **AND** the resulting selection and source ranges remain UTF-8 safe

#### Scenario: Existing inline link is edited
- **WHEN** the caret is within an exactly mapped inline link and the user changes its URL or title
- **THEN** the complete link source is replaced once while preserving its visible label unless edited
- **AND** one Undo restores the complete prior link and selection

#### Scenario: Link edit is canceled
- **WHEN** the visual link editor is dismissed without confirmation
- **THEN** document text, version, dirty state, selection, undo history, and derived cache identity remain unchanged

### Requirement: Visual Edit SHALL provide a complete slash-command block palette
When a collapsed Visual Edit caret is on a line containing only optional indentation and a slash query, Markion SHALL show a localized, filtered command palette. The palette SHALL provide Text, Heading 1 through Heading 6, Bulleted List, Numbered List, Task List, Quote, Code Block, Divider, and Table commands. Up and Down SHALL change the active result, Enter SHALL apply it, Escape SHALL close the palette without changing source, and pointer selection SHALL apply the same command. Confirmation SHALL replace the slash query through one canonical UTF-8-safe source edit.

#### Scenario: Slash query filters commands
- **WHEN** the user types `/hea` on an otherwise empty Visual Edit block
- **THEN** the palette shows the matching localized heading commands
- **AND** typing or navigating the palette does not create a parallel document value

#### Scenario: Keyboard confirmation is one edit
- **WHEN** the user selects Heading 2 with the keyboard and presses Enter
- **THEN** the slash query becomes an H2 Markdown block with the caret at its editable content position
- **AND** one Undo restores the exact slash query and selection

#### Scenario: Escape preserves canonical source
- **WHEN** the slash palette is open and the user presses Escape
- **THEN** the palette closes without changing document version, source, selection, history, or derived-cache identity

#### Scenario: Stale slash target is rejected
- **WHEN** the document version or query range changes before a palette command is confirmed
- **THEN** the palette closes without guessing a mutation

### Requirement: Visual Edit SHALL support exact block transformations and operations
A supported focused Visual Edit block SHALL expose contextual operations to turn it into Text, Heading 1 through Heading 6, Bulleted List, Numbered List, Task List, Quote, or Code Block, and to Duplicate or Delete it. The contextual block-operation menu SHALL render in an overlay above all Visual Edit document rows and media, SHALL remain anchored near its invoking control within the usable viewport, and SHALL keep every command reachable when space is constrained. Showing, positioning, scrolling within, or dismissing the menu SHALL NOT change canonical source, document version, history, or derived-cache identity. Each operation SHALL validate current document version, block identity, and exact source ownership; it SHALL perform one canonical source mutation with one undo entry and preserve unrelated bytes, line endings, dirty state, autosave/recovery behavior, tab isolation, and cache invariants.

#### Scenario: Heading turns into a task item
- **WHEN** the user transforms an exactly mapped heading into a Task List block
- **THEN** only the proven structural marker is replaced with canonical unchecked-task Markdown
- **AND** inline authored content and UTF-8 text remain byte-identical

#### Scenario: Code block turns into text
- **WHEN** a closed exactly mapped fenced code block is transformed to Text
- **THEN** its payload becomes paragraph source and the fence metadata is removed in one edit
- **AND** an unclosed or ambiguous fence is not transformed speculatively

#### Scenario: Duplicate and delete are atomic
- **WHEN** the user duplicates or deletes a supported block
- **THEN** the complete exact block source and deterministic separator whitespace are duplicated or removed
- **AND** one Undo restores the prior source and selection

#### Scenario: Stale or ambiguous transform is rejected
- **WHEN** a block event carries a stale version/identity/range or the source ownership overlaps an ambiguous nested structure
- **THEN** no source, history, document version, or cache identity changes
- **AND** complete source editing remains available

#### Scenario: Block menu overlays later visual content
- **WHEN** the user opens a supported block's operation menu where its bounds overlap following headings, formatted prose, an image, or another Visual Edit row
- **THEN** the complete menu background and commands paint above the overlapping document content
- **AND** underlying document content cannot visually obscure the menu or receive pointer actions within its bounds

#### Scenario: Block menu stays reachable near viewport edges
- **WHEN** the user opens the block-operation menu with insufficient space below or beside its invoking control
- **THEN** the menu flips or is constrained within the usable viewport
- **AND** overflow commands remain reachable through menu-local scrolling without scrolling the document

#### Scenario: Block menu dismissal is presentation-only
- **WHEN** the user dismisses an open block-operation menu with Escape, an outside action, document scrolling, a tab or mode change, or stale-target invalidation
- **THEN** the menu closes without changing canonical Markdown, document version, selection, history, dirty state, or derived-cache identity

### Requirement: Visual Edit SHALL support source-safe block reordering
Supported non-overlapping Visual Edit blocks SHALL be reorderable through Move Up, Move Down, and a drag grip with before/after drop targets. All reorder paths SHALL use the same exact source-unit operation, SHALL preserve the moved block bytes and deterministic separator whitespace, and SHALL create one undo entry. Nested list items, quote-group leaves, overlapping ranges, and stale targets SHALL not expose or accept guessed reordering.

#### Scenario: Block moves with button action
- **WHEN** the user invokes Move Down on a supported paragraph before another supported block
- **THEN** the two source units exchange order without altering either block's authored bytes
- **AND** selection follows the moved block and one Undo restores the previous order

#### Scenario: Drag uses the same reorder semantics
- **WHEN** the user drags a supported block grip to a valid before or after target
- **THEN** the same canonical source result is produced as the corresponding button moves
- **AND** drag movement before drop does not mutate source or document version

#### Scenario: Unsafe reorder is unavailable
- **WHEN** the focused row is nested, part of an overlapping quote group, or lacks a complete exact source unit
- **THEN** reorder controls are disabled or absent and drops are ignored
- **AND** source mode remains the lossless fallback

### Requirement: Visual Edit renders HTML images
Visual Edit SHALL present raw-HTML images the same way Read mode does wherever Read mode renders them, and SHALL NOT collapse prose blocks into raw-source islands solely because they contain image tags. Standalone raw-HTML blocks containing `<img>` SHALL render read-only through the shared HTML-parts pipeline (text, images, tables) with the existing focused source-island editing affordance. Inline `<img>` tags inside paragraphs, headings, list items, blockquote leaves, and footnote text SHALL render as inline image atoms loaded through the same image pipeline as preview (workspace-relative paths, remote URLs, and data URIs), while the surrounding prose remains rendered and editable. Each inline image atom SHALL be source-backed: entering its byte-exact authored `<img>` tag range with the caret or a selection endpoint SHALL reveal the complete authored tag as one editable source run, and leaving the range SHALL restore the rendered atom without changing the document version. Prose blocks whose only inline HTML consists of complete `<img>` tags SHALL NOT use a whole-block HTML source island. When a prose block mixes `<img>` tags with other inline HTML (for example `<a href=…>` wrappers, `<br>`, or `<em>…</em>`), the image atoms SHALL still render and the non-image inline HTML SHALL appear as byte-exact conservative source fragments in the same mixed layout; the block SHALL NOT collapse into a whole-block source island as long as it carries at least one inline image atom. Images inside GFM table cells SHALL present the flattened alt/URL text exactly as Read mode does.

#### Scenario: Standalone HTML image block renders
- **WHEN** an unfocused Visual Edit document contains a raw-HTML block such as `<p align="center"><img src="logo.svg" alt="Logo"></p>`
- **THEN** the block renders through the shared HTML-parts pipeline showing the image and honoring centering
- **AND** focusing the block presents the existing conservative source island for editing its raw HTML

#### Scenario: Inline HTML image renders inside prose
- **WHEN** an unfocused Visual Edit paragraph, heading, list item, or blockquote line contains text and one or more complete `<img>` tags
- **THEN** each tag renders as an inline image atom between the surrounding rendered prose runs
- **AND** the block does not present a whole-block raw-source island

#### Scenario: Focused inline image reveals its exact source
- **WHEN** the caret or a selection endpoint enters the authored `<img …>` source range of an inline image atom
- **THEN** the complete byte-exact tag is revealed as one editable source run
- **AND** moving the caret out restores the rendered atom without a document-version change

#### Scenario: Mixed inline HTML renders images beside conservative source fragments
- **WHEN** a prose block mixes one or more `<img>` tags with other inline HTML such as `<a href=…>` wrappers, `<br>`, or `<em>…</em>`
- **THEN** the block renders each image atom in the mixed layout
- **AND** the non-image inline HTML appears as byte-exact conservative source fragments alongside the atoms
- **AND** the block does not collapse into a whole-block source island while it carries at least one inline image atom

#### Scenario: Other inline HTML keeps the conservative fallback
- **WHEN** a prose block contains inline HTML but no `<img>` tag (for example only `<br>` or `<em>…</em>`)
- **THEN** the block keeps the whole-block HTML source-island presentation
- **AND** no partial rendering mutates or misrepresents the authored source

#### Scenario: HTML image in a table cell matches Read mode
- **WHEN** a GFM table cell contains a complete `<img>` tag and the table contains no other inline HTML
- **THEN** the table renders with the cell showing the flattened alt/URL text as Read mode does
- **AND** the table does not collapse into a whole-table source island

#### Scenario: Inline HTML images share the preview image lifecycle
- **WHEN** an inline HTML image is visible in Visual Edit
- **THEN** its URL is claimed, preloaded, and evicted through the same preview image cache lifecycle as block-level images
- **AND** pending and failed loads present the same placeholders as Read mode

### Requirement: Document-ordered block stream for list-nested blocks

When a list item contains nested block constructs (fenced code blocks, tables, blockquotes, HTML blocks), the parsed preview block stream SHALL present the list item and each nested block as separate blocks in document (source) order: the list item's text content appears before any block nested inside it. A list item block's source range SHALL NOT swallow the source range of a nested block that is also emitted as its own block; consumers that assume monotonically ordered, disjoint block source ranges SHALL NOT encounter an overlap from this pattern.

#### Scenario: List item with trailing nested fenced code block

- **WHEN** a list item's text is followed by a fenced code block indented to nest inside that same item
- **THEN** the parsed block stream contains the list item block before the code block
- **AND** the list item's source range ends no later than the nested code block's source range start

#### Scenario: Multiple list items each with nested code

- **WHEN** several sibling list items each contain a nested fenced code block
- **THEN** the block stream alternates item, code block, item, code block in source order
- **AND** reading mode renders each code block below the bullet it belongs to

#### Scenario: List items without nested blocks are unaffected

- **WHEN** a list contains only plain or inline-formatted items
- **THEN** block variants, content, ordering, and source ranges are unchanged from CommonMark event order

### Requirement: Visual Edit list items with nested fenced code blocks

In Visual Edit mode, a list item containing a nested fenced code block SHALL render the item's text as one normal, directly editable list row and the nested code block as one source-backed code editor row, in source order. Neither the item text nor the code content SHALL appear twice on screen, and neither SHALL fall back to a conservative raw-source box solely because of the nesting structure.

#### Scenario: Nested fence renders item row plus code editor row

- **WHEN** Visual Edit displays a list item whose indented continuation contains a fenced code block
- **THEN** the item's bullet and inline content render as a normal editable list row
- **AND** the fenced code block renders below it as the code editor row used for top-level fences
- **AND** no raw `- `, link syntax, or literal fence markers are shown for either row

#### Scenario: No duplicated content

- **WHEN** Visual Edit displays the document region spanning such a list item and its nested code block
- **THEN** every source byte of the region is owned by exactly one visual row
- **AND** no row is force-marked as an unsupported source island due to range overlap

#### Scenario: Editing either row stays source-backed

- **WHEN** the user edits the item text row or the nested code payload in Visual Edit
- **THEN** the edit applies to the canonical Markdown source through the existing mutation paths
- **AND** the other row's rendered content remains intact

### Requirement: Visual Edit mixed-prose line breaks
When Visual Edit lays out a prose row as mixed fragments (because the row contains a link or footnote navigation icon, an inline math atom, or an inline HTML image), authored soft breaks and hard breaks inside that row SHALL still start a new visual line, matching Read and Split Preview for the same source. Intra-line wrapping of long prose, progressive syntax reveal, and source-backed editing SHALL remain unchanged. Interaction-only layout grouping SHALL NOT change document version or invalidate per-version derived caches.

#### Scenario: Consecutive source lines with a link stay stacked
- **WHEN** a paragraph (or heading / list item / quoted leaf) is written as consecutive source lines with no blank separator, at least one of those lines contains a Markdown link, and Visual Edit is active
- **THEN** each authored line renders on its own visual row rather than joining into a single line
- **AND** the link keeps its rendered label and navigation icon on the line that owns the link

#### Scenario: Hard breaks still break in mixed layout
- **WHEN** a mixed-fragment prose row contains a Markdown hard break (two trailing spaces or a backslash before the newline)
- **THEN** the text after that break renders on the following visual row

#### Scenario: Single-line mixed prose is unchanged
- **WHEN** a mixed-fragment prose row contains no authored line break
- **THEN** its fragments continue to wrap as one flowing paragraph
- **AND** navigation icons and inline atoms stay on that same flow

### Requirement: WYSIWYG coverage roadmap
The repository SHALL maintain, as part of the Visual Edit WYSIWYG coverage matrix, a prioritized roadmap of every Markdown construct that is currently classified as a WYSIWYG coverage gap. The roadmap SHALL name, for each gap, the construct, its current rendering (transitional source view), its target WYSIWYG class (rendered or progressive-reveal), its priority, its rough implementation effort, and the implementation seam in the existing code. The roadmap SHALL be closed incrementally by future changes, each of which SHALL move one or more constructs out of the gap class and update this roadmap. The initial roadmap SHALL include at minimum the following primary gaps in priority order: (1) decoded HTML entities in prose blocks (for example `&amp;`), (2) front matter (an editing form for YAML `---` regions, and detection of TOML/JSON forms), and (3) indented code blocks. The roadmap SHALL also track secondary gaps including unclosed or malformed fenced code, reference-style and malformed inline images, malformed tables, unsupported inline-HTML forms and angle-bracket autolinks in prose, task-list checkbox click interaction, GFM definition lists, empty list items, and math render-failure states.

#### Scenario: Primary gaps are tracked with priority and effort
- **WHEN** a contributor reads the WYSIWYG coverage roadmap
- **THEN** the current primary gaps (decoded HTML entities, front matter, indented code blocks) are listed with priority, effort, target class, and implementation seam
- **AND** each primary gap points at the source location of the current transitional source-view rendering

#### Scenario: Closing a gap updates the roadmap
- **WHEN** a future change implements WYSIWYG rendering for a construct that the roadmap tracks as a gap
- **THEN** that change's spec delta moves the construct out of the gap class in the `Maintained Visual Edit support classification` matrix and removes it from this roadmap
- **AND** the change's proposal cites this roadmap requirement as its motivation

#### Scenario: Closed gaps do not regress
- **WHEN** a construct previously tracked as a gap has been implemented as rendered or progressive-reveal WYSIWYG (for example escaped punctuation, the supported inline-HTML subset, standalone HTML blocks, reference-style links, inline-dollar math, footnote and link-reference definitions, heading attributes, or GFM alerts)
- **THEN** the coverage matrix classifies the construct in its implemented class and the construct does not reappear on the roadmap

#### Scenario: Secondary gaps are visible but lower priority
- **WHEN** a contributor evaluates whether to pick up a secondary gap (for example task-list checkbox interaction or angle-bracket autolinks)
- **THEN** the roadmap lists the secondary gap with its effort and implementation seam
- **AND** the contributor can open a change that closes it without re-litigating whether it is a gap

#### Scenario: New gaps discovered in implementation are added to the roadmap
- **WHEN** implementation or testing reveals a Markdown construct that renders as raw source in Visual Edit and is not yet on the roadmap
- **THEN** the discovering change SHALL add the construct to this roadmap with its class, priority, effort, and seam before completing
- **AND** the change SHALL NOT close the gap in the same change unless the gap is trivial

### Requirement: GFM table preview blocks carry parser event source ranges

The parser SHALL assign each `PreviewBlock::Table` the source range of the pulldown-cmark table event that produced that block’s rows. The range SHALL be non-empty and SHALL cover the authored GFM table bytes. The parser SHALL NOT assign an empty `0..0` placeholder range, and SHALL NOT zip table cell content to source ranges produced by a separate document scan that can skip tables the CommonMark+GFM parser emits. After derivation, GFM tables SHALL appear in the preview block stream in document (source) order. Nested-in-list table ordering MAY still be restored by sorting blocks on those event source-range starts; that sort SHALL NOT be used to repair invented placeholder ranges. Table cell-editing lookup MAY continue to use a dedicated scan of two-or-more-column tables and is not required to be 1:1 with preview table blocks.

#### Scenario: One-column GFM tables keep their event ranges

- **WHEN** the document contains a one-column GFM table (`| header |\n| --- |` with or without body rows) that the CommonMark+GFM parser emits as a table
- **THEN** the corresponding preview table block’s source range is non-empty
- **AND** the source slice for that range contains the table’s header line at its authored offset

#### Scenario: Mixed one-column and multi-column tables stay in document order

- **WHEN** a document has ordinary multi-column GFM tables, then one or more one-column `| command |\n| --- |` tables, then later multi-column result tables
- **THEN** the preview block stream lists those tables in authored source order
- **AND** no result-table block is placed at source offset `0` unless that table is actually authored at the start of the document
- **AND** no table block is inserted between an H2 and the immediately following H3 when the source between them is only blank lines

#### Scenario: Empty placeholder ranges are not used for tables

- **WHEN** the parser emits a GFM table with at least one row
- **THEN** that table’s preview `source_range` is non-empty
- **AND** `source_range.start` equals the start of the pulldown-cmark table event for that table (adjusted for any front-matter body offset)

#### Scenario: Nested list tables still follow document order

- **WHEN** a list item contains a nested GFM table
- **THEN** the preview stream places the list item block before that table
- **AND** the list item’s source range ends no later than the nested table’s source range start

### Requirement: CRLF HTML events coalesce into one preview block

When pulldown-cmark emits consecutive `Event::Html` pieces whose source ranges are separated only by whitespace that is not a CommonMark blank line (including a lone CR left after CRLF normalization), the parser SHALL emit a single `PreviewBlock::Html`. That block’s `source_range` SHALL be the contiguous span from the first piece through the last, and its `html` string SHALL be the corresponding slice of canonical document text (not a concatenation of event payloads that omit CR). Two HTML blocks separated by a blank line SHALL remain two preview blocks. The same coalescing SHALL apply to HTML accumulated into list items and blockquotes. Incremental source-mapped derivation SHALL match this full-parse result.

#### Scenario: CRLF table lines become one HTML preview block

- **WHEN** the document contains a multi-line raw HTML `<table>…</table>` whose line endings are CRLF and which contains no blank line between tags
- **THEN** `preview_blocks()` contains exactly one `PreviewBlock::Html` for that table
- **AND** that block’s `source_range` covers the authored `<table` through `</table>`
- **AND** the block’s `html` matches the document slice for that range

#### Scenario: LF table lines stay one HTML preview block

- **WHEN** the same table markup uses LF line endings
- **THEN** `preview_blocks()` still contains exactly one `PreviewBlock::Html` for that table

#### Scenario: Blank line keeps two HTML blocks apart

- **WHEN** the document contains two complete raw HTML blocks separated by a blank line (for example two `<p>…</p>` blocks)
- **THEN** `preview_blocks()` contains two `PreviewBlock::Html` entries in document order

