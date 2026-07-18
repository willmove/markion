## MODIFIED Requirements

### Requirement: Source-backed Visual Edit mode
The editor SHALL provide a WYSIWYG-oriented Visual Edit mode that keeps supported Markdown content as close to its rendered result as can be edited through an exact, lossless source mutation, while preserving `MarkdownDocument.text` as the single canonical document representation. Visual Edit SHALL prefer direct rendered editing, including dedicated field or payload editors for exactly ranged complex blocks, SHALL reveal only the smallest complete source syntax needed for the active operation, and SHALL use a source-backed edit island only when an exact visual mutation or mapping cannot be proven. Visual Edit SHALL register a platform text-input target whenever its surface is active, including for an empty document, and Visual Edit mutations SHALL update the Markdown source text through the same dirty-state, undo/redo, autosave, recovery, and per-tab isolation paths as source editing. Every valid source caret position in a non-empty document, including whitespace-only gaps and trailing whitespace, SHALL have a source-backed visual editing affordance. Cursor-only interaction state, visual-list following, direct-widget focus, caret geometry, composition geometry, and navigation layout SHALL remain independent from document-version-derived caches.

#### Scenario: Visual prose editing updates Markdown source
- **WHEN** the user edits visible prose inside a paragraph, heading, blockquote, or list item in Visual Edit mode
- **THEN** the corresponding Markdown source text is updated
- **AND** the document dirty flag and undo history are updated through the existing document mutation path

#### Scenario: Exact constructs prefer rendered editing
- **WHEN** a Markdown construct has an exact source/display mapping and a lossless direct visual edit path
- **THEN** Visual Edit keeps the construct rendered during ordinary editing
- **AND** it does not replace the whole block with raw source solely because the construct is focused

#### Scenario: Exact complex blocks use dedicated editors
- **WHEN** an ordinary fenced code block, block-math construct, inline Markdown image, or GFM table has proven exact field or payload ranges
- **THEN** Visual Edit presents its rendered block together with the dedicated direct editing controls defined for that construct
- **AND** each control mutates only validated canonical source ranges through the shared application edit path

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
- **WHEN** the user focuses an HTML/front-matter region, registered diagram fence, malformed image/table, unclosed fence, or another construct without an exact direct visual edit path
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
- **WHEN** the user moves the cursor, changes selection, scrolls the visual list, hovers text, focuses a visual edit island or direct block field, updates visual caret/composition geometry, or records navigation layout without changing document text
- **THEN** the document version SHALL remain unchanged
- **AND** derived Markdown caches SHALL NOT be invalidated

## ADDED Requirements

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
