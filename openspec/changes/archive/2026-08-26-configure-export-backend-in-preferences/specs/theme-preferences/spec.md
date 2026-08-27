## ADDED Requirements

### Requirement: Preferences panel SHALL expose an Export tab
The Preferences panel SHALL provide an Export tab alongside General and Shortcuts. The tab SHALL expose the PDF/DOCX backend choice (built-in writer, the default, vs. pandoc) as mutually exclusive controls that apply immediately and persist via the `[export] backend` config value. While the pandoc backend is selected, the tab SHALL additionally expose the pandoc binary path, the DOCX reference template (each with a native file picker and a reset action restoring the default), and the pandoc PDF engine. The tab SHALL show a pandoc-availability status probed in the background so rendering never spawns processes. The tab SHALL also carry the format option sections — Word page size, Word table of contents, Word image policy, PDF page size, PDF margin, PDF table of contents, and the PDF page-number footer — each mapped onto the corresponding persisted export option.

#### Scenario: Backend choice applies and persists
- **WHEN** the user selects the built-in or pandoc backend in the Export tab
- **THEN** the next PDF/DOCX export uses that backend, the choice persists across restarts, and the previous choice is indicated as active

#### Scenario: Pandoc options appear only for the pandoc backend
- **WHEN** the backend selection changes between built-in and pandoc
- **THEN** the pandoc path, reference template, and PDF-engine rows appear or disappear accordingly without restarting the app

#### Scenario: Pandoc availability is probed off the UI thread
- **WHEN** the Export tab opens, the backend switches to pandoc, or the pandoc path changes
- **THEN** availability is probed on a background executor and the cached result renders as a status line without blocking or re-spawning on every frame

#### Scenario: File pickers write valid paths
- **WHEN** the user picks a pandoc binary or a reference template through the Export tab's file picker
- **THEN** the absolute path is stored and persisted; resetting restores the PATH lookup / bundled template respectively

#### Scenario: Format options map onto persisted export options
- **WHEN** the user changes page size, table of contents, image policy, margin, or page numbers in the Export tab
- **THEN** the change applies to the next export of that format and persists via the existing `[export.docx]` / `[export.pdf]` sections
