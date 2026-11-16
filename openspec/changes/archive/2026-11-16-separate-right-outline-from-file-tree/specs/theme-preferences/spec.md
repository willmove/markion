# Spec Delta

## MODIFIED Requirements

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

## ADDED Requirements

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
