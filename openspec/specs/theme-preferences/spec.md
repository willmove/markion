# theme-preferences

## Purpose

Covers the in-app **Preferences panel** surface for choosing a theme (swatch grid) and an interface language (pill list), together with the persistence-format contract for those two choices. The built-in theme catalog itself (the 14 themes, ordering invariant, custom `.theme` loading) is described under `chrome-platform`; this capability focuses on the selection UI and the persisted-preferences contract.

## Requirements

### Requirement: The editor SHALL ship a fixed catalog of built-in themes
The system SHALL provide a built-in theme catalog of at least fourteen themes returned by `builtin_theme_definitions()`, covering the original six (Paper, Ink, Solar, Forest, Rose, Graphite) plus popular editor palettes (GitHub Light/Dark, Solarized Light/Dark, One Light/Dark, Tokyo Night/Light). Each theme SHALL carry a stable name, a dark/light flag, and a `ThemeColors` palette. Theme names are identity keys written to the preferences file, so renames SHALL be avoided to prevent orphaning saved selections.

#### Scenario: Built-in catalog includes the legacy themes first
- **WHEN** `builtin_theme_definitions()` is called
- **THEN** the first six entries are Paper, Ink, Solar, Forest, Rose, Graphite in that exact order, so the legacy `cycle_theme` path and its test continue to hold

#### Scenario: Built-in catalog includes popular editor palettes
- **WHEN** the preferences panel enumerates available themes
- **THEN** the list includes GitHub Light, GitHub Dark, Solarized Light, Solarized Dark, One Light, One Dark, Tokyo Night, and Tokyo Night Light, each with a unique name

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

### Requirement: Theme and language choices SHALL persist in a single preferences file
The system SHALL persist the selected theme and the selected interface language as lines in the same preferences file (`theme=<name>`, `language=<code>`). Reading the preferences file SHALL tolerate the absence of either line by applying the documented default, and SHALL tolerate unknown values by falling back to the default.

#### Scenario: Language and theme round-trip through the preferences file
- **WHEN** preferences with a theme of `Forest` and a language of `zh` are saved and reloaded
- **THEN** both values are restored exactly

#### Scenario: Older preferences files without a language line upgrade gracefully
- **WHEN** a preferences file written by an older build (no `language=` line) is loaded
- **THEN** the language defaults to English and the theme is honored as before

### Requirement: Custom themes SHALL be authored as TOML files

User-authored (custom) themes SHALL be stored as `.toml` files in the themes directory, with a `name`, an `is_dark` flag, and a `[colors]` sub-table carrying the eight `ThemeColors` keys (`app_bg`, `panel_bg`, `surface_bg`, `text`, `muted`, `border`, `active_bg`, `active_text`). Color values SHALL be written as `"#rrggbb"` strings and SHALL deserialize leniently (a leading `#` is optional), so a hand-edited `app_bg = "10131a"` loads the same as `app_bg = "#10131a"`. Every color key SHALL be `#[serde(default)]` so a partial file loads with the fallback palette rather than failing. A theme MAY additionally carry a `[fonts]` sub-table with optional string keys `editor`, `rendered`, and `code`, each naming a font family that the theme contributes to the corresponding document plane slot when the user has no explicit preference for it; absent keys and an absent `[fonts]` table SHALL mean the theme specifies no font for that slot, and an empty or whitespace-only value SHALL be treated as absent. When the editor loads a custom theme and finds a legacy `.theme` (`key=value`) file of the same stem with no `.toml` beside it, it SHALL parse the legacy file, write out an equivalent `.toml`, leave the legacy `.theme` in place, and log the migration — the legacy file is then ignored on subsequent loads. Listing the themes directory SHALL dedupe by file stem so a migrated pair (`midnight.theme` + `midnight.toml`) surfaces as a single theme.

#### Scenario: A TOML custom theme round-trips

- **WHEN** a `midnight.toml` with `name = "Midnight"`, `is_dark = true`, and all eight `[colors]` keys is saved and reloaded
- **THEN** every color value is restored exactly, and the theme appears in `available_themes()` as a custom entry

#### Scenario: A partial TOML theme loads with the fallback palette

- **WHEN** a `.toml` custom theme omits some `[colors]` keys (e.g. only `app_bg` and `text` are present)
- **THEN** the missing keys take the default `ThemeColors` values and the file still loads

#### Scenario: Theme fonts round-trip and apply per slot

- **WHEN** a `.toml` custom theme carries `[fonts]` with `rendered = "Georgia"` and the user has no explicit rendered-font preference
- **THEN** the theme loads with that font recorded for the rendered slot, the value survives a save/reload round-trip, and rendered body text uses Georgia while that theme is active

#### Scenario: A theme without fonts or with partial fonts loads unchanged

- **WHEN** a `.toml` custom theme has no `[fonts]` table, or a `[fonts]` table listing only some keys
- **THEN** the theme loads exactly as before this change and unspecified slots fall back to their defaults (no theme contribution)

#### Scenario: Empty font values are treated as absent

- **WHEN** a `[fonts]` key is present but empty or whitespace-only
- **THEN** the theme loads successfully and that slot behaves as if the key were absent

#### Scenario: First use installs a sample theme with fonts

- **WHEN** the themes directory does not exist and the Preferences panel first opens
- **THEN** the editor creates the directory and writes a sample `typewriter.toml` (light palette) whose `[fonts]` table demonstrates the optional editor/rendered/code contributions
- **AND** the sample loads as a selectable custom theme alongside the built-ins; an existing themes directory is never modified

#### Scenario: A legacy `.theme` file migrates once to TOML

- **WHEN** the themes directory contains a `midnight.theme` (`key=value`) but no `midnight.toml`
- **THEN** the first load parses the legacy file, writes a `midnight.toml` next to it, and returns the migrated theme; the `midnight.theme` is left in place
- **AND** on the next load, the `midnight.toml` is read directly and the `midnight.theme` is not parsed again

#### Scenario: A migrated pair surfaces as a single theme

- **WHEN** the themes directory contains both `midnight.theme` and `midnight.toml`
- **THEN** `list_theme_definitions` returns exactly one `Midnight` entry, sourced from the `.toml`

### Requirement: Preferences panel SHALL expose Preview adaptive width
The Preferences panel SHALL include a Preview adaptive width toggle in its non-theme display settings. Activating the toggle SHALL apply the rendered-content width behavior immediately across Read mode and Visual Edit mode and persist the preference. The panel SHALL additionally include a Sync scroll toggle in the same display settings; activating it SHALL apply the Split Preview proportional scroll-coupling behavior immediately and persist the preference.

#### Scenario: Toggle appears in Preferences panel
- **WHEN** the Preferences panel is open
- **THEN** it shows a Preview adaptive width toggle and a Sync scroll toggle, each reflecting its current enabled/disabled state

#### Scenario: Toggling Preview adaptive width applies immediately
- **WHEN** the user toggles Preview adaptive width
- **THEN** the active app state updates immediately
- **AND** Read mode and Visual Edit mode rendered-content layout reflects the new setting on the next render

#### Scenario: Toggling Preview adaptive width persists
- **WHEN** the user toggles Preview adaptive width
- **THEN** the preferences file is updated with the new boolean value

#### Scenario: Toggling Sync scroll applies immediately
- **WHEN** the user toggles Sync scroll
- **THEN** Split Preview proportional scroll coupling reflects the new setting on the next render

### Requirement: Preferences panel SHALL expose supported display settings as controls
The Preferences panel SHALL expose focus mode, typewriter mode, code line numbers, Preview adaptive width, and sidebar visibility as interactive controls. Files/Outline choices SHALL select the shared left sidebar tab when Outline position is Left and independently toggle the corresponding pane when it is Right. Controls SHALL apply immediately and persist through the existing preferences file.

#### Scenario: Boolean settings are editable in the panel
- **WHEN** the Preferences panel is open
- **THEN** focus mode, typewriter mode, code line numbers, Preview adaptive width, and sidebar visibility each render as an actionable control showing the current state

#### Scenario: Toggling a setting applies immediately
- **WHEN** the user activates a boolean Preferences control
- **THEN** the corresponding app state changes immediately
- **AND** the new value is persisted to the preferences file

#### Scenario: Sidebar tab is editable in the panel
- **WHEN** the Preferences panel is open and Outline position is Left
- **THEN** the sidebar tab preference renders as a mutually exclusive Files/Outline choice that indicates the current tab

#### Scenario: Selecting a sidebar tab applies immediately
- **WHEN** the user selects a different sidebar tab in the Preferences panel while Outline position is Left
- **THEN** the app switches the sidebar to that tab, keeps the sidebar visible, and persists the new tab

#### Scenario: Right layout exposes independent visibility controls
- **WHEN** Outline position is Right
- **THEN** the Files and Outline preference controls reflect and toggle their own pane visibility independently
- **AND** both controls can be active simultaneously
- **AND** the master Sidebar control hides both visible panes or shows both when neither is visible

### Requirement: Preferences panel controls SHALL use editable affordances
The Preferences panel SHALL render configurable values with button-like or segmented-control affordances instead of plain `on` / `off` summary text. The controls SHALL use localized labels and active-theme colors.

#### Scenario: Boolean values are not plain text only
- **WHEN** the Preferences panel renders a boolean setting
- **THEN** the enabled and disabled states are presented as clickable button-like controls

#### Scenario: Controls follow active language and theme
- **WHEN** the active language or theme changes
- **THEN** Preferences panel control labels and colors update on the next render

### Requirement: Preferences panel SHALL expose Heading menu depth
The Preferences panel SHALL include a Heading menu depth control in its non-theme display settings with two choices: **H1–H5** (default) and **H1–H6**. Activating a choice SHALL apply the setting immediately, update Format menu contents, reinstall native menus, and persist the preference.

#### Scenario: Control appears in Preferences panel
- **WHEN** the Preferences panel is open
- **THEN** it shows the Heading menu depth control with the current choice highlighted

#### Scenario: Selecting H1–H6 applies immediately
- **WHEN** the user selects H1–H6
- **THEN** the Format menu includes H4–H6 on the next render and native menus are reinstalled

#### Scenario: Selecting H1–H5 applies immediately
- **WHEN** the user selects H1–H5 after using H1–H6
- **THEN** H6 disappears from the Format menu on the next render while H4 and H5 remain visible, and native menus are reinstalled

#### Scenario: Choice persists
- **WHEN** the user selects H1–H6 and restarts the editor
- **THEN** Heading menu depth remains H1–H6

### Requirement: Heading menu depth preference persistence
The editor SHALL persist Heading menu depth in `config.toml` as `heading_menu_max_level` with allowed values `5` or `6`. Missing or invalid values SHALL default to `5`. The preference SHALL be included in preferences reset behavior.

#### Scenario: Missing preference defaults to H1–H5
- **WHEN** `config.toml` omits `heading_menu_max_level`
- **THEN** the editor starts with Heading menu depth H1–H5

#### Scenario: Invalid value defaults to H1–H5
- **WHEN** `heading_menu_max_level` is present but not `5` or `6`
- **THEN** the editor treats the value as `5`

#### Scenario: Reset restores H1–H5
- **WHEN** the user resets preferences
- **THEN** Heading menu depth returns to H1–H5

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

### Requirement: Document typography preferences SHALL persist safely

The editor SHALL persist source font size as `editor_font_size`, rendered font size as `rendered_font_size`, and rendered paragraph spacing as `paragraph_spacing` in `config.toml`, with defaults of 14px, 14px, and 12px respectively. Font sizes SHALL normalize to 10–32px inclusive and paragraph spacing SHALL normalize to 0–32px inclusive. The editor SHALL additionally persist optional font-family preferences as string keys `editor_font_family`, `rendered_font_family`, and `code_font_family` in `config.toml`; each key SHALL be optional, an absent, empty, or whitespace-only value SHALL mean "follow the active theme, then the built-in default", and a present value SHALL be persisted and reloaded verbatim without validating it against the set of installed fonts. Missing or non-numeric size/spacing fields SHALL use their defaults, numeric out-of-range fields SHALL clamp to the nearest bound, and reset SHALL restore all three size/spacing defaults and clear all three font-family preferences to the follow-theme state.

#### Scenario: Typography values round-trip

- **WHEN** preferences containing `editor_font_size = 18`, `rendered_font_size = 20`, and `paragraph_spacing = 16` are saved and reloaded
- **THEN** all three values are restored exactly and reflected by the Preferences controls

#### Scenario: Older config uses current defaults

- **WHEN** an existing `config.toml` omits all typography fields
- **THEN** the editor starts with 14px source text, 14px rendered body text, and 12px rendered paragraph spacing

#### Scenario: Invalid and out-of-range values are safe

- **WHEN** typography fields are non-numeric or outside their supported ranges
- **THEN** non-numeric values use defaults and numeric values clamp to their nearest supported bound
- **AND** the preferences file does not prevent the editor from starting

#### Scenario: Font-family preferences round-trip verbatim

- **WHEN** `config.toml` contains `editor_font_family = "Cascadia Code"` and `rendered_font_family = "Source Serif 4"` and preferences are saved and reloaded
- **THEN** both values are restored exactly and apply to their document planes

#### Scenario: Absent font-family keys follow the theme

- **WHEN** `config.toml` omits all three font-family keys or contains an empty value for one
- **THEN** the corresponding slot resolves from the active theme's `[fonts]` contribution, if any, and otherwise from the built-in default

#### Scenario: Reset restores typography defaults and clears font choices

- **WHEN** the user resets preferences after changing typography and font families
- **THEN** Source font size returns to 14px, Reading font size returns to 14px, and Paragraph spacing returns to 12px
- **AND** all three font-family preferences clear to the follow-theme state
- **AND** visible document surfaces reflow to those defaults

#### Scenario: Preferences summary includes typography

- **WHEN** the user opens the preferences summary
- **THEN** it reports the current source font size, rendered font size, and paragraph spacing using localized labels and pixel values

### Requirement: Preferences panel SHALL expose a Show hidden files toggle

The Preferences panel SHALL include a Show hidden files/folders toggle in its non-theme display settings, reflecting the current enabled/disabled state. Activating the toggle SHALL apply the new file-tree hidden-entry visibility behavior immediately by re-scanning the workspace, and SHALL persist the preference. The toggle SHALL default to off.

#### Scenario: Toggle appears in the Preferences panel
- **WHEN** the Preferences panel is open
- **THEN** it shows a Show hidden files/folders toggle in the display-settings area, rendered as an actionable control reflecting the current state

#### Scenario: Toggling applies immediately and re-scans the tree
- **WHEN** the user activates the Show hidden files/folders toggle
- **THEN** the workspace is re-scanned under the new visibility rule
- **AND** the file tree updates to reveal or hide hidden entries on the next render

#### Scenario: Toggling persists
- **WHEN** the user toggles Show hidden files/folders
- **THEN** the preferences file is updated with the new boolean value

### Requirement: Show hidden files preference SHALL persist safely

The editor SHALL persist the Show-hidden-files preference in `config.toml` as a boolean, defaulting to off. A missing field SHALL default to off, and a non-boolean field SHALL degrade to off rather than preventing the editor from starting. The preference SHALL be included in preferences reset behavior.

#### Scenario: Missing preference defaults to off
- **WHEN** `config.toml` omits the Show-hidden-files field
- **THEN** the editor starts with hidden-entry visibility off

#### Scenario: Invalid value defaults to off
- **WHEN** the Show-hidden-files field is present but not a valid boolean
- **THEN** the editor treats the value as off and the preferences file does not prevent the editor from starting

#### Scenario: Value round-trips
- **WHEN** preferences with Show-hidden-files enabled are saved and reloaded
- **THEN** the enabled state is restored exactly and reflected by the Preferences control

#### Scenario: Reset restores off
- **WHEN** the user resets preferences after enabling Show-hidden-files
- **THEN** the preference returns to off and hidden entries are omitted from the tree on the next scan

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

### Requirement: Preferences language picker SHALL contain variable-width labels
The Preferences panel SHALL render every entry from `Language::all()` as a complete interactive pill whose active check mark and native-language name remain inside that pill's border across supported display scales and variable system UI-font metrics. Language pills MUST NOT shrink below their marker-and-label content width; when the row lacks sufficient width, it SHALL wrap complete pills instead of clipping text, painting into adjacent controls, or compressing labels below their content width.

#### Scenario: Wide UI font keeps labels inside their pills
- **WHEN** the Preferences panel is rendered with a wide or monospaced system UI font
- **THEN** every native-language name and the active check mark remain fully contained by their own pill border without overlapping a neighbor

#### Scenario: Constrained width wraps complete pills
- **WHEN** the available language-row width is less than the combined intrinsic width of all language pills
- **THEN** complete pills wrap onto an additional row and every language remains visible and interactive
- **AND** no pill shrinks its marker or native-language name beyond its content width

#### Scenario: Active marker uses compact stable spacing
- **WHEN** the user selects any supported interface language
- **THEN** exactly that language pill shows a check mark in a dedicated leading slot separated from its native-language name by a compact fixed visual gap
- **AND** inactive pills reserve the same leading slot without inserting whitespace characters into their labels, so language names do not shift when selection changes

#### Scenario: Sufficient width retains a single comfortable row
- **WHEN** the General Preferences panel has sufficient logical width for all language pills, including at 125% Windows display scaling
- **THEN** all language pills appear on one row with their configured minimum width and complete marker-and-label content

### Requirement: Backup and Sync preferences SHALL separate ordinary choices from advanced Git settings
Preferences SHALL expose an ordinary Backup and Sync section for connection state, human-readable sync location, opt-in checks for updates from other devices, disconnect, and recovery access. The background-check label and description SHALL state that checking does not automatically upload or apply changes. Missing settings SHALL preserve existing editing behavior and keep background checking off; background write synchronization SHALL not be offered in this change.

Git executable detection/override, branch/endpoints, approved roots, message template, commit author identity, raw diagnostics, and transport details SHALL remain accessible under Advanced Git Settings or troubleshooting rather than appearing as ordinary sync choices. Repository-specific settings SHALL be persisted atomically in a versioned local policy file separate from session data and Git-owned remote/upstream configuration. Global and per-repository background settings SHALL identify their respective default/current-workspace scope.

#### Scenario: Existing installation upgrades
- **WHEN** the app loads configuration without Git keys or a repository policy file
- **THEN** it retains existing preferences and does not connect or synchronize any repository automatically

#### Scenario: Git override is invalid
- **WHEN** the user selects an unusable executable path
- **THEN** Advanced Git Settings report the failed detection without making ordinary document operations unavailable

#### Scenario: Ordinary settings are opened
- **WHEN** the user opens the General preferences category
- **THEN** it shows one Backup and Sync section using note/device language and keeps executable selection behind an Advanced Git disclosure

#### Scenario: Background checking is offered
- **WHEN** the user reviews the background update-check option
- **THEN** the interface states that it checks for updates from other devices and does not automatically upload or apply them

#### Scenario: Repository-specific background setting is shown
- **WHEN** the current workspace has a per-repository background-check override
- **THEN** preferences distinguish that workspace setting from the global default rather than presenting two unlabeled equivalent toggles

### Requirement: Repository policies SHALL outlive recent sessions and fail closed
Recent-workspace eviction and general Preferences reset SHALL NOT delete repository policies, Git history, credentials or unresolved recovery. Disconnect SHALL remove the sync binding/scheduling without deleting files, `.git` or remotes. Invalid/unsupported policy data SHALL disable writes with actionable feedback rather than broaden allowed scope. Replaced/moved repository identities SHALL require reconnection before using prior authorization.

#### Scenario: Workspace falls out of recent list
- **WHEN** a connected workspace is evicted from the bounded recent-workspace list
- **THEN** its separate repository policy remains available when that same repository is opened again

#### Scenario: Preferences are reset
- **WHEN** the user resets general preferences
- **THEN** global Git settings return to defaults and existing repository policies and recovery remain intact

#### Scenario: Policy cannot be parsed
- **WHEN** the repository policy is corrupted or has an unsupported version
- **THEN** one-click writes are disabled with repair feedback and no permissive replacement policy is silently generated

### Requirement: Preferences panel SHALL expose silent save and auto-save delay
The Preferences panel General tab SHALL include an Auto-save section with (1) a boolean control for silent save-to-file mapped to `[auto_save] silent_save` (default on), and (2) a numeric control for the inactivity interval mapped to `[auto_save] delay_secs` (default 5 seconds, minimum 1). Activating either control SHALL apply immediately, persist through the existing preferences save path, and use localized labels. The panel SHALL NOT expose `[auto_save] enabled`.

#### Scenario: Silent-save toggle appears and persists
- **WHEN** the Preferences panel General tab is open
- **THEN** a silent save-to-file control is visible and reflects the current `silent_save` value
- **AND** toggling it updates subsequent autosave destination behavior and writes `silent_save` to `config.toml`

#### Scenario: Delay control appears and persists
- **WHEN** the user adjusts the auto-save delay control within the supported range
- **THEN** the new `delay_secs` value is applied to subsequent inactivity timers and persisted
- **AND** values below 1 are not stored (control disables decrement at the minimum or clamps on commit)

#### Scenario: Master enabled switch stays out of the panel
- **WHEN** the Preferences panel General tab is open
- **THEN** no control is offered for `[auto_save] enabled`

### Requirement: Preferences panel SHALL expose an Appearance tab
The Preferences panel SHALL provide an Appearance tab at the same tab-strip level as General, Shortcuts, Export, and AI. General, Appearance, Shortcuts, and Export SHALL retain that relative order; additional capability-owned tabs such as Images SHALL retain their existing placement, and AI SHALL be appended after Export. Opening the Preferences panel from File → Preferences SHALL land on General. The Appearance tab SHALL host the theme swatch grid and the document typography controls (Source font size, Reading font size, Paragraph spacing, and the source/rendered/code font-family slots). The Appearance tab SHALL NOT host language, display/workspace toggles, auto-save, shortcut, export, or AI controls.

#### Scenario: Appearance tab sits beside the other Preferences tabs
- **WHEN** the user opens the Preferences panel
- **THEN** General, Appearance, Shortcuts, and Export appear in their existing relative order, other existing tabs retain their placement, and AI is appended after Export

#### Scenario: Opening Preferences lands on General
- **WHEN** the user opens the Preferences panel from File → Preferences
- **THEN** the General tab is active

#### Scenario: Appearance tab contains theme and typography
- **WHEN** the Appearance tab is active
- **THEN** the tab body shows the theme swatch grid and the typography controls
- **AND** language, display/workspace, auto-save, shortcut, export, and AI controls are not rendered in that body

#### Scenario: General tab does not host appearance controls
- **WHEN** the Preferences panel General tab is open
- **THEN** the theme swatch grid is not rendered in that tab
- **AND** Source font size, Reading font size, Paragraph spacing, and the three font-family slots are not rendered in that tab

### Requirement: Preferences panel SHALL expose Keep inline syntax hidden

The Preferences panel SHALL include a Keep inline syntax hidden toggle in its non-theme display settings, reflecting the current enabled or disabled state. Activating the toggle SHALL apply the Visual Edit inline-projection behavior on the next render and SHALL persist the preference. The toggle SHALL default to off. Activating it SHALL NOT change document text, document version, dirty state, undo history, or derived Markdown caches.

#### Scenario: Toggle appears in the Preferences panel

- **WHEN** the Preferences panel is open
- **THEN** it shows a Keep inline syntax hidden toggle in the display-settings area, rendered as an actionable control reflecting the current state

#### Scenario: Toggling applies on the next Visual Edit render

- **WHEN** the user activates Keep inline syntax hidden while a Visual Edit document is open
- **THEN** the active app state updates immediately
- **AND** the next Visual Edit render uses the new inline-projection behavior
- **AND** document text and document version stay unchanged

#### Scenario: Toggling persists

- **WHEN** the user toggles Keep inline syntax hidden
- **THEN** the preferences file is updated with the new boolean value

### Requirement: Keep inline syntax hidden SHALL persist safely

The editor SHALL persist Keep inline syntax hidden in `config.toml` as the boolean `stable_visual_inline`, defaulting to off. A missing field SHALL default to off, and a non-boolean field SHALL degrade to off rather than preventing the editor from starting. The preference SHALL be included in preferences reset behavior.

#### Scenario: Missing preference defaults to off

- **WHEN** `config.toml` omits `stable_visual_inline`
- **THEN** the editor starts with Keep inline syntax hidden off
- **AND** Visual Edit keeps the existing caret-triggered inline reveal behavior

#### Scenario: Invalid value defaults to off

- **WHEN** `stable_visual_inline` is present but not a valid boolean
- **THEN** the editor treats the value as off and the preferences file does not prevent the editor from starting

#### Scenario: Value round-trips

- **WHEN** preferences with `stable_visual_inline` enabled are saved and reloaded
- **THEN** the enabled state is restored exactly and reflected by the Preferences control

#### Scenario: Reset restores off

- **WHEN** the user resets preferences after enabling Keep inline syntax hidden
- **THEN** the preference returns to off
- **AND** Visual Edit returns to caret-triggered inline reveal on the next render

### Requirement: Preferences panel SHALL expose Outline position

General preferences SHALL provide localized Left and Right outline-position choices using active-theme colors. Left SHALL use the existing Files/Outline tabs in the left sidebar. Right SHALL show the file tree on the left and a separate Outline pane on the right simultaneously, with a single content-specific header on each pane. Changing the choice SHALL apply immediately.

#### Scenario: Default placement and choices
- **WHEN** the user opens General preferences without an outline-position preference
- **THEN** Outline position offers Left and Right with Left selected
- **AND** the outline remains to the left of the document workspace

#### Scenario: Changing the side takes effect immediately
- **WHEN** a sidebar is visible and the user selects Right
- **THEN** Files, the document workspace, and Outline appear in that order on the next render in Edit, Visual Edit, Split Preview, and Read modes
- **AND** actual file rows and outline headings are visible simultaneously
- **AND** document tabs remain above the document workspace
- **AND** outline navigation, folding, and scrolling remain available

#### Scenario: Files stays on the left
- **WHEN** Outline position is Right and the user navigates files, headings, or document tabs
- **THEN** Files stays in its left pane and Outline stays in its separate right pane
- **AND** neither pane contains a Files/Outline tab switch

#### Scenario: Hidden sidebar retains visibility state
- **WHEN** the user changes Outline position while all sidebars are hidden
- **THEN** both panes stay hidden
- **AND** showing Files or Outline in Right mode only reveals that pane

#### Scenario: Sidebar resizing follows its side
- **WHEN** the user drags the divider of a left-side sidebar to the right or a right-side outline to the left
- **THEN** only that pane becomes wider within the existing width limits
- **AND** both pane widths persist independently
- **AND** double-clicking either divider restores only its pane's default width

#### Scenario: Returning to Left restores shared tabs
- **WHEN** the user changes Outline position from Right to Left
- **THEN** the separate right pane disappears and the left Files/Outline tabs return
- **AND** if only the right outline was visible, the left sidebar opens on Outline

#### Scenario: Position changes preserve document state
- **WHEN** the user changes Outline position
- **THEN** document text, version, dirty state, selection, undo history, per-document folding, and version-cached derived Markdown state remain unchanged

### Requirement: Outline position SHALL persist safely

The editor SHALL persist Outline position as `outline_position = "left"` or `"right"` in `config.toml` and restore it on startup. Missing, unknown, or non-string values SHALL fall back to Left without losing other valid preferences. Reset Preferences SHALL restore Left without resetting either saved pane width.

#### Scenario: Both choices round-trip
- **WHEN** the user saves either outline-position choice and restarts the editor
- **THEN** that choice is restored and reflected by the preference control and visible outline

#### Scenario: Older or invalid preferences retain compatibility
- **WHEN** `outline_position` is absent, unknown, or has a non-string TOML value
- **THEN** Outline position defaults to Left
- **AND** other valid preferences still load

#### Scenario: Reset restores Left
- **WHEN** the user confirms Reset Preferences after selecting Right
- **THEN** Outline position returns to Left and is saved
- **AND** both saved pane widths are preserved

### Requirement: Detached Outline SHALL have independent persisted visibility

When Outline position is Right, Outline and File Tree actions SHALL toggle only their respective pane. The editor SHALL persist detached-outline visibility as `outline_visible` in `config.toml`; a missing or non-boolean value SHALL inherit the saved sidebar visibility. Right-outline width SHALL persist separately as `outline_width` in session layout with the same defaults and bounds as the existing sidebar width.

#### Scenario: Outline toggle leaves Files visible
- **WHEN** both panes are visible in Right mode and the user hides Outline
- **THEN** the right pane closes while Files remains visible on the left
- **AND** showing Outline again restores the right pane without replacing Files

#### Scenario: File Tree toggle leaves Outline visible
- **WHEN** both panes are visible in Right mode and the user hides File Tree
- **THEN** the left pane closes while Outline remains visible on the right

#### Scenario: Visibility and widths restore independently
- **WHEN** the user restarts after hiding one pane and resizing the other
- **THEN** the Right layout restores each pane's saved visibility and its own saved width

#### Scenario: Existing right-position preferences upgrade
- **WHEN** a configuration has Right position but no `outline_visible` key
- **THEN** both pane visibility values inherit the previous sidebar visibility
- **AND** a missing or invalid outline width uses the existing sidebar-width default and bounds

### Requirement: Preferences SHALL expose accessible AI setup
The Preferences panel SHALL provide an AI tab even while AI is disabled. It SHALL expose basic enablement/provider/key/model/test controls and collapsed advanced settings using editable controls, visible validation/status, active-theme colors, and the current language. Content and the tab strip SHALL remain reachable when viewport width/height or font scaling prevents fitting all controls. Explicit AI setup entry points SHALL land on AI without changing the default File → Preferences landing tab. Invalid draft values SHALL not replace a working stored profile until committed validly.

#### Scenario: Disabled feature can be configured
- **WHEN** AI is disabled and the user opens Preferences → AI
- **THEN** setup controls remain usable and no AI request is sent merely by opening the tab

#### Scenario: Invalid endpoint draft
- **WHEN** the user types an invalid advanced endpoint
- **THEN** inline feedback appears and the last valid profile remains intact

### Requirement: Preferences tab strip SHALL stay on one row
The Preferences panel tab strip SHALL lay out every preferences tab on a single non-wrapping row, in the existing tab order. Tabs SHALL size to their labels instead of using a minimum width that forces a second row. When the panel is narrower than the labels, the strip SHALL scroll horizontally and every tab SHALL remain reachable. The strip SHALL NOT wrap a tab onto a second row. Opening File → Preferences SHALL still land on General.

#### Scenario: Six tabs stay on one row
- **WHEN** the Preferences panel is open at its normal width in Simplified Chinese
- **THEN** General, Appearance, Images, Shortcuts, Export, and AI Agent appear on one row
- **AND** no tab is placed on a second row

#### Scenario: Narrow panel scrolls the strip
- **WHEN** the preferences panel is narrower than the combined tab labels
- **THEN** the tab strip scrolls horizontally
- **AND** the user can still activate every tab, including AI Agent

#### Scenario: Opening Preferences lands on General
- **WHEN** the user opens the Preferences panel from File → Preferences
- **THEN** the General tab is active

### Requirement: Preferences SHALL expose a default-on slash-command popup toggle

Preferences General SHALL expose a localized actionable boolean control for showing quick commands when typing `/`, reflecting the current value and using the active theme. The setting SHALL default to enabled, apply immediately, and persist as `slash_commands_enabled` in `config.toml`. Missing or non-boolean values SHALL fall back to enabled while retaining other valid settings. Reset Preferences SHALL restore enabled behavior. Changing the setting SHALL NOT modify document text, version, selection, dirty state, undo/redo history, or shared derived Markdown caches.

#### Scenario: Disable and reload
- **WHEN** the user disables the slash-command popup in General preferences and restarts the app
- **THEN** the control remains disabled and automatic slash completion remains off

#### Scenario: Older or invalid configuration
- **WHEN** `slash_commands_enabled` is absent or contains a non-boolean TOML value
- **THEN** the popup setting defaults to enabled and other valid preferences still load

#### Scenario: Reset restores enabled
- **WHEN** the user confirms Reset Preferences after disabling the popup
- **THEN** the setting returns to enabled and that value is saved

#### Scenario: Toggle preserves the document
- **WHEN** the user changes the popup setting in either direction
- **THEN** document text, version, selection, dirty state, undo/redo history, and shared derived-cache identities remain unchanged

### Requirement: Preferences SHALL control the automatic block-grip popup

General preferences SHALL expose a localized **Show block menu on grip click** toggle using the active theme. The toggle SHALL default to enabled, apply immediately and persist across application restarts. Disabling it SHALL close an open grip-invoked popup while retaining its selected block range. The toggle SHALL control only automatic grip-click opening; block selection, Shift extension, dragging, explicit right-click and keyboard context menus SHALL remain available. Changing this preference SHALL NOT change document source, version, dirty state, undo history or shared derived-cache identity.

#### Scenario: Enabled preference opens the popup
- **WHEN** the preference is enabled and the user completes an ordinary click on an eligible Visual Edit block grip
- **THEN** the clicked block is selected and its compact quick-settings popup opens

#### Scenario: Disabled preference retains selection-only clicks
- **WHEN** the preference is disabled and the user clicks a block grip
- **THEN** the block is selected without opening a popup
- **AND** Shift-click extension, dragging, right-click and keyboard context menus retain their documented behavior

#### Scenario: Toggling off dismisses only the automatic popup
- **WHEN** the user disables the preference while a grip popup is open
- **THEN** that popup closes immediately and its established block selection remains
- **AND** document source, version, dirty state, history and shared derived caches remain unchanged

### Requirement: Block-grip popup preference SHALL persist safely

The editor SHALL persist the automatic block-grip popup preference as the boolean `block_handle_menu` in `config.toml`. Missing or non-boolean values SHALL fall back to enabled without preventing startup or losing unrelated valid preferences. Reset Preferences SHALL restore enabled.

#### Scenario: Old configuration enables the popup
- **WHEN** an existing configuration omits `block_handle_menu`
- **THEN** automatic grip-click popups are enabled

#### Scenario: Invalid configuration falls back safely
- **WHEN** `block_handle_menu` contains a string, number or other non-boolean value
- **THEN** automatic grip-click popups default to enabled and unrelated valid preferences are honored

#### Scenario: Disabled value round-trips
- **WHEN** the user disables automatic grip-click popups and restarts the editor
- **THEN** the preference and its General-tab control remain disabled

#### Scenario: Reset restores enabled
- **WHEN** the user resets preferences after disabling automatic grip-click popups
- **THEN** automatic grip-click popups are enabled again

### Requirement: Preferences SHALL control automatic update downloads

General preferences SHALL expose a localized Updates section with an actionable Automatically download updates boolean, defaulting to enabled and using active-theme colors. It SHALL explain the five-hour check cadence and explicit restart confirmation, and SHALL describe browser-download fallback when the current installation cannot self-update. The value SHALL apply to subsequent downloads immediately and persist as `automatically_download_updates` in `config.toml`. Missing or non-boolean values SHALL default to true without losing unrelated valid preferences. Reset Preferences SHALL restore true. Disabling the preference SHALL not cancel an already-started download or discard a ready update, and SHALL not disable version checks or manual downloads. Changing it SHALL preserve documents and derived caches.

#### Scenario: Default and backward compatibility
- **WHEN** preferences are new or the automatic-download key is absent or invalid
- **THEN** automatic downloads default to enabled and other valid preferences load

#### Scenario: Disable and restart
- **WHEN** the user disables automatic update downloads and restarts Markion
- **THEN** the control remains disabled and later release discovery waits for download consent

#### Scenario: Reset and immediate application
- **WHEN** the user resets preferences after disabling automatic update downloads
- **THEN** automatic downloads become enabled and are persisted
- **AND** no document text, version, selection, dirty state, history or derived cache is changed
