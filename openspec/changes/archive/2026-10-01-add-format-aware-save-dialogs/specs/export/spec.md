## ADDED Requirements

### Requirement: Format-aware export destination dialogs
Each styled HTML, plain HTML, PDF, LaTeX, DOCX, PNG, and JPEG export action SHALL open a save dialog that identifies the selected output type and advertises that type's accepted filename extensions. The selected action SHALL remain authoritative for the exporter: filename text SHALL NOT select a different encoder. Before exporting, the editor SHALL preserve an accepted extension case-insensitively and SHALL replace a missing, empty, or incompatible final extension with the selected type's canonical extension.

The extension profiles SHALL be: styled HTML accepts `.html` and `.htm` with canonical `.html`; plain HTML accepts `.html` and `.htm` with canonical `.html` and keeps the `.plain.html` suggested suffix; PDF accepts/canonicalizes to `.pdf`; LaTeX accepts `.tex` and `.latex` with canonical `.tex`; DOCX accepts/canonicalizes to `.docx`; PNG accepts/canonicalizes to `.png`; and JPEG accepts `.jpg` and `.jpeg` with canonical `.jpg`.

#### Scenario: Export dialog identifies the selected type
- **WHEN** the user invokes a format-specific export action
- **THEN** the save dialog's title, file-type label, accepted extensions, and suggested filename correspond to that action's output profile

#### Scenario: Missing or incompatible export extension is normalized
- **WHEN** the user confirms an export path whose final extension is missing, empty, or not accepted by the selected output profile
- **THEN** the editor replaces the final extension with that profile's canonical extension before invoking the exporter

#### Scenario: Accepted export extension alias is preserved
- **WHEN** the user confirms a path using an accepted alias such as `.htm`, `.latex`, or `.jpeg` in any letter case
- **THEN** the editor preserves that path and exports the selected format to it

#### Scenario: Filename does not switch the exporter
- **WHEN** the user invokes PDF export but types a filename ending in an extension associated with another format
- **THEN** the editor normalizes the path to `.pdf` and invokes the PDF exporter rather than switching formats

#### Scenario: Plain HTML keeps its distinguishing suggestion
- **WHEN** the user invokes plain HTML export for a document whose stem is `report`
- **THEN** the dialog suggests `report.plain.html` while advertising the HTML extensions

#### Scenario: Export cancellation is non-destructive
- **WHEN** the user cancels a format-aware export dialog
- **THEN** no file is written and the document path, contents, dirty state, and undo history remain unchanged
