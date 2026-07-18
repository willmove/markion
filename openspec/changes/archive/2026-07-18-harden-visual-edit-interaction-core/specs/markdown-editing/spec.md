## MODIFIED Requirements

### Requirement: Source-backed Visual Edit mode
The editor SHALL provide a WYSIWYG-oriented Visual Edit mode that keeps supported Markdown content as close to its rendered result as can be edited through an exact, lossless source mutation, while preserving `MarkdownDocument.text` as the single canonical document representation. Visual Edit SHALL prefer direct rendered editing, SHALL reveal only the smallest complete source syntax needed for the active operation, and SHALL use a source-backed edit island only when an exact visual mutation or mapping cannot be proven. Visual Edit SHALL register a platform text-input target whenever its surface is active, including for an empty document, and Visual Edit mutations SHALL update the Markdown source text through the same dirty-state, undo/redo, autosave, recovery, and per-tab isolation paths as source editing. Every valid source caret position in a non-empty document, including whitespace-only gaps and trailing whitespace, SHALL have a source-backed visual editing affordance. Cursor-only interaction state, visual-list following, caret geometry, composition geometry, and navigation layout SHALL remain independent from document-version-derived caches.

#### Scenario: Visual prose editing updates Markdown source
- **WHEN** the user edits visible prose inside a paragraph, heading, blockquote, or list item in Visual Edit mode
- **THEN** the corresponding Markdown source text is updated
- **AND** the document dirty flag and undo history are updated through the existing document mutation path

#### Scenario: Exact constructs prefer rendered editing
- **WHEN** a Markdown construct has an exact source/display mapping and a lossless direct visual edit path
- **THEN** Visual Edit keeps the construct rendered during ordinary editing
- **AND** it does not replace the whole block with raw source solely because the construct is focused

#### Scenario: Platform text input reaches a non-empty visual document
- **WHEN** Visual Edit is active, the app editing focus is active, and the user enters normal platform text
- **THEN** the text replaces the current source selection or inserts at the current source caret
- **AND** the canonical Markdown text, dirty state, undo history, autosave, and recovery behavior follow the existing source-editing mutation path

#### Scenario: Empty visual document accepts first input
- **WHEN** Visual Edit is active for an empty document and the user enters platform text
- **THEN** the text is inserted into `MarkdownDocument.text`
- **AND** the new visual block is rendered without switching to source Edit mode

#### Scenario: Visual Edit supports IME composition
- **WHEN** the platform begins, updates, or commits an IME composition in Visual Edit
- **THEN** the existing marked-text range and source-backed replacement path are used
- **AND** GPUI receives visual range geometry for candidate-window placement when the active row has been laid out

#### Scenario: Visual formatting actions remain source-backed
- **WHEN** the user applies bold, italic, inline code, link, image, heading, list, task list, blockquote, or fenced-code formatting in Visual Edit mode
- **THEN** the editor updates the underlying Markdown markers in `MarkdownDocument.text`
- **AND** switching to Edit mode shows Markdown source that represents the visual result

#### Scenario: Focused syntax is exposed minimally
- **WHEN** the cursor enters visually formatted inline content whose hidden Markdown syntax is needed for precise editing
- **THEN** the editor SHALL expose the smallest complete source syntax group or source-backed edit island required for that focused content
- **AND** unrelated exact content in the same block remains rendered

#### Scenario: Complex or ambiguous constructs use conservative edit islands
- **WHEN** the user focuses a fenced code block, HTML/front matter region, image, or another construct without an exact direct visual edit path
- **THEN** the editor SHALL provide a source-backed editing affordance or preserve the existing source editing workflow
- **AND** the construct SHALL NOT be mutated through an ambiguous rendered-tree edit

#### Scenario: Whitespace caret positions remain editable
- **WHEN** the source caret moves into an empty line, a whitespace-only gap between rendered blocks, or trailing document whitespace in Visual Edit
- **THEN** the active whitespace range provides a visible source-backed caret or edit island
- **AND** inserting or deleting text at that position mutates the exact underlying Markdown range

#### Scenario: Cursor navigation reveals the active visual block
- **WHEN** keyboard navigation, text mutation, mode entry, search navigation, or an outline jump moves the source caret to a visual block outside the current viewport
- **THEN** the Visual Edit list scrolls enough to reveal that active block
- **AND** subsequent manual scrolling is not forced back to the caret unless another cursor-moving operation occurs

#### Scenario: Read mode remains non-editable
- **WHEN** Read mode is active and the user enters platform text or starts an IME composition
- **THEN** no Visual Edit input target mutates the document

#### Scenario: Visual-only interaction does not reparse unnecessarily
- **WHEN** the user moves the cursor, changes selection, scrolls the visual list, hovers text, focuses a visual edit island, updates visual caret/composition geometry, or records navigation layout without changing document text
- **THEN** the document version SHALL remain unchanged
- **AND** derived Markdown caches SHALL NOT be invalidated

## ADDED Requirements

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
