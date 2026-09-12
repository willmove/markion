## ADDED Requirements

### Requirement: Markdown delimiter auto-pair on editing surfaces
When Markdown auto-pair is enabled, typing an opening delimiter on the source editor, Visual Edit prose, or a Visual Edit table cell SHALL insert the matching closer and leave the caret between the pair; typing an opener over a non-empty selection SHALL wrap that selection; typing a closer whose next source character is already that closer SHALL move the caret past it without inserting; Backspace between an empty pair SHALL delete both characters. The paired openers SHALL be `*`, `_`, `` ` ``, `$`, `(`, `[`, `{`, `"`, and `'`. Pairing SHALL NOT run while an IME marked range is active, when the previous character is a backslash, or inside a fenced-code, block-math, or diagram payload editor. Pairing SHALL be one canonical source mutation through the existing dirty-state, undo/redo, autosave, and recovery path.

#### Scenario: Typing an opener inserts a wrap pair
- **WHEN** Markdown auto-pair is enabled and the user types `*` with an empty selection in Visual Edit prose or the source editor
- **THEN** the source contains `**` at that offset with the caret between the two characters
- **AND** one Undo removes both characters

#### Scenario: A selection is wrapped
- **WHEN** Markdown auto-pair is enabled and the user types `(` over a non-empty, exactly mapped selection
- **THEN** the selected text is wrapped in `(` and `)`
- **AND** the resulting selection stays on the wrapped content

#### Scenario: Typing the closer skips an existing pair
- **WHEN** the source after the caret is already the matching closer for the character just typed
- **THEN** the editor moves the caret past that closer
- **AND** it does not insert a second closer or bump derived caches beyond the caret move

#### Scenario: Backspace unwraps an empty pair
- **WHEN** the caret sits between a just-inserted empty pair and the user presses Backspace
- **THEN** both the opener and closer are removed in one edit

#### Scenario: Pairing is suppressed in code and during IME
- **WHEN** the caret is inside a fenced-code, block-math, or diagram payload editor, or an IME composition is active
- **THEN** typed `*` / `` ` `` / `$` insert only the typed character

#### Scenario: Disabled preference types literally
- **WHEN** Markdown auto-pair is off
- **THEN** typing an opener inserts only that opener

### Requirement: Unfocused Visual Edit block markers stay hidden after Enter
When the Visual Edit caret leaves a heading, ordered or unordered list item, task-list item, or blockquote row — including by pressing Enter to start the next paragraph or list item — that row SHALL hide its structural Markdown prefixes (`#`–`######`, list/task markers, `>`) and present the rendered form (heading typography, list/task glyph, quote bar). Hiding prefixes SHALL NOT rewrite `MarkdownDocument.text`. A line whose source is a just-typed ATX, list, task, or quote prefix (with optional heading title text) SHALL be classified as that block on the next derivation so the unfocused row does not keep the marker characters as visible prose. Focusing the row SHALL still reveal the prefix through the existing progressive-reveal mapping.

#### Scenario: Enter after a heading hides the hashes
- **WHEN** the Visual Edit caret is in an ATX heading and the user presses Enter, producing a following paragraph
- **THEN** the heading row paints without visible `#` markers
- **AND** the heading source still contains those hashes
- **AND** moving the caret back onto the heading reveals the prefix

#### Scenario: Enter after a list item hides the previous marker source
- **WHEN** the caret is in a non-empty list or task item and the user presses Enter to continue the list
- **THEN** the previous item shows the rendered bullet, number, or checkbox glyph rather than `- ` / `1. ` / `- [ ]`
- **AND** the new item may reveal its prefix while it owns the caret

#### Scenario: Typed heading prefix becomes a heading, not leftover prose
- **WHEN** the user types `## Title` at the start of a Visual Edit paragraph and presses Enter
- **THEN** the previous row is a heading whose unfocused presentation hides `##`
- **AND** the following row is a paragraph

#### Scenario: Caret leave without Enter also hides prefixes
- **WHEN** the user clicks another Visual Edit row so a heading or list row no longer owns the caret
- **THEN** that row hides its structural prefixes
- **AND** document version, dirty state, and derived Markdown caches are unchanged

### Requirement: Visual Edit task-list checkboxes are clickable
In Visual Edit, an unfocused (prefix-hidden) task-list item SHALL present its checkbox glyph as a pointer target. Activating that glyph SHALL toggle the item's canonical task marker between `[ ]` and `[x]` as one exact source mutation, preserving the item text, list indentation, and surrounding source. `[X]` SHALL toggle off to `[ ]`. The click SHALL NOT run when the task prefix is revealed as source, SHALL NOT edit Read or Split Preview, and SHALL NOT bump document version if the pointer misses the glyph. Showing the glyph and hovering it SHALL NOT mutate the document.

#### Scenario: Clicking an unchecked box checks it
- **WHEN** a Visual Edit task item is unchecked and its checkbox glyph is visible
- **AND** the user primary-clicks that glyph
- **THEN** the source marker at that item becomes `[x]`
- **AND** one Undo restores `[ ]`

#### Scenario: Clicking a checked box unchecks it
- **WHEN** a Visual Edit task item is checked (`[x]` or `[X]`) and the user primary-clicks its checkbox glyph
- **THEN** the source marker becomes `[ ]`

#### Scenario: Revealed source marker is not a toggle hit target
- **WHEN** the caret is in a task item so `- [ ]` / `- [x]` is revealed
- **THEN** clicking inside the revealed prefix edits or places the caret in source
- **AND** it does not perform a separate checkbox-toggle mutation

#### Scenario: Read and Split Preview stay inert
- **WHEN** the user clicks a task-list checkbox glyph in Read mode or Split Preview
- **THEN** the document text, dirty state, and undo history are unchanged

### Requirement: Colon emoji shortcode completion
When the user types `:` at a word boundary (start of line, whitespace, or opening punctuation) on the source editor or Visual Edit prose, outside fenced-code / math / diagram payloads and outside an IME composition, the editor SHALL open a localized shortcode palette filtered by the ASCII prefix after that colon against the maintained emoji shortcode table. Confirming a row (Enter, Tab, or pointer) SHALL replace the open `:query` with the canonical `:name:` (including the closing colon) as one undoable source edit; the preview and unfocused Visual Edit SHALL render the matching emoji glyph through the existing shortcode parser. Escape or moving the caret so the token is no longer an open shortcode SHALL dismiss the palette without writing. Opening, filtering, moving the highlight, and dismissing SHALL NOT increment document version. If a slash-command palette is already open for a `/` query, that palette SHALL take priority and this completer SHALL NOT open.

#### Scenario: Colon at a word boundary opens the palette
- **WHEN** the user types `:smi` in Visual Edit prose after a space
- **THEN** a palette lists shortcodes whose names start with `smi` (for example `smile`)
- **AND** document version is unchanged until a row is confirmed

#### Scenario: Confirm inserts the shortcode
- **WHEN** the palette is open on `smile` and the user confirms
- **THEN** the source at that token is `:smile:`
- **AND** unfocused Visual Edit and preview render the smile emoji
- **AND** one Undo restores the text from before the confirmation

#### Scenario: Escape dismisses without writing
- **WHEN** the emoji palette is open and the user presses Escape
- **THEN** the palette closes
- **AND** the authored `:query` remains as typed
- **AND** document version is unchanged by the dismiss

#### Scenario: Time and URLs do not open the palette
- **WHEN** the user types `12:30` or `https://example.com`
- **THEN** the emoji palette does not open

#### Scenario: Slash commands win on a slash query
- **WHEN** the slash-command palette is open because the line starts with `/`
- **THEN** typing `:` inside that query does not open the emoji palette

## MODIFIED Requirements

### Requirement: Maintained Visual Edit support classification
The repository SHALL maintain a current Visual Edit WYSIWYG coverage matrix that classifies every user-visible Markdown construct into exactly one of three classes: **rendered WYSIWYG** (the construct is shown in its rendered form, including dedicated field/payload editors for code, math, diagrams, images, and tables whose editors ARE the rendered form), **progressive-reveal WYSIWYG** (the construct is rendered by default and reveals its smallest complete source syntax group when the caret enters it — inline formatting, links, inline math, structural prefixes), or **WYSIWYG coverage gap** (the construct currently shows raw source and is tracked under the `WYSIWYG coverage roadmap` for closure by a future change). The matrix SHALL name the canonical editable range and the verification evidence for each rendered/reveal class, and SHALL name the roadmap priority and implementation seam for each gap. The matrix SHALL agree with the stable requirements and the implemented `VisualBlock`/`VisualBlockEditor` behavior.

#### Scenario: Contributor evaluates current WYSIWYG coverage
- **WHEN** a contributor reads the Visual Edit WYSIWYG coverage matrix
- **THEN** it distinguishes rendered WYSIWYG constructs (prose, code, math, diagrams, images, tables, task lists including clickable Visual Edit checkboxes, footnote definitions and references, blockquotes, alerts, rules, HTML blocks), progressive-reveal WYSIWYG constructs (inline formatting, links, inline math, escaped punctuation, supported inline HTML, structural prefixes, heading attributes), and open WYSIWYG gaps (decoded entities, front matter, indented code, unclosed fences, reference-style images, malformed tables, unsupported inline-HTML forms, autolinks, definition lists, empty list items)
- **AND** it explains that canonical Markdown remains the single persisted representation and that no construct is edited through a parallel rendered tree

#### Scenario: A new visual block behavior is proposed
- **WHEN** a proposal changes how a Markdown construct is presented or edited in Visual Edit
- **THEN** the proposal selects one of the three coverage classes for the construct
- **AND** if the proposal moves a construct out of the gap class, it updates the matrix and the `WYSIWYG coverage roadmap`
- **AND** implementation and documentation cannot be considered complete until the matrix and invariant evidence are updated

### Requirement: WYSIWYG coverage roadmap
The repository SHALL maintain, as part of the Visual Edit WYSIWYG coverage matrix, a prioritized roadmap of every Markdown construct that is currently classified as a WYSIWYG coverage gap. The roadmap SHALL name, for each gap, the construct, its current rendering (transitional source view), its target WYSIWYG class (rendered or progressive-reveal), its priority, its rough implementation effort, and the implementation seam in the existing code. The roadmap SHALL be closed incrementally by future changes, each of which SHALL move one or more constructs out of the gap class and update this roadmap. The initial roadmap SHALL include at minimum the following primary gaps in priority order: (1) decoded HTML entities in prose blocks (for example `&amp;`), (2) front matter (an editing form for YAML `---` regions, and detection of TOML/JSON forms), and (3) indented code blocks. The roadmap SHALL also track secondary gaps including unclosed or malformed fenced code, reference-style and malformed inline images, malformed tables, unsupported inline-HTML forms and angle-bracket autolinks in prose, GFM definition lists, empty list items, and math render-failure states. Task-list checkbox click interaction SHALL NOT remain on the roadmap once Visual Edit checkboxes are clickable.

#### Scenario: Primary gaps are tracked with priority and effort
- **WHEN** a contributor reads the WYSIWYG coverage roadmap
- **THEN** the current primary gaps (decoded HTML entities, front matter, indented code blocks) are listed with priority, effort, target class, and implementation seam
- **AND** each primary gap points at the source location of the current transitional source-view rendering

#### Scenario: Closing a gap updates the roadmap
- **WHEN** a future change implements WYSIWYG rendering for a construct that the roadmap tracks as a gap
- **THEN** that change's spec delta moves the construct out of the gap class in the `Maintained Visual Edit support classification` matrix and removes it from this roadmap
- **AND** the change's proposal cites this roadmap requirement as its motivation

#### Scenario: Closed gaps do not regress
- **WHEN** a construct previously tracked as a gap has been implemented as rendered or progressive-reveal WYSIWYG (for example escaped punctuation, the supported inline-HTML subset, standalone HTML blocks, reference-style links, inline-dollar math, footnote and link-reference definitions, heading attributes, GFM alerts, or Visual Edit task-list checkbox click)
- **THEN** the coverage matrix classifies the construct in its implemented class and the construct does not reappear on the roadmap

#### Scenario: Secondary gaps are visible but lower priority
- **WHEN** a contributor evaluates whether to pick up a secondary gap (for example angle-bracket autolinks)
- **THEN** the roadmap lists the secondary gap with its effort and implementation seam
- **AND** the contributor can open a change that closes it without re-litigating whether it is a gap

#### Scenario: New gaps discovered in implementation are added to the roadmap
- **WHEN** implementation or testing reveals a Markdown construct that renders as raw source in Visual Edit and is not yet on the roadmap
- **THEN** the discovering change SHALL add the construct to this roadmap with its class, priority, effort, and seam before completing
- **AND** the change SHALL NOT close the gap in the same change unless the gap is trivial

#### Scenario: Task-list checkbox click is a closed gap
- **WHEN** a contributor reads the WYSIWYG coverage roadmap after this change
- **THEN** task-list checkbox click is absent from the open-gap list
- **AND** the coverage matrix records clickable Visual Edit task checkboxes as rendered WYSIWYG
