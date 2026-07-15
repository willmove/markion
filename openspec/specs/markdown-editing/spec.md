# markdown-editing

## Purpose

Covers the core source-text editing surface, Markdown parsing, the formatting actions, keyboard shortcuts, and the extended Markdown syntax set. This is the source/split editing model — the editor pane shows raw Markdown text and the preview pane renders it. Single-surface WYSIWYG editing (markers hiding around the cursor) is **not** part of this capability; it is a future candidate.
## Requirements
### Requirement: Markdown parsing via CommonMark + GFM
The parser SHALL parse Markdown using `pulldown-cmark` configured for CommonMark conformance plus the GitHub Flavored Markdown extensions in use (tables, task lists, strikethrough, footnotes, superscript/subscript, highlight, autolinks). Parsing SHALL produce structured data consumed by the preview, outline, stats, and search subsystems.

#### Scenario: Full reparse per edit yields structured blocks
- **WHEN** the document text changes
- **THEN** the parser runs over the full document text and produces the structured preview blocks, outline, stats, and line count consumed downstream

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
The editor SHALL hold zero or more open documents as tabs within a single window (`tabs: Vec<EditorTab>` + an `active_tab` index), rather than a single document per window. Each tab SHALL carry its own isolated document, cursor/selection, scroll position, undo/redo history, IME composition state, layout caches, dirty flag, and autosave/recovery tracking — switching tabs SHALL NOT disturb another tab's state. Tabs for filesystem-backed documents SHALL be unique by file path within a window: when an open request targets a file that is already open in another tab, the editor SHALL focus that existing tab instead of opening a duplicate tab. A tab bar SHALL be rendered only when more than one tab is open; with a single tab the editor looks identical to the pre-tab single-document layout. Tabs are session-only: they are not persisted across launches (restarting returns to a single untitled document).

#### Scenario: Opening files creates switchable tabs with isolated state
- **WHEN** the user opens a second file (via the file tree, or the OpenInNewTab action)
- **THEN** a new tab is appended and activated, and switching back to the first tab restores its exact cursor position, scroll offset, and undo history

#### Scenario: Opening an already-open file focuses its existing tab
- **WHEN** the user opens a file by path and that same file is already open in a tab
- **THEN** the existing tab is activated
- **AND** no duplicate tab is appended or replaced
- **AND** the existing tab's document text, dirty flag, cursor/selection, undo/redo history, editor scroll position, preview scroll position, and derived Markdown caches are preserved

#### Scenario: File→Open replaces the active tab
- **WHEN** the user invokes File→Open and picks a file that is not already open
- **THEN** the active tab's document is replaced (after a dirty-guard on that tab), matching the single-document behavior, rather than spawning a new tab

#### Scenario: Tab navigation and closing
- **WHEN** the user presses the next/previous tab shortcut (Ctrl+Tab / Ctrl+Shift+Tab) or clicks a tab / its close button
- **THEN** the active tab switches in opening order, or the targeted tab closes; closing the last tab creates a fresh untitled document rather than closing the window

#### Scenario: Closing an unsaved tab prompts for confirmation
- **WHEN** the user closes a tab whose document has unsaved changes
- **THEN** the editor prompts for confirmation before discarding those changes

#### Scenario: Quitting with multiple unsaved tabs
- **WHEN** the user quits or closes the window while two or more tabs have unsaved changes
- **THEN** the editor detects the unsaved tabs and prompts before discarding them

#### Scenario: Autosave targets the tab that was active when scheduled
- **WHEN** an autosave timer fires after the user has switched tabs
- **THEN** the autosave writes the tab whose generation was captured at schedule time, not whichever tab is now active

#### Scenario: Single-tab layout is unchanged
- **WHEN** only one tab is open
- **THEN** no tab bar is rendered and the editor's appearance matches the pre-tab single-document layout

### Requirement: Editor view modes
The editor SHALL provide four mutually exclusive view modes: Edit, Visual Edit, Split Preview, and Read. Edit mode SHALL show the Markdown source editing surface without the rendered preview pane. Visual Edit mode SHALL show a single source-backed visual editing surface where common Markdown constructs render close to their preview appearance while remaining editable. Split Preview mode SHALL show the Markdown source editing surface and rendered preview pane together, preserving the current live-preview workflow. Read mode SHALL show the rendered Markdown preview without the source editing pane and SHALL NOT allow editing through the rendered preview.

#### Scenario: Edit mode shows only source editing
- **WHEN** the active view mode is Edit
- **THEN** the source editing surface is visible and accepts normal editing operations
- **AND** the rendered preview pane is not visible

#### Scenario: Visual Edit mode shows one editable visual surface
- **WHEN** the active view mode is Visual Edit
- **THEN** the editor shows a single visual editing surface
- **AND** common Markdown prose constructs are rendered visually where supported
- **AND** edits continue to mutate the underlying Markdown source text

#### Scenario: Split Preview mode shows both panes
- **WHEN** the active view mode is Split Preview
- **THEN** the source editing surface and rendered preview pane are both visible
- **AND** edits in the source surface continue to update the preview through the existing derived Markdown state

#### Scenario: Read mode shows only rendered Markdown
- **WHEN** the active view mode is Read
- **THEN** the rendered preview pane is visible
- **AND** the source editing surface is not visible
- **AND** interacting with rendered preview content does not mutate the document text

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
The editor SHALL provide a Visual Edit mode that presents common Markdown constructs in a rendered, editable form while preserving `MarkdownDocument.text` as the single canonical document representation. Visual Edit mutations SHALL update the Markdown source text through the same dirty-state, undo/redo, autosave, recovery, and per-tab isolation paths as source editing.

#### Scenario: Visual prose editing updates Markdown source
- **WHEN** the user edits visible prose inside a paragraph, heading, blockquote, or list item in Visual Edit mode
- **THEN** the corresponding Markdown source text is updated
- **AND** the document dirty flag and undo history are updated through the existing document mutation path

#### Scenario: Visual formatting actions remain source-backed
- **WHEN** the user applies bold, italic, inline code, link, image, heading, list, task list, blockquote, or fenced-code formatting in Visual Edit mode
- **THEN** the editor updates the underlying Markdown markers in `MarkdownDocument.text`
- **AND** switching to Edit mode shows Markdown source that represents the visual result

#### Scenario: Focused syntax can be exposed for editing
- **WHEN** the cursor enters visually formatted inline content whose hidden Markdown syntax is needed for precise editing
- **THEN** the editor SHALL expose the relevant source syntax or a source-backed edit island for that focused content

#### Scenario: Complex constructs use conservative edit islands
- **WHEN** the user focuses a fenced code block, math block, HTML/front matter region, image, or other construct not supported by direct visual editing in v1
- **THEN** the editor SHALL provide a source-backed editing affordance or preserve the existing source editing workflow
- **AND** the construct SHALL NOT be mutated through an ambiguous rendered-tree edit

#### Scenario: Visual-only interaction does not reparse unnecessarily
- **WHEN** the user moves the cursor, changes selection, hovers text, or focuses a visual edit island without changing document text
- **THEN** the document version SHALL remain unchanged
- **AND** derived Markdown caches SHALL NOT be invalidated

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

