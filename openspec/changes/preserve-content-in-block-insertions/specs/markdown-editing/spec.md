## MODIFIED Requirements

### Requirement: Visual Edit SHALL support exact block transformations and operations
A supported Visual Edit block SHALL expose compact block commands to turn it into Text, Heading 1 through Heading 5, Bulleted List, Numbered List, Task List, Quote, or Code Block, and to Copy, Cut, Duplicate, Move Up, Move Down, Add Below or Delete it. Suitable prose targets SHALL also offer Paste and Link, Formula and Image settings through existing clipboard/editor/resource behavior. Image targets SHALL retain width/alignment presets and Image Properties, replacement, save and upload actions. The right-click/keyboard menu SHALL retain block transforms, duplication, movement, insertion below and explicitly labeled Delete Block alongside fine editing. Each command SHALL use its own captured scope: block commands target the clicked block, while ordinary clipboard and inline editing target the selected content or exact pointer/caret location. Grip commands SHALL retain whole-block clipboard semantics. Both menus SHALL share the compact style of slash completion: direct buttons, localized tooltips, grouped labeled insertion actions and separated destructive Delete. Heading/list controls SHALL be directly reachable without format submenus, while image scaling SHALL use its contextual flyout, and the current block type and unavailable operations SHALL remain visibly identified. H6 SHALL remain supported through other established heading-formatting entry points.

Right-clicking an eligible block SHALL open the ordinary editing menu without collapsing or expanding the current exact text/content selection; the keyboard context-menu action SHALL open the same editing catalog near the caret. Grip selection, drag behavior and the existing automatic-menu preference SHALL be preserved. Every enabled command SHALL be reachable by pointer and keyboard. Both menus SHALL render above all Visual Edit rows and media, remain anchored near the invoking pointer or caret inside the usable viewport, and keep overflow commands reachable through menu-local scrolling and keyboard navigation. Showing, positioning, navigating, scrolling within, or dismissing a menu SHALL NOT change source, document version, selection, history, dirty state, or derived-cache identity. Every operation SHALL validate current document identity/version and exact ownership; source-changing operations SHALL use the canonical mutation/undo path and preserve unrelated bytes, line endings, persistence/recovery behavior, tab isolation and cache invariants.

Editing context menus specialize this catalog with top-level prose/heading/block format and whole-block controls before selection editing, a bottom Insert flyout, and separate table Row/Column flyouts with whole-table operations at the root. Images use Properties and supported scaling/alignment/resource commands. Whole-unit actions explicitly name the current table/image/heading/paragraph. Grip command grids remain directly available. Divider and Table SHALL be insertion-only commands in these menus: their old conversion buttons SHALL be absent, and compact Insert Divider Below and Insert Table Below controls SHALL appear immediately after Insert Paragraph Below in the whole-block operations group of block-grip and ordinary prose/heading/block menus. Localized labels SHALL name the complete captured target and the insertion direction. Table-cell editing menus SHALL retain their separate row/column and selection catalog. The underlying block transformation boundary SHALL reject replacing non-empty content with a divider or empty table template. This organization supersedes the earlier contextual format and combined Table flyouts.

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
- **THEN** Text, H1–H5 and list/quote/code controls appear directly in a compact grid consistent with slash completion
- **AND** divider/table insertion below, insertion settings, clipboard/block actions and separated Delete remain reachable according to target availability

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

#### Scenario: Insert a divider below a complete table
- **WHEN** the user selects Insert Divider Below from a table block-grip menu
- **THEN** the full table, including authored cells, alignments and any column-width comment, remains byte-identical and a new standalone divider is inserted after it
- **AND** a blank paragraph below the divider receives the caret, without consuming the following block or an unrelated selection
- **AND** source separators preserve LF/CRLF, including a table at document end, and one Undo restores the original source and selection while Redo restores the insertion

#### Scenario: Insert templates without discarding prose or code
- **WHEN** the user activates Insert Divider Below or Insert Table Below on a non-empty paragraph, heading, code block or table block
- **THEN** the new block is inserted after the complete captured source unit and all original bytes are retained
- **AND** the divider continuation paragraph or new table receives the caret, and the insertion is one atomic history entry
- **AND** a direct attempt to replace non-empty block content with either template is rejected without mutation

#### Scenario: Empty paragraph insertion and stale commands stay safe
- **WHEN** divider insertion is chosen from Insert on an exactly mapped empty paragraph, or an insertion command carries a stale document/block capture
- **THEN** the live empty paragraph is reused with valid block separation and a blank continuation paragraph below the new divider, while a stale capture produces no mutation
- **AND** opening, navigating and dismissing menus preserve canonical selection, source, version, dirty state, history and shared per-version derivations until confirmation

#### Scenario: Compact insertion below remains accessible
- **WHEN** a block-grip or ordinary prose/heading/block menu opens in either theme or a constrained viewport
- **THEN** the compact divider/table insertion controls are grouped with Insert Paragraph Below, have scoped localized tooltips, and remain reachable by pointer and keyboard through menu-local scrolling
- **AND** ordinary Insert flyout and slash insertion remain available with their existing catalogs

