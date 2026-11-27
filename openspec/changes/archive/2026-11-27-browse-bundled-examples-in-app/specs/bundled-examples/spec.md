## MODIFIED Requirements

### Requirement: Discoverable and non-destructive example access
Both the native and in-window Help menus SHALL offer a localized Examples and Templates action without a default shortcut. Invoking it SHALL dismiss the menu and display a scrollable overview of the bundled index, guides, and templates within Markion without launching an external program. Each card SHALL include a content thumbnail and open its corresponding Markdown document when activated with the mouse or keyboard. Markion SHALL open an editable draft in a new tab or focus an existing draft of the same bundled document. Until the draft is saved, its bundled source SHALL provide the base for relative images and links to other examples. Saving a draft for the first time SHALL require the user to choose a destination. The welcome document and index SHALL describe the in-app gallery. Missing or unreadable resources SHALL produce localized feedback.

#### Scenario: Open examples while editing
- **WHEN** the user invokes Help → Examples and Templates with unsaved edits in an open document
- **THEN** Markion displays the thumbnail overview inside the current window
- **AND** opening or dismissing the overview preserves the active document, dirty flags, undo history, workspace selection, and derived caches

#### Scenario: Open a sample or template from its thumbnail
- **WHEN** the user activates a thumbnail
- **THEN** Markion opens that Markdown in a new editable tab or focuses the existing draft of that document
- **AND** the current workspace and the previously open documents remain available
- **AND** changes to the draft cannot autosave to the installed sample

#### Scenario: Follow bundled resources from a draft
- **WHEN** a newly opened bundled guide refers to an image or another bundled Markdown document by a relative path
- **THEN** its image resolves against the original sample directory and its Markdown link opens the target in Markion

#### Scenario: Examples are missing
- **WHEN** Markion cannot locate the example collection or read the selected document
- **THEN** Markion shows localized feedback without replacing the user's document or workspace

#### Scenario: Windows resource path contains spaces or non-ASCII characters
- **WHEN** the Windows resource path contains spaces or non-ASCII characters and the user activates a thumbnail
- **THEN** Markion reads the corresponding Markdown and opens it in the editor

#### Scenario: Windows file manager cannot launch
- **WHEN** the Windows file manager is unavailable and the user invokes Help → Examples and Templates
- **THEN** Markion displays the gallery and can open the selected bundled Markdown in the editor
