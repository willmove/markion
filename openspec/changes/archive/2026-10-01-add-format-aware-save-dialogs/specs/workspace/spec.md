## ADDED Requirements

### Requirement: Format-aware Markdown Save As
The editor SHALL treat Save As as saving Markion's canonical Markdown source document, open a save dialog that identifies the file type as Markdown, and advertise `.md`, `.markdown`, and `.mdown` as accepted extensions. The dialog SHALL suggest the current filename when one exists and `Untitled.md` otherwise. Before writing, the editor SHALL preserve an accepted extension case-insensitively and SHALL replace a missing, empty, or incompatible extension with `.md`. A successful Save As SHALL retain the existing document-path, dirty-state, recovery-file, and workspace-root update behavior.

#### Scenario: Save As identifies Markdown documents
- **WHEN** the user invokes Save As
- **THEN** the save dialog identifies Markdown as the target type and advertises `.md`, `.markdown`, and `.mdown`

#### Scenario: Save As supplies a canonical extension
- **WHEN** the user confirms a Save As path with no extension or an incompatible extension
- **THEN** the editor replaces the final extension with `.md` before writing the Markdown source

#### Scenario: Save As preserves a Markdown alias
- **WHEN** the user confirms a Save As path ending in `.md`, `.markdown`, or `.mdown` in any letter case
- **THEN** the editor preserves that path and saves the Markdown source to it

#### Scenario: Save As cancellation is non-destructive
- **WHEN** the user cancels the format-aware Save As dialog
- **THEN** the active document path, contents, dirty state, recovery state, workspace root, and undo history remain unchanged
