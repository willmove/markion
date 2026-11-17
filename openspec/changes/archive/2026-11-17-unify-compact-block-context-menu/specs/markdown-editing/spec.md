## ADDED Requirements

### Requirement: Context-labeled hover flyouts
Visual Edit editing menus SHALL offer separate Row and Column hover flyouts for tables, insertion for ordinary blocks and image scaling as applicable. Paragraph/heading/block formatting and whole-block actions SHALL appear directly at the top of the root menu, before selection editing, without a format flyout. Insert SHALL be the final ordinary-block root item and SHALL offer new paragraphs, H1–H5, bulleted/numbered/task lists, quotes, code blocks, dividers, tables, all five supported GFM callout kinds, and Link/Formula/Image settings. Block insertion on non-empty targets SHALL add a new source unit after the clicked complete block without converting it or replacing unrelated selected text, preserve neighbors and line endings, place the caret inside the new payload, and create one undo entry. Inline settings SHALL retain exact selection/caret targeting. Flyouts SHALL open without clicking, remain pointer-accessible beside the root, fit the viewport by flipping/scrolling, and support keyboard Right/Enter/Left/Escape. Clipboard and inline editing SHALL preserve the captured selection scope; whole-unit commands SHALL identify their actual context in localized labels. Changing hover or selection presentation SHALL not mutate canonical source, history or per-version derivations. Stale target/page callbacks SHALL not select or execute a different action.

#### Scenario: Block controls precede editing controls
- **WHEN** an ordinary paragraph, heading or supported block context menu opens
- **THEN** its format and whole-block actions appear directly before clipboard/inline controls, with Insert last and no paragraph/block submenu
- **AND** current/disabled states and keyboard reachability are retained

#### Scenario: Context menus omit H6
- **WHEN** the user opens the block context menu or its Insert flyout
- **THEN** heading choices are limited to H1–H5 and no H6 command is shown
- **AND** existing H6 document content and other heading-formatting entry points retain their support

#### Scenario: Insert into an empty paragraph
- **WHEN** a context menu opens on an exactly mapped empty document, blank paragraph or document-end insertion row
- **THEN** the root heading is localized as Format Block and the Insert flyout's supported block, callout and inline settings actions are enabled independently of whole-block clipboard/deletion
- **AND** block insertion reuses the exact clicked/caret-owned blank line, preserves unrelated selection and neighboring content with correct LF/CRLF separators, rejects stale captures and is one undoable edit
- **AND** opening/canceling inline settings leaves source, selection, version, history and derived caches unchanged

#### Scenario: Insert a new block without converting the target
- **WHEN** a new heading, list, task, quote, code, divider, table or callout is selected from Insert
- **THEN** it is inserted after the complete clicked source unit and before the following unit with valid separators
- **AND** unrelated selection/source bytes remain intact, stale captures are rejected and one Undo restores source and selection

#### Scenario: Hover into a flyout and back
- **WHEN** the pointer enters an enabled contextual parent and then crosses to its flyout
- **THEN** the flyout opens without a click and stays actionable without dismissing or moving the root
- **AND** hovering another root control changes or closes the flyout, while an outside click or document transition dismisses the complete menu

### Requirement: Empty callout bodies remain editable
Visual Edit SHALL partition an alert's marker line and authored quoted blank body into distinct contiguous source owners, including newly inserted body-less callouts with a quoted blank line. The blank body SHALL have a visible body-height editing target. The insertion caret, ordinary body clicks and normal styled-title clicks when a body exists SHALL target the body payload after its quote prefix, so typing cannot accidentally prepend to or enter the alert marker. Such clicks SHALL not mutate source, version, history or per-version caches. Explicit keyboard/source entry into the title SHALL retain existing marker/custom-title editing. Editing body content SHALL retain alert semantics, source mappings, LF/CRLF, neighboring bytes and atomic undo/redo.

#### Scenario: Click and type into a newly inserted callout
- **WHEN** the user inserts any supported callout and then clicks its empty body or styled title and types Unicode text
- **THEN** the text is entered on the quoted body line after the intact alert marker line
- **AND** the callout remains styled, the marker cannot become `text[!NOTE]` or `[!NOTE]text`, and Undo/Redo restore the body edit independently from insertion

### Requirement: Images use properties instead of Visual Edit source toggles
Markdown images in Visual Edit SHALL remain rendered when hovered, selected, focused, failed or subject to the stable-inline preference. They SHALL NOT expose a `</>` source toggle, an Edit Source context command or a source-expansion payload surface. Image Properties SHALL edit URL/alt/title in bounded localized fields, with exact occurrence/version checks and one-step undo. Opening/canceling or confirming unchanged properties SHALL preserve source, including reference syntax. Explicit changed reference properties MAY convert only that occurrence to inline syntax, preserving definitions and other occurrences. Existing safe width/alignment/resource controls and whole-document Source mode SHALL remain available.

#### Scenario: Hover or select an image
- **WHEN** an image is hovered/selected, including a missing-file, reference or large data-URI image
- **THEN** the picture or bounded load-error presentation remains visible without raw Markdown payload or source toggle
- **AND** context Properties provides the supported edit path without constructing a raw payload projection during hover/selection

#### Scenario: Change image properties and undo
- **WHEN** the user changes supported fields in a live image Properties pane and confirms
- **THEN** only the exact captured occurrence changes as one checked undoable mutation, retaining inline width/alignment metadata
- **AND** cancellation, no-op confirmation or a stale document generation leaves canonical source and unrelated image occurrences unchanged

## MODIFIED Requirements

### Requirement: Reference-style block images stay rendered when focused
Visual Edit SHALL treat single-line reference-style Markdown block images (`![alt][label]`, collapsed `![alt][]`, or shortcut `![alt]`) as rendered images while focused or selected. Exact authored range metadata SHALL remain available to checked operations, but no collapsible image source payload or source toggle SHALL be mounted. Width/alignment controls SHALL remain unavailable for reference syntax. Image Properties SHALL preserve source on opening, cancel and no-op confirmation; explicit changed properties MAY convert only the selected occurrence to inline syntax, preserving other occurrences and definitions. Whole-document Source mode remains the raw editing surface for unprovable forms.

#### Scenario: Focused reference image keeps the picture
- **WHEN** Visual Edit shows `![alt][asset]` with a later `[asset]: pic.png` definition and the user focuses/selects the picture
- **THEN** the rendered picture remains visible without raw Markdown or a source-toggle button
- **AND** Properties provides the supported context editing path

#### Scenario: Shortcut and collapsed reference forms are proven
- **WHEN** a single-line shortcut or collapsed image is parsed as a block image
- **THEN** it retains exact authored range metadata and the same rendered-only image context behavior

#### Scenario: Presentation controls stay inline-only
- **WHEN** the focused image is reference-style
- **THEN** width and alignment controls are unavailable and cannot rewrite the reference span
- **AND** explicit property confirmation affects only its captured occurrence as one undoable edit

### Requirement: Visual Edit SHALL support exact block transformations and operations
A supported Visual Edit block SHALL expose compact block commands to turn it into Text, Heading 1 through Heading 5, Bulleted List, Numbered List, Task List, Quote, Code Block, Divider, or Table, and to Copy, Cut, Duplicate, Move Up, Move Down, Add Below or Delete it. Suitable prose targets SHALL also offer Paste and Link, Formula and Image settings through existing clipboard/editor/resource behavior. Image targets SHALL retain width/alignment presets and Image Properties, replacement, save and upload actions. The right-click/keyboard menu SHALL retain block transforms, duplication, movement, insertion below and explicitly labeled Delete Block alongside fine editing. Each command SHALL use its own captured scope: block commands target the clicked block, while ordinary clipboard and inline editing target the selected content or exact pointer/caret location. Grip commands SHALL retain whole-block clipboard semantics. Both menus SHALL share the compact style of slash completion: direct buttons, localized tooltips, grouped labeled insertion actions and separated destructive Delete. Heading/list controls SHALL be directly reachable without format submenus, while image scaling SHALL use its contextual flyout, and the current block type and unavailable operations SHALL remain visibly identified. H6 SHALL remain supported through other established heading-formatting entry points.

Right-clicking an eligible block SHALL open the ordinary editing menu without collapsing or expanding the current exact text/content selection; the keyboard context-menu action SHALL open the same editing catalog near the caret. Grip selection, drag behavior and the existing automatic-menu preference SHALL be preserved. Every enabled command SHALL be reachable by pointer and keyboard. Both menus SHALL render above all Visual Edit rows and media, remain anchored near the invoking pointer or caret inside the usable viewport, and keep overflow commands reachable through menu-local scrolling and keyboard navigation. Showing, positioning, navigating, scrolling within, or dismissing a menu SHALL NOT change source, document version, selection, history, dirty state, or derived-cache identity. Every operation SHALL validate current document identity/version and exact ownership; source-changing operations SHALL use the canonical mutation/undo path and preserve unrelated bytes, line endings, persistence/recovery behavior, tab isolation and cache invariants.

Editing context menus specialize this catalog with top-level prose/heading/block format and whole-block controls before selection editing, a bottom Insert flyout, and separate table Row/Column flyouts with whole-table operations at the root. Images use Properties and supported scaling/alignment/resource commands. Whole-unit actions explicitly name the current table/image/heading/paragraph. Grip command grids remain directly available. This organization supersedes the earlier contextual format and combined Table flyouts.

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
- **AND** underlying content cannot visually obscure the menu or receive pointer actions within its bounds

#### Scenario: Block menu stays reachable near viewport edges
- **WHEN** the user opens the block-operation menu with insufficient space below or beside its invoking control
- **THEN** the panel flips or is constrained within the usable viewport
- **AND** overflow commands remain reachable through menu-local scrolling and keyboard navigation without scrolling the document

#### Scenario: Block menu dismissal is presentation-only
- **WHEN** the user dismisses an open block menu with Escape, an outside action, document scrolling, a tab or mode change, or stale-target invalidation
- **THEN** it closes without changing source, document version, selection, history, dirty state or derived-cache identity

#### Scenario: Right-click targets a non-caret block without collapsing selection
- **WHEN** Visual Edit owns an exact non-empty text selection and the user right-clicks another eligible block
- **THEN** the editing menu preserves the canonical selection and its editing actions target that selection rather than the clicked block
- **AND** more specific child interactions consuming the right-click are not replaced by the generic menu

#### Scenario: Keyboard context action opens an operable block menu
- **WHEN** the caret owns an eligible Visual Edit block and the user invokes the keyboard context-menu action
- **THEN** the compact editing menu opens near the painted caret or a bounded surface fallback and targets the current selection or caret, while block commands retain their independently captured block target
- **AND** Up, Down, Left, Right, Enter and Escape navigate, confirm or dismiss without mutating source before confirmation

#### Scenario: Compact transform groups expose every current block type
- **WHEN** the user opens the combined right-click/keyboard menu or block commands from an enabled ordinary grip click
- **THEN** Text, H1–H5 and list/quote/code/divider controls appear directly in a compact grid consistent with slash completion
- **AND** Table, insertion settings, clipboard/block actions and separated Delete remain reachable according to target availability

#### Scenario: Insertion settings preserve the clicked block and source until confirmation
- **WHEN** the user activates Link, Formula or Image settings from a valid prose block menu
- **THEN** settings target the exact selected text in that block, otherwise its editable caret position, otherwise its editable content end
- **AND** opening/canceling preserves source, selection, version, history, dirty state and derived-cache identity
- **AND** confirmation replaces only the captured range through existing source-backed editors/resources and one Undo restores the source

#### Scenario: Paste affects the clicked block
- **WHEN** the user invokes Paste on a valid prose target while the original caret or selection belongs to another block
- **THEN** the existing clipboard command inserts at the resolved editable target in the clicked block
- **AND** unrelated text and line endings remain intact and one Undo restores the prior source

#### Scenario: Image menu retains practical controls
- **WHEN** an exactly mapped image block menu opens, or an image is right-clicked without an active text/content selection
- **THEN** width and alignment presets are compact direct controls and Image Properties, replacement, save, upload and block operations remain reachable
- **AND** unavailable or stale image operations do not guess a mutation

### Requirement: Mouse block selection and ordinary editor context menu
Source SHALL support double-click word selection and triple-click current-block selection, and Visual Edit SHALL support triple-click current-block selection. The editor SHALL provide an ordinary context menu offering Cut, Copy, Paste and Select All, using existing command paths and appropriate availability for the current selection. In Visual Edit it SHALL additionally offer selection Delete and share compact presentation with block commands while using a combined catalog of block, selection and applicable table commands. Visual Edit Copy, Cut and Delete SHALL operate only on the captured selected text or structured content, including partial paragraphs, cross-block ranges and table-cell selections; they SHALL be disabled when nothing is selected. Paste SHALL replace the current captured selection or insert at the exact mouse location for pointer invocation or the current caret for keyboard invocation, without expanding to a block or falling back to its content end. Supported inline formatting and Link/Formula/Image settings SHALL target the current exact selection, or exact mouse/caret position when there is no selection; supported inline formats SHALL include Bold, Italic, Inline Code, Strikethrough and Link; unsupported contexts SHALL disable those actions. Whole-block transforms, duplication, movement, Add Below and explicitly labeled Delete Block SHALL also remain reachable in the context menu and SHALL target the clicked block. Generic Copy/Cut/Paste/Delete Selection SHALL retain fine editing semantics. Opening or dismissing the menu SHALL NOT mutate document text, version, history, selection, or derived caches. Visual Edit activation SHALL reject changed document identity/version or selection and SHALL use the document selection even if another input field retains focus.

#### Scenario: Double and triple click
- **WHEN** the user double-clicks Source text or triple-clicks a Source or Visual text block
- **THEN** the word or exact editable block content is selected through the canonical selection path

#### Scenario: Context menu editing
- **WHEN** the user opens the editor context menu and invokes an enabled editing action
- **THEN** the existing Cut, Copy, Paste, Delete or Select All command runs with its established source and undo behavior
- **AND** unrelated menu lifecycle changes leave the document unchanged

#### Scenario: Copy and cut only selected text
- **WHEN** part of a paragraph or a range across paragraphs is selected and Copy or Cut is invoked from the right-click menu on any eligible row
- **THEN** only that selected text/content is copied to the clipboard
- **AND** Cut removes only the selection, preserves surrounding bytes and line endings, and one Undo restores source and selection

#### Scenario: Selection deletion and paste
- **WHEN** Delete or Paste is invoked from the editing menu with a non-empty selection
- **THEN** only the selected text/content is removed or replaced
- **AND** the clicked block is not implicitly selected or removed, and one Undo restores the prior source and selection

#### Scenario: Caret-only editing menu
- **WHEN** the editing menu opens with no text/content selection
- **THEN** Copy, Cut and selection Delete are disabled while Paste inserts at the captured mouse position or keyboard caret
- **AND** no ordinary clipboard or deletion action substitutes the clicked block

#### Scenario: Changed context is rejected
- **WHEN** document identity/version, the text selection or structured selection differs from the menu's captured context
- **THEN** activation closes the menu without changing source or clipboard

#### Scenario: Block and fine editing commands coexist
- **WHEN** text in one block is selected and the user right-clicks another eligible block
- **THEN** heading/text/list/code/divider/table transforms, block movement and Delete Block remain reachable for the clicked block
- **AND** Copy, Cut, Paste and supported inline formats still affect only the preserved selection

#### Scenario: Pointer targeting with a collapsed selection
- **WHEN** the user right-clicks an exact text location different from the current caret with no active content selection
- **THEN** Paste and supported inline-format/settings actions target that captured mouse location
- **AND** opening/canceling preserves source, selection, version, history and cache identity

#### Scenario: Strikethrough edits only the selected content
- **WHEN** Strikethrough is invoked for an exactly mapped prose or single-cell text selection
- **THEN** the selected source is wrapped or unwrapped with double tildes as one undoable mutation
- **AND** surrounding text, cell delimiters and unrelated block structure remain unchanged

#### Scenario: Table cell click dismisses menus
- **WHEN** a block or table-axis menu is open and the user left-clicks table cell text or padding, including the same cell
- **THEN** the menu closes and the cell's normal caret/selection behavior proceeds
- **AND** dismissal itself does not mutate source, version, history or derived caches

#### Scenario: New Tab with an open context menu
- **WHEN** a document context menu is open and the user invokes New Tab, including Ctrl+Shift+N
- **THEN** the new tab becomes active, the previous document's menus are dismissed and the application remains running
- **AND** the original document's content and undo history remain intact

#### Scenario: Document transition and stale rendering
- **WHEN** a document is opened, replaced, switched, closed, changed into an image tab, or a previously captured menu target is otherwise stale at render time
- **THEN** document-bound menus are cleared or omitted without panicking or accessing image tabs as documents
- **AND** stale actions cannot affect the newly active document
