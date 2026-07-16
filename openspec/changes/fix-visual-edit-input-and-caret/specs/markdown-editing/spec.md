## MODIFIED Requirements

### Requirement: Source-backed Visual Edit mode
The editor SHALL provide a Visual Edit mode that presents common Markdown constructs in a rendered, editable form while preserving `MarkdownDocument.text` as the single canonical document representation. Visual Edit SHALL register a platform text-input target whenever its surface is active, including for an empty document, and Visual Edit mutations SHALL update the Markdown source text through the same dirty-state, undo/redo, autosave, recovery, and per-tab isolation paths as source editing. Every valid source caret position in a non-empty document, including whitespace-only gaps and trailing whitespace, SHALL have a source-backed visual editing affordance. Cursor-only interaction state, visual-list following, and caret geometry SHALL remain independent from document-version-derived caches.

#### Scenario: Visual prose editing updates Markdown source
- **WHEN** the user edits visible prose inside a paragraph, heading, blockquote, or list item in Visual Edit mode
- **THEN** the corresponding Markdown source text is updated
- **AND** the document dirty flag and undo history are updated through the existing document mutation path

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
- **AND** GPUI receives a visual caret rectangle for candidate-window placement when the active row has been laid out

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
- **WHEN** the user moves the cursor, changes selection, scrolls the visual list, hovers text, focuses a visual edit island, or updates visual caret geometry without changing document text
- **THEN** the document version SHALL remain unchanged
- **AND** derived Markdown caches SHALL NOT be invalidated
