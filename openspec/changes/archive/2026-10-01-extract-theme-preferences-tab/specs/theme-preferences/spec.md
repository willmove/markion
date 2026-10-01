## ADDED Requirements

### Requirement: Preferences panel SHALL expose a Theme tab
The Preferences panel SHALL provide a Theme tab at the same tab-strip level as General, Shortcuts, and Export. The tab order SHALL be General, Theme, Shortcuts, Export. Opening the Preferences panel from File → Preferences SHALL land on General. The Theme tab SHALL host the theme swatch grid and SHALL NOT host language, typography, display, auto-save, shortcut, or export controls.

#### Scenario: Theme tab sits beside the other Preferences tabs
- **WHEN** the user opens the Preferences panel
- **THEN** the tab strip offers General, Theme, Shortcuts, and Export as sibling tabs in that order

#### Scenario: Opening Preferences lands on General
- **WHEN** the user opens the Preferences panel from File → Preferences
- **THEN** the General tab is active

#### Scenario: Theme tab contains only theme selection
- **WHEN** the Theme tab is active
- **THEN** the tab body shows the theme swatch grid
- **AND** language, typography, display, auto-save, shortcut, and export controls are not rendered in that body

## MODIFIED Requirements

### Requirement: The Preferences panel SHALL let the user choose a theme by swatch
The system SHALL render a Theme tab in the Preferences panel containing a swatch grid where each theme (built-in plus any custom `.theme` files) is a card showing a preview of representative palette colors, the theme name, and a check mark on the active theme. Activating a card SHALL apply that theme immediately and persist the choice. The General tab SHALL NOT contain the swatch grid.

#### Scenario: Theme cards show a color preview and the active marker
- **WHEN** the Preferences panel Theme tab is open
- **THEN** each theme card displays a multi-segment color swatch drawn from the theme palette and shows a check mark only on the currently active theme

#### Scenario: Selecting a theme applies and persists it
- **WHEN** the user clicks a theme card
- **THEN** that theme becomes active immediately, the preferences file is updated with its name, and the active card receives a highlighted border

#### Scenario: Custom themes appear alongside built-ins
- **WHEN** custom `.theme` files exist in the themes directory
- **THEN** they appear in the swatch grid together with the built-in themes, with built-ins winning on name collisions

#### Scenario: General tab does not host the swatch grid
- **WHEN** the Preferences panel General tab is open
- **THEN** the theme swatch grid is not rendered in that tab

### Requirement: Preferences panel SHALL expose an Export tab
The Preferences panel SHALL provide an Export tab alongside General, Theme, and Shortcuts. The tab SHALL expose the PDF/DOCX backend choice (built-in writer, the default, vs. pandoc) as mutually exclusive controls that apply immediately and persist via the `[export] backend` config value. While the pandoc backend is selected, the tab SHALL additionally expose the pandoc binary path, the DOCX reference template (each with a native file picker and a reset action restoring the default), and the pandoc PDF engine. The tab SHALL show a pandoc-availability status probed in the background so rendering never spawns processes. The tab SHALL also carry the format option sections — Word page size, Word table of contents, Word image policy, PDF page size, PDF margin, PDF table of contents, and the PDF page-number footer — each mapped onto the corresponding persisted export option.

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

## REMOVED Requirements

### Requirement: Preferences panel SHALL show Language before Theme
**Reason**: Language and theme selection now live on sibling tabs (General vs Theme), so intra-tab section order no longer applies.
**Migration**: Language remains the first section of the General tab; theme selection is reached by choosing the Theme tab.
