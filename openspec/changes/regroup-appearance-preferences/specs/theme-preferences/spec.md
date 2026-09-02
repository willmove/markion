## ADDED Requirements

### Requirement: Preferences panel SHALL expose an Appearance tab
The Preferences panel SHALL provide an Appearance tab at the same tab-strip level as General, Shortcuts, and Export. The tab order SHALL be General, Appearance, Shortcuts, Export. Opening the Preferences panel from File → Preferences SHALL land on General. The Appearance tab SHALL host the theme swatch grid and the document typography controls (Source font size, Reading font size, Paragraph spacing, and the source/rendered/code font-family slots). The Appearance tab SHALL NOT host language, display/workspace toggles, auto-save, shortcut, or export controls.

#### Scenario: Appearance tab sits beside the other Preferences tabs
- **WHEN** the user opens the Preferences panel
- **THEN** the tab strip offers General, Appearance, Shortcuts, and Export as sibling tabs in that order

#### Scenario: Opening Preferences lands on General
- **WHEN** the user opens the Preferences panel from File → Preferences
- **THEN** the General tab is active

#### Scenario: Appearance tab contains theme and typography
- **WHEN** the Appearance tab is active
- **THEN** the tab body shows the theme swatch grid and the typography controls
- **AND** language, display/workspace, auto-save, shortcut, and export controls are not rendered in that body

#### Scenario: General tab does not host appearance controls
- **WHEN** the Preferences panel General tab is open
- **THEN** the theme swatch grid is not rendered in that tab
- **AND** Source font size, Reading font size, Paragraph spacing, and the three font-family slots are not rendered in that tab

## MODIFIED Requirements

### Requirement: The Preferences panel SHALL let the user choose a theme by swatch
The system SHALL render an Appearance tab in the Preferences panel containing a swatch grid where each theme (built-in plus any custom `.theme` files) is a card showing a preview of representative palette colors, the theme name, and a check mark on the active theme. Activating a card SHALL apply that theme immediately and persist the choice. The General tab SHALL NOT contain the swatch grid.

#### Scenario: Theme cards show a color preview and the active marker
- **WHEN** the Preferences panel Appearance tab is open
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

### Requirement: Preferences panel SHALL expose document typography controls
The Preferences panel Appearance tab SHALL expose localized numeric controls for Source font size, Reading font size, and Paragraph spacing. Each control SHALL display its current logical-pixel value, provide decrement and increment actions in 1px steps, disable actions at the supported bound, use active-theme colors, apply a changed value immediately, and persist it through the existing preferences save path. The General tab SHALL NOT host these numeric typography controls.

#### Scenario: Typography controls show current values
- **WHEN** the Preferences panel Appearance tab is open
- **THEN** Source font size, Reading font size, and Paragraph spacing each render with a localized label, current pixel value, and minus/plus affordances
- **AND** the controls follow the active language and theme

#### Scenario: Numeric control applies and persists
- **WHEN** the user increments or decrements a typography control within its supported range
- **THEN** the affected document surfaces reflow immediately
- **AND** the normalized value is written to `config.toml`

#### Scenario: Numeric controls enforce bounds
- **WHEN** a typography value is at its minimum or maximum
- **THEN** the control disables the action that would move beyond that bound
- **AND** activating the disabled action does not rewrite preferences or change layout

#### Scenario: General tab does not host typography sizes
- **WHEN** the Preferences panel General tab is open
- **THEN** Source font size, Reading font size, and Paragraph spacing controls are not rendered in that tab

### Requirement: Preferences panel SHALL expose document font family controls

The Preferences panel Appearance tab typography section SHALL include one control per font slot (source, rendered, code) with localized labels consistent with the font-size controls. Each control SHALL present a follow-theme state and an explicit-family state: in the follow-theme state it SHALL indicate that the theme (or default) font applies; activating the control SHALL present a selection list populated from the fonts installed on the machine, each entry rendered in its own family as live preview, plus a follow-theme entry that clears the stored preference. Selecting an entry SHALL apply it immediately to that document plane and persist it. The control SHALL show an advisory warning when the currently stored family (for example hand-edited into `config.toml`) is not among the installed fonts. The General tab SHALL NOT host these font-family controls.

#### Scenario: Controls reflect the current slot state
- **WHEN** the Preferences panel Appearance tab is open
- **THEN** each of the three font controls shows either the follow-theme state (with the effective family named) or the user's explicit family for that slot

#### Scenario: The selection list enumerates installed fonts with live previews
- **WHEN** the user opens a slot's font control
- **THEN** the list presents a follow-theme entry followed by every font family installed on the machine, each rendered in its own family
- **AND** the entry matching the slot's current explicit family is marked active

#### Scenario: Selecting a family applies and persists immediately
- **WHEN** the user selects an installed family from a slot's list
- **THEN** that document plane re-renders with the new family on the next frame
- **AND** the choice is written to `config.toml` as the slot's `*_font_family` key
- **AND** the selection list closes

#### Scenario: Follow theme entry clears an explicit choice
- **WHEN** the user selects the follow-theme entry for a slot that had an explicit family
- **THEN** the stored preference is removed so the slot resolves from the active theme and then the default
- **AND** the plane re-renders accordingly and the list closes

#### Scenario: A stored but uninstalled family warns
- **WHEN** a slot's stored family (e.g. hand-edited into `config.toml`) is not installed on the machine
- **THEN** the control shows a localized advisory warning while keeping the value applied and persisted verbatim

#### Scenario: Controls follow language and theme
- **WHEN** the active language or theme changes
- **THEN** font control labels, states, and colors update on the next render

#### Scenario: General tab does not host font-family slots
- **WHEN** the Preferences panel General tab is open
- **THEN** the source, rendered, and code font-family controls are not rendered in that tab

### Requirement: Preferences panel SHALL expose an Export tab
The Preferences panel SHALL provide an Export tab alongside General, Appearance, and Shortcuts. The tab SHALL expose the PDF/DOCX backend choice (built-in writer, the default, vs. pandoc) as mutually exclusive controls that apply immediately and persist via the `[export] backend` config value. While the pandoc backend is selected, the tab SHALL additionally expose the pandoc binary path, the DOCX reference template (each with a native file picker and a reset action restoring the default), and the pandoc PDF engine. The tab SHALL show a pandoc-availability status probed in the background so rendering never spawns processes. The tab SHALL also carry the format option sections — Word page size, Word table of contents, Word image policy, PDF page size, PDF margin, PDF table of contents, and the PDF page-number footer — each mapped onto the corresponding persisted export option.

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
**Reason**: Language and appearance (theme plus typography) now live on sibling tabs (General vs Appearance), so intra-tab section order no longer applies.
**Migration**: Language remains the first section of the General tab; theme and typography are reached by choosing the Appearance tab.
