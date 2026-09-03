## ADDED Requirements

### Requirement: Preferences panel SHALL expose a Code blocks section
The Preferences panel Appearance tab SHALL include a **Code blocks** section below the Typography section, gathering every code-display control in one place: a code highlight theme control offering **Light** and **Dark** as mutually exclusive choices (default Dark); the code line-numbers toggle, relocated from the General tab's display settings; a wrap-long-lines toggle; a code font-size numeric control with decrement/increment actions in 1px steps, bound-disabled behavior, and a follow-reading-size action that clears an explicit stored size; and the code font-family control, relocated from the Typography section. Activating any control SHALL apply the change immediately to the affected code-block surfaces and persist it through the existing preferences file. Controls SHALL use localized labels and active-theme colors.

#### Scenario: Code blocks section appears on the Appearance tab
- **WHEN** the Preferences panel is open on the Appearance tab
- **THEN** a Code blocks section appears below the Typography section containing the code highlight theme choice, the code line-numbers toggle, the wrap-long-lines toggle, the code font-size stepper, and the code font-family control
- **AND** the General tab no longer shows a code line-numbers control and the Typography section no longer shows a code font-family control

#### Scenario: Choosing the code highlight theme applies immediately
- **WHEN** the user selects Light or Dark in the code highlight theme control
- **THEN** fenced code blocks in Read mode, Split Preview's rendered pane, and the Visual Edit direct code editor repaint with that theme's chrome and token colors on the next render
- **AND** the previous choice is indicated as inactive and the selection persists

#### Scenario: Line-numbers toggle relocates without behavior change
- **WHEN** the user toggles Show line numbers in the Code blocks section
- **THEN** fenced code blocks in Read mode and Split Preview's rendered pane show or hide the numbered gutter exactly as the General-tab toggle did before the relocation
- **AND** the choice persists through the existing `code_line_numbers` preference

#### Scenario: Code font size stepper applies and clears
- **WHEN** the user increments or decrements the code font size within its supported range
- **THEN** all rendered code-block surfaces reflow at the new size immediately and the value persists
- **WHEN** the user activates the follow-reading-size action while an explicit size is stored
- **THEN** the stored size is cleared, code blocks return to deriving their size from the reading font size, and the control reflects the derived value

### Requirement: Code display preferences SHALL persist safely
The editor SHALL persist the code highlight theme in `config.toml` as `code_theme` with allowed values `light` or `dark` (default `dark`; any other value degrades to `dark`), the wrap preference as a boolean `code_long_line_wrap` (default on; missing or non-boolean values degrade to on), and the code font size as an optional integer `code_font_size` where an absent key means "follow the reading font size" and a present value normalizes to 10–32px inclusive. The existing `code_line_numbers` key and default are unchanged. Resetting preferences SHALL restore Dark, wrap on, line numbers on, and SHALL clear `code_font_size` to the follow-reading state.

#### Scenario: Code display values round-trip
- **WHEN** preferences containing `code_theme = "light"`, `code_long_line_wrap = false`, and `code_font_size = 16` are saved and reloaded
- **THEN** all three values are restored exactly and reflected by the Code blocks section controls

#### Scenario: Older config uses current defaults
- **WHEN** an existing `config.toml` omits all three new keys
- **THEN** the editor starts with the Dark code theme, long-line wrapping on, and code font size following the reading font size, identical to the pre-change appearance

#### Scenario: Invalid values are safe
- **WHEN** `code_theme` holds an unrecognized string, `code_long_line_wrap` holds a non-boolean, or `code_font_size` is outside 10–32
- **THEN** the theme degrades to Dark, the wrap preference degrades to on, and the size clamps to its nearest bound
- **AND** the preferences file does not prevent the editor from starting

#### Scenario: Reset restores code display defaults
- **WHEN** the user resets preferences after changing code display settings
- **THEN** the code highlight theme returns to Dark, wrapping returns to on, line numbers return to on, and the code font size clears to the follow-reading state
- **AND** visible code blocks repaint to those defaults

## MODIFIED Requirements

### Requirement: Preferences panel SHALL expose supported display settings as controls
The Preferences panel SHALL expose focus mode, typewriter mode, Preview adaptive width, sidebar visibility, and sidebar tab as interactive controls when those preferences are already supported by the app state and preferences file. Activating a control SHALL apply the setting immediately and persist it through the existing preferences file. The code line-numbers setting SHALL no longer be surfaced here; its control SHALL instead live in the Appearance tab's Code blocks section (see the Code blocks section requirement).

#### Scenario: Boolean settings are editable in the panel
- **WHEN** the Preferences panel is open
- **THEN** focus mode, typewriter mode, Preview adaptive width, and sidebar visibility each render as an actionable control showing the current state

#### Scenario: Toggling a setting applies immediately
- **WHEN** the user activates a boolean Preferences control
- **THEN** the corresponding app state changes immediately
- **AND** the new value is persisted to the preferences file

#### Scenario: Sidebar tab is editable in the panel
- **WHEN** the Preferences panel is open
- **THEN** the sidebar tab preference renders as a mutually exclusive Files/Outline choice that indicates the current tab

#### Scenario: Selecting a sidebar tab applies immediately
- **WHEN** the user selects a different sidebar tab in the Preferences panel
- **THEN** the app switches the sidebar to that tab, keeps the sidebar visible, and persists the new tab

### Requirement: Preferences panel SHALL expose document font family controls
The Preferences panel SHALL include one control per font slot with localized labels consistent with the font-size controls: the **source** and **rendered** slots in the typography section, and the **code** slot in the Appearance tab's Code blocks section. Each control SHALL present a follow-theme state and an explicit-family state: in the follow-theme state it SHALL indicate that the theme (or default) font applies; activating the control SHALL present a selection list populated from the fonts installed on the machine, each entry rendered in its own family as live preview, plus a follow-theme entry that clears the stored preference. Selecting an entry SHALL apply it immediately to that document plane and persist it. The control SHALL show an advisory warning when the currently stored family (for example hand-edited into `config.toml`) is not among the installed fonts.

#### Scenario: Controls reflect the current slot state
- **WHEN** the Preferences panel is open
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
