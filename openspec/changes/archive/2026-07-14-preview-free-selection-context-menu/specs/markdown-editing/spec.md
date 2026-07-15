## ADDED Requirements

### Requirement: Preview pane free-range text selection and copy
When the rendered preview pane is visible (Split Preview or Read mode), the editor SHALL allow the user to select textual content with the pointer across one or more contiguous preview blocks in document order (for example a heading together with following paragraphs, or multiple list items) and copy the selected plain text to the system clipboard via Copy (menu or shortcut). Selection and copy in the preview SHALL NOT mutate the document text, dirty flag, undo/redo history, or derived Markdown caches. The preview SHALL remain non-editable: cut, paste, and typing MUST NOT apply to preview content. A non-empty preview selection SHALL take copy precedence over the source editor selection.

#### Scenario: Drag-select across multiple preview blocks
- **WHEN** the rendered preview pane is visible and the user drag-selects from text in one preview block into text in a later or earlier block in document order
- **THEN** the selection covers the contiguous textual content between the drag start and end (partial first and last runs, full runs in between)
- **AND** the selection is highlighted across those runs
- **AND** the document text and derived Markdown state are unchanged

#### Scenario: Drag-select within a single preview text run
- **WHEN** the rendered preview pane is visible and the user drag-selects text within a single preview text run
- **THEN** the selected range is highlighted in that run
- **AND** the document text and derived Markdown state are unchanged

#### Scenario: Copy free-range selection as plain text
- **WHEN** a non-empty multi-block or single-run preview selection exists and the user invokes Copy (menu or shortcut)
- **THEN** the selected plain text (joined across covered runs in document order) is written to the system clipboard
- **AND** the document text, dirty flag, and undo/redo history are unchanged

#### Scenario: Preview selection takes copy precedence
- **WHEN** a non-empty preview text selection exists and the source editor also has a selection
- **THEN** Copy uses the preview selection's plain text rather than the source editor selection

#### Scenario: Read mode allows free-range copy but not edit
- **WHEN** the active view mode is Read and the user selects preview text spanning multiple blocks and copies it
- **THEN** the clipboard receives the selected plain text
- **AND** interacting with the preview still does not mutate the document text

#### Scenario: Link click still works alongside free-range selection
- **WHEN** the user clicks a preview link without creating a meaningful text selection
- **THEN** the link opens as before
- **AND** a drag that creates a non-empty selection does not open the link

### Requirement: Preview pane context menu with multi-format copy
When the rendered preview pane is visible, the editor SHALL provide a right-click context menu on the preview with actions to copy the current preview selection as plain text, as Markdown source, and as an HTML fragment. The menu SHALL also offer Select All for preview textual content, and Copy Link Address when the right-click resolves to a link URL. Context-menu actions SHALL NOT mutate the document text, dirty flag, undo/redo history, or derived Markdown caches.

#### Scenario: Right-click opens the preview context menu
- **WHEN** the preview pane is visible and the user right-clicks inside it
- **THEN** a context menu appears at the pointer with the localized copy and selection actions

#### Scenario: Copy as Markdown from a multi-block selection
- **WHEN** a non-empty preview selection covering one or more blocks exists and the user chooses Copy as Markdown
- **THEN** the clipboard receives Markdown source corresponding to the selected region (derived from document source ranges for the covered blocks)
- **AND** the document remains unmodified

#### Scenario: Copy as HTML from a preview selection
- **WHEN** a non-empty preview selection exists and the user chooses Copy as HTML
- **THEN** the clipboard receives an HTML fragment for that selection
- **AND** the document remains unmodified

#### Scenario: Copy as Plain Text from the context menu
- **WHEN** a non-empty preview selection exists and the user chooses Copy as Plain Text
- **THEN** the clipboard receives the same plain text that Edit→Copy would produce for that selection

#### Scenario: Copy actions disabled without a selection
- **WHEN** the preview context menu is open and there is no non-empty preview selection
- **THEN** Copy as Plain Text, Copy as Markdown, and Copy as HTML are unavailable (disabled or omitted)
- **AND** Select All remains available

#### Scenario: Select All selects the full preview text
- **WHEN** the user chooses Select All from the preview context menu
- **THEN** the preview selection covers all textual preview content for the active document from the first run to the last

#### Scenario: Copy Link Address when right-clicking a link
- **WHEN** the user right-clicks a preview link and chooses Copy Link Address
- **THEN** the clipboard receives that link's URL
- **AND** the document remains unmodified
