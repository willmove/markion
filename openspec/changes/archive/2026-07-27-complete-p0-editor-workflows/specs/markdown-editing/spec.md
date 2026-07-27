## ADDED Requirements

### Requirement: Visual Edit SHALL provide selection-contextual formatting controls
When Visual Edit owns a non-empty, exactly source-mapped text selection, Markion SHALL present contextual controls for strong emphasis, emphasis, inline code, and link editing. Invoking a control SHALL use the existing canonical Markdown mutation, semantic undo, selection, autosave, and exact UTF-8 source paths. Merely showing, moving, or dismissing the controls SHALL NOT change document state or invalidate derived caches.

#### Scenario: Selection toolbar formats visual text
- **WHEN** the user selects exactly mapped prose in Visual Edit and invokes Bold, Italic, or Inline Code from the contextual controls
- **THEN** the corresponding canonical Markdown markers are changed through one semantic command
- **AND** one Undo restores the prior source and selection

#### Scenario: Ambiguous selection stays conservative
- **WHEN** a selection crosses an ambiguous or source-island boundary
- **THEN** Markion does not present an unsafe contextual mutation for that range
- **AND** raw source editing remains available

### Requirement: Links SHALL have an exact source-backed visual editor
Creating or focusing an exactly mapped inline link SHALL provide a visual editor for label, URL, and optional title. Submitting the editor SHALL serialize one valid inline Markdown link and apply it as one source mutation. Canceling or changing focus SHALL leave the authored source byte-for-byte unchanged. Reference-style, malformed, or crossing links SHALL retain conservative source editing.

#### Scenario: Selected text creates a link
- **WHEN** the user selects exactly mapped visual prose, opens the link editor, enters a URL and optional title, and confirms
- **THEN** the selected text becomes the link label in one canonical-source mutation
- **AND** the resulting selection and source ranges remain UTF-8 safe

#### Scenario: Existing inline link is edited
- **WHEN** the caret is within an exactly mapped inline link and the user changes its URL or title
- **THEN** the complete link source is replaced once while preserving its visible label unless edited
- **AND** one Undo restores the complete prior link and selection

#### Scenario: Link edit is canceled
- **WHEN** the visual link editor is dismissed without confirmation
- **THEN** document text, version, dirty state, selection, undo history, and derived cache identity remain unchanged

