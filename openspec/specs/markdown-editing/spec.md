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
The editor SHALL provide a Visual Edit mode that presents common Markdown constructs, including valid inline and display math, in a rendered, editable form while preserving `MarkdownDocument.text` as the single canonical document representation. Visual Edit mutations SHALL update the Markdown source text through the same dirty-state, undo/redo, autosave, recovery, and per-tab isolation paths as source editing. Math SHALL be rendered while unfocused and SHALL reveal its complete authored delimiter group or a source-backed edit island when focused; it SHALL NOT be mutated through an inferred rendered formula tree.

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

#### Scenario: Unfocused math is rendered in Visual Edit
- **WHEN** valid inline, display, or fenced math is visible in Visual Edit and neither its source range nor delimiter group is focused
- **THEN** inline math appears as a baseline-aligned formula atom and display math appears as a typeset block
- **AND** the authored Markdown remains the canonical content

#### Scenario: Focused inline math reveals one complete source group
- **WHEN** the caret or a selection endpoint enters an inline math source range in Visual Edit
- **THEN** the complete byte-exact delimiter group is revealed as one source-backed editable range
- **AND** unrelated prose in the same block remains rendered

#### Scenario: Focused display math uses a source edit island
- **WHEN** the user focuses `$$...$$` or fenced `math` content in Visual Edit
- **THEN** that formula presents a source-backed edit island containing its exact authored syntax
- **AND** moving focus away restores formula rendering without changing the document version

#### Scenario: Complex constructs use conservative edit islands
- **WHEN** the user focuses a fenced code block, HTML/front matter region, image, malformed math, or other construct not supported by direct visual editing
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

### Requirement: Visual Edit inline formatting fidelity
Visual Edit SHALL render byte-exact supported inline formatting in prose blocks without exposing its Markdown delimiters while the construct is unfocused. Supported formatting SHALL include emphasis, strong emphasis, safely nested strong/emphasis combinations, strikethrough, inline code, links, highlight, superscript, and subscript. Moving the caret or a selection endpoint into a supported formatted construct SHALL reveal one safe containing source group for precise editing without converting unrelated inline content in the same block to raw Markdown. Constructs whose source/display mapping is malformed, crossing, escaped, or otherwise ambiguous SHALL retain the conservative source-editing fallback.

#### Scenario: Default inline formatting paragraph stays visual
- **WHEN** the default welcome document is opened in Visual Edit mode and its Inline formatting paragraph is not focused
- **THEN** supported Markdown delimiters in that paragraph are hidden
- **AND** italic, bold, combined bold-and-italic, strikethrough, inline code, link, highlight, superscript, and subscript content is rendered with its corresponding visual style

#### Scenario: Nested formatting reveals one safe containing group
- **WHEN** the caret or a selection endpoint enters byte-exact nested strong/emphasis content in Visual Edit
- **THEN** the editor reveals one outermost containing Markdown source range without duplicating text
- **AND** source/display mappings remain monotonic and UTF-8 safe
- **AND** unrelated inline content in the same block remains rendered

#### Scenario: Extended inline markers remain source-backed
- **WHEN** the caret enters a valid highlight, superscript, or subscript construct in Visual Edit
- **THEN** the complete local delimiters are revealed for source-backed editing
- **AND** moving the caret away hides those delimiters and restores the visual style
- **AND** cursor-only reveal does not change the document version or invalidate cached visual blocks

#### Scenario: Ambiguous inline syntax remains conservative
- **WHEN** a prose block contains escaped, malformed, crossing, or byte-inexact inline syntax that cannot be mapped safely
- **THEN** Visual Edit preserves a source-backed conservative editing affordance
- **AND** the editor does not guess a rendered-tree mutation for that construct

### Requirement: Visual Edit whitespace activation
The system SHALL keep source-backed whitespace ranges available for exact caret mapping while treating whitespace between rendered blocks as passive layout until the source caret intentionally enters that range. When the source caret owns a whitespace row — whether because the user pressed Enter at the end of a paragraph (whose source range excludes the trailing newline) or because keyboard navigation moved the caret into a whitespace-only range — Visual Edit SHALL present the row as the same passive-height layout it uses when unfocused, plus a thin insertion caret line visually consistent with the caret in a paragraph or heading, and SHALL accept subsequent typed text at the exact source caret position. Visual Edit SHALL NOT wrap a whitespace row that owns the caret in a source-island box (border, padding, monospace styling, or differentiated background), because such chrome misrepresents ordinary inter-paragraph spacing as a code-like block. Source islands SHALL remain reserved for blocks whose source has no rendered visual form (frontmatter, code, HTML, unsupported constructs) or for inline runs whose source/display mapping is ambiguous and therefore requires a conservative source-editing fallback.

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
When Visual Edit is active, vertical and line-boundary navigation SHALL follow the painted visual layout rather than only logical Markdown source lines. Up/Down and their selection variants SHALL retain a preferred horizontal coordinate across wrapped lines and adjacent visual blocks, while Home/End SHALL target the active painted line in rendered content.

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

### Requirement: Direct Markdown image editing in Visual Edit
Visual Edit SHALL present an exactly ranged inline Markdown image as its image preview together with direct text controls for alt text, destination, and optional title. Each control SHALL edit only its validated authored field range, preserve unrelated delimiters and escaping, and use the canonical source selection, platform input, IME, history, dirty-state, and multi-tab paths. Reference-style images, multiline or malformed syntax, and field forms whose exact boundaries cannot be proven MUST retain the complete source-backed image island.

#### Scenario: Image preview exposes editable authored fields
- **WHEN** an exactly ranged inline Markdown image is shown in Visual Edit
- **THEN** the image preview is accompanied by editable alt text and destination controls
- **AND** an authored title is editable without exposing the complete Markdown source

#### Scenario: Destination edit updates image presentation
- **WHEN** the user edits the destination field and commits platform text input
- **THEN** one exact canonical source replacement updates the destination
- **AND** the preview requests the new local or remote image without persisting preview state into the document

#### Scenario: Broken image remains editable
- **WHEN** the destination cannot be loaded or decoded
- **THEN** Visual Edit shows a bounded unavailable-image presentation while keeping all proven image fields editable
- **AND** the load failure does not mutate source, history, or document version

#### Scenario: Ambiguous image syntax remains source-backed
- **WHEN** an image uses reference syntax, malformed delimiters, unsupported multiline syntax, or another form without proven field ranges
- **THEN** Visual Edit presents the complete authored image source island
- **AND** it does not guess alt, destination, or title mutations

### Requirement: Maintained Visual Edit support classification
The repository SHALL maintain a current Visual Edit support matrix that classifies every user-visible Markdown construct as rendered direct editing, rendered editing with progressive source reveal, a dedicated field/payload editor, a passive exact source position, or a complete conservative source island. The matrix SHALL identify the canonical editable range, uncertainty trigger, and required verification evidence for each classification, and SHALL agree with the stable requirements and implemented `VisualBlock`/`VisualBlockEditor` behavior.

#### Scenario: Contributor evaluates current WYSIWYG coverage
- **WHEN** a contributor reads the Visual Edit support matrix
- **THEN** it distinguishes directly editable prose, inline source reveal, code/math/image/table editors, whitespace/passive positions, and HTML/front-matter/diagram/ambiguous fallbacks
- **AND** it explains that canonical Markdown remains the single persisted representation

#### Scenario: A new visual block behavior is proposed
- **WHEN** a proposal changes how a Markdown construct is presented or edited in Visual Edit
- **THEN** the proposal selects one support classification and names its exact fallback trigger
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

