## ADDED Requirements

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
