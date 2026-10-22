## Purpose

Covers opening, editing, and persisting structured data and configuration text documents — JSON, YAML, and TOML — as a distinct data-text document kind: source-only editing with whole-file grammar-based syntax highlighting, no Markdown derivation, and the same tab isolation and persistence guarantees as other text documents.

## ADDED Requirements

### Requirement: Data text documents SHALL open and edit as a distinct kind
The editor SHALL recognize files with the extensions `.json`, `.yaml`, `.yml`, and `.toml` (compared case-insensitively on the ASCII-lowercased form) as **data text documents**. Opening a data text file — from the file tree, File → Open, Open Recent, an external drag-and-drop, a CLI open intent, or session restore — SHALL load it as UTF-8 text into its own tab and classify it as a data text document for that tab's lifetime. Tabs for filesystem-backed content SHALL remain unique by file path, reusing an already-open tab for the same path. Renaming or saving a document under a data extension SHALL reclassify it as a data text document; saving under a Markdown or plain-text extension SHALL reclassify it accordingly.

#### Scenario: JSON file opens as a data text document
- **WHEN** the user opens a `.json` file from the file tree or File → Open
- **THEN** the file opens as UTF-8 text in its own tab (or focuses an existing tab for the same path) classified as a data text document

#### Scenario: YAML and TOML extensions are recognized
- **WHEN** the user opens a `.yaml`, `.yml`, or `.toml` file
- **THEN** the file is classified as a data text document, case-insensitively by extension

#### Scenario: An already-open data file is not duplicated
- **WHEN** an open request targets a data text file that is already open in another tab
- **THEN** the existing tab is focused instead of opening a duplicate tab

#### Scenario: Save As reclassifies the document
- **WHEN** the user saves an untitled document as `config.toml`, or saves an open Markdown document as `data.json` through Save As
- **THEN** the tab is reclassified to a data text document and subsequent editing follows this capability's rules

### Requirement: Data text documents SHALL bypass the Markdown pipeline
A data text document SHALL NOT be parsed as Markdown and SHALL NOT produce Markdown-derived state: no rendered preview blocks, no Markdown-derived outline, and no Visual Edit surface. Its editing surface SHALL be the source editing surface only; Read, Visual Edit, and Split Preview view modes SHALL NOT be offered for a data text document, and view-mode shortcuts or toggles invoked while a data text document is active SHALL leave the document on its source surface. Status-bar line and word counts for a data text document SHALL be computed from its plain text. When the active tab switches from a data text document to a Markdown or plain-text document, the previously selected view mode for that tab SHALL resume unchanged.

#### Scenario: A hash comment does not render as a heading
- **WHEN** a `.toml` or `.yaml` data document contains a `# comment` line
- **THEN** no rendered preview is produced for that document and no heading is derived from the comment

#### Scenario: View modes are restricted to the source surface
- **WHEN** a data text document is the active tab and the user invokes a Read, Visual Edit, or Split Preview view-mode shortcut or menu action
- **THEN** the document remains on its source editing surface and no preview or Visual Edit pane is created for it

#### Scenario: Outline does not show Markdown structure for data documents
- **WHEN** a data text document is active and the Outline sidebar is visible
- **THEN** the outline does not present Markdown-derived headings for that document

#### Scenario: Plain-text counts drive the status bar
- **WHEN** a data text document is active
- **THEN** the status bar reports line and word counts computed from the document's plain text

### Requirement: Data text documents SHALL apply whole-file grammar-based highlighting
A data text document SHALL be syntax-highlighted as one unit by resolving its file extension to a grammar in the bundled extended grammar registry (syntect defaults plus the two-face extended syntax set), classifying syntect scopes into the editor's theme-mapped token classes so every application theme continues to control the colors. A data document whose extension resolves to no grammar, or whose parse fails, SHALL render as plain text without error. Highlighting results SHALL be memoized per document version and reused across renders, recomputed only when the document version changes; the shared grammar registry SHALL remain lazily loaded with the existing background warm-up. Markdown and curated plain-text tabs SHALL NOT gain whole-file highlighting from this requirement.

#### Scenario: JSON document is highlighted by its grammar
- **WHEN** a `.json` data document containing strings, numbers, and booleans is open
- **THEN** those tokens are colored by theme-mapped token classes derived from the JSON grammar's syntect scopes

#### Scenario: TOML document is highlighted by its grammar
- **WHEN** a `.toml` data document containing keys, table headers, and comments is open
- **THEN** keys, table headers, and comments are colored by theme-mapped token classes derived from the TOML grammar

#### Scenario: Highlighting is memoized per version
- **WHEN** a data text document is rendered repeatedly without edits, then edited once
- **THEN** the memoized highlighting is reused across the renders before the edit and recomputed for the new document version

#### Scenario: Unresolvable extension falls back to plain text
- **WHEN** a data text document's extension resolves to no grammar in the registry or the grammar parse fails
- **THEN** the document renders as plain text without surfacing an error

### Requirement: Data text documents SHALL render with the code font
The source editing surface of a data text document SHALL use the editor's code font slot (with its monospace fallback chain) rather than the prose editor font slot. The code font slot's preference and theme resolution SHALL apply unchanged. Switching tabs SHALL NOT change the font slot of any other tab.

#### Scenario: Data document uses the code font slot
- **WHEN** a data text document is open beside a Markdown document and the user switches between the tabs
- **THEN** the data text document's source surface renders in the code font slot and the Markdown document's source surface keeps its existing editor font slot

### Requirement: Data text documents SHALL share editing and persistence behavior with text documents
A data text tab SHALL carry its own isolated document, cursor/selection, scroll position, undo/redo history, IME composition state, layout caches, dirty flag, and autosave/recovery tracking, and switching tabs SHALL NOT disturb another tab's state. Editing, saving, Save As, autosave, crash recovery, search and replace, and per-version derived-state caching rules SHALL behave exactly as for curated plain-text documents, except that the Markdown-derived caches are not produced. UTF-8 decode failures on open SHALL be handled by the existing open path's error reporting.

#### Scenario: Edits mark the document dirty and autosave persists
- **WHEN** the user edits an open `.json` document with autosave enabled
- **THEN** the tab is marked dirty and the changed bytes are persisted by the existing autosave pipeline

#### Scenario: Tab isolation holds for data documents
- **WHEN** the user edits and scrolls a `.toml` tab, then switches to another tab and back
- **THEN** the `.toml` tab's text, cursor, selection, scroll position, and undo history are unchanged

#### Scenario: Crash recovery restores data document text
- **WHEN** the editor restarts after a crash with unsaved changes to a data text document
- **THEN** the existing crash-recovery prompt offers the recovered data document text like any other text document
