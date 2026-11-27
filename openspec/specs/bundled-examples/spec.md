# bundled-examples Specification

## Purpose

Provide installed users with discoverable, offline Markdown examples and reusable templates for learning Markion and completing everyday writing tasks.

## Requirements

### Requirement: Installed offline example library
Every supported native distribution SHALL include `assets/examples` under its application resource root. The collection SHALL contain a bilingual index, Chinese getting-started instructions, focused format exercises, and at least ten distinct reusable templates covering daily notes, meetings, weekly reports, project plans, requirements, reading notes, knowledge articles, publishing drafts, troubleshooting, and release notes. Templates SHALL include meaningful starter sections and explicit replaceable placeholders. The collection SHALL be readable without network access and without creating or modifying any user documents at launch.

#### Scenario: First installation includes useful documents
- **WHEN** a user installs or extracts a supported Markion distribution
- **THEN** the complete example collection is available locally under `assets/examples`
- **AND** its index describes each guide and template and how to reuse it

### Requirement: Examples teach supported formats and editing workflows
The library SHALL demonstrate headings, paragraphs, inline emphasis/strong/strikethrough/code/highlight/subscript/superscript, quotes, separators, ordered/unordered/nested/task lists, Markdown tables, HTML tables with merged cells, supported HTML text formatting, local images, links, footnotes, fenced code, math, and supported diagrams. Guides SHALL include concrete editing exercises and explain source/split/read/visual modes, file and folder opening, save/reuse, search and replace, outline navigation, and export. Required images and relative document links SHALL resolve within the copied example directory; external links SHALL be optional references rather than required runtime resources.

#### Scenario: Offline practice after copying
- **WHEN** the user copies the complete examples directory into a writable location and opens it in Markion
- **THEN** the index and all relative sample links and images resolve from that copy
- **AND** the user can practice common editing and export tasks without downloading sample assets

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

### Requirement: Distribution verifies example completeness
Native package verification SHALL compare the packaged example collection with the checked-in collection and fail before publication if any required document or image is absent or altered.

#### Scenario: A package omits a template
- **WHEN** a supported package is inspected and its examples tree omits a checked-in template or contains different bytes
- **THEN** verification fails with the relative path of the missing or altered file

#### Scenario: A complete package passes verification
- **WHEN** every checked-in example document and image is present with matching contents in a supported package
- **THEN** the example completeness check succeeds alongside existing workspace verification
