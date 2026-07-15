## ADDED Requirements

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
