## MODIFIED Requirements

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

## ADDED Requirements

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
