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
Both the native and in-window Help menus SHALL offer a localized Examples and Templates action without a default shortcut. Invoking it SHALL reveal the bundled example index in the system file manager, dismiss the active menu, and leave the workspace and all open documents unchanged. A missing collection SHALL produce localized status feedback without crashing or creating an empty directory. The welcome document SHALL identify the Help entry. The index SHALL instruct users to copy examples before editing because installed resources may be read-only or replaced by upgrades, and SHALL explain that image-bearing copies must retain their resource directory.

#### Scenario: Open examples while editing
- **WHEN** the user invokes Help → Examples and Templates with unsaved edits in an open document
- **THEN** the system file manager reveals the bundled index
- **AND** document text, dirty flags, undo history, workspace selection, and derived caches are unchanged

#### Scenario: Examples are missing
- **WHEN** the example index cannot be found in application resources
- **THEN** Markion reports a localized missing-examples status and dismisses the menu
- **AND** no document or directory is created or replaced

### Requirement: Distribution verifies example completeness
Native package verification SHALL compare the packaged example collection with the checked-in collection and fail before publication if any required document or image is absent or altered.

#### Scenario: A package omits a template
- **WHEN** a supported package is inspected and its examples tree omits a checked-in template or contains different bytes
- **THEN** verification fails with the relative path of the missing or altered file

#### Scenario: A complete package passes verification
- **WHEN** every checked-in example document and image is present with matching contents in a supported package
- **THEN** the example completeness check succeeds alongside existing workspace verification
