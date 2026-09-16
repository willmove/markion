## MODIFIED Requirements

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

#### Scenario: Continued list row is visible immediately
- **WHEN** Enter continues a list item, including the final item before blank lines
- **THEN** the new empty item occupies a visible row below the preceding item and the painted caret moves into that row
- **AND** typing fills that item, another Enter on the empty item exits the list, and Undo restores the prior source and caret
- **AND** this holds for ordered, unordered and task lists with LF and CRLF source

#### Scenario: Exiting an empty list item preserves the visible caret row
- **WHEN** Enter removes a newly continued empty item's prefix after the first, middle or last list item
- **THEN** the blank row remains visible and the caret does not return to the previous item's text
- **AND** each subsequent Enter visibly advances the caret to a new source-backed row
- **AND** list items containing inline links retain the same blank-row caret behavior
