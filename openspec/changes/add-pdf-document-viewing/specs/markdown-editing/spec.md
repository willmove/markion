## ADDED Requirements

### Requirement: Read-only PDF tabs SHALL preserve document editing isolation
The heterogeneous tab host SHALL support read-only PDF tabs alongside Markdown/text document tabs and image tabs. Generic tab operations SHALL use content-independent identity, path, title, dirty-state, navigation, replacement, and close behavior, while every Markdown editing or derivation path SHALL establish that its target is a document before accessing document state. A PDF tab SHALL be safe to replace and SHALL not contribute to unsaved-tab, autosave, recovery, undo, or document-cache behavior.

#### Scenario: Opening a PDF beside a dirty document preserves the document
- **WHEN** a dirty editable document remains open and a PDF is opened in another tab
- **THEN** switching between those tabs preserves the document's text, dirty flag, selection, undo/redo history, scroll positions, and derived Markdown cache identities
- **AND** the PDF acquires none of that document state

#### Scenario: Replacing a PDF needs no dirty guard
- **WHEN** the active tab contains a PDF and an open request resolves to replace-active
- **THEN** Markion replaces the PDF without an unsaved-changes prompt
- **AND** pending PDF results cannot populate the replacement tab

#### Scenario: Quit counts only dirty documents
- **WHEN** the user quits with any number of PDF and image tabs plus one or more dirty document tabs
- **THEN** quit confirmation is determined solely by the dirty document tabs
- **AND** PDFs do not create recovery files or autosave targets

#### Scenario: A single PDF uses the normal content area
- **WHEN** a PDF is the only open tab
- **THEN** the tab bar remains hidden and the PDF surface occupies the normal document-workspace area
