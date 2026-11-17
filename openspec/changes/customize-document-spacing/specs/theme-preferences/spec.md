## MODIFIED Requirements

### Requirement: Preferences panel SHALL expose document typography controls
The Preferences panel Appearance tab SHALL expose localized numeric controls for Source font size, Reading font size, Paragraph spacing, and Line spacing. Font-size and paragraph-spacing controls SHALL display logical pixels in 1px steps; Line spacing SHALL display a percentage relative to defaults in 5-percentage-point steps over 75–300%. Each control SHALL provide decrement and increment actions, disable actions at the supported bound, use active-theme colors, apply a changed value immediately, and persist it through the existing preferences save path. The General tab SHALL NOT host these numeric typography controls.

#### Scenario: Typography controls show current values
- **WHEN** the Preferences panel Appearance tab is open
- **THEN** Source font size, Reading font size, Paragraph spacing, and Line spacing each render with a localized label, current value with its pixel or percentage unit, and minus/plus affordances
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
- **THEN** Source font size, Reading font size, Paragraph spacing, and Line spacing controls are not rendered in that tab

### Requirement: Document typography preferences SHALL persist safely

The editor SHALL persist source font size as `editor_font_size`, rendered font size as `rendered_font_size`, and rendered paragraph spacing as `paragraph_spacing` in `config.toml`, with defaults of 14px, 14px, and 12px respectively. Font sizes SHALL normalize to 10–32px inclusive and paragraph spacing SHALL normalize to 0–128px inclusive. Line spacing SHALL persist as integer `line_spacing` with default 100 and bounds 75–300 inclusive. The editor SHALL additionally persist optional font-family preferences as string keys `editor_font_family`, `rendered_font_family`, and `code_font_family` in `config.toml`; each key SHALL be optional, an absent, empty, or whitespace-only value SHALL mean "follow the active theme, then the built-in default", and a present value SHALL be persisted and reloaded verbatim without validating it against the set of installed fonts. Missing or non-numeric size/spacing fields SHALL use their defaults, numeric out-of-range fields SHALL clamp to the nearest bound, and reset SHALL restore all four size/spacing defaults and clear all three font-family preferences to the follow-theme state.

#### Scenario: Typography values round-trip

- **WHEN** preferences containing `editor_font_size = 18`, `rendered_font_size = 20`, `paragraph_spacing = 96`, and `line_spacing = 150` are saved and reloaded
- **THEN** all four values are restored exactly and reflected by the Preferences controls

#### Scenario: Older config uses current defaults

- **WHEN** an existing `config.toml` omits all typography fields
- **THEN** the editor starts with 14px source text, 14px rendered body text, 12px rendered paragraph spacing, and 100% line spacing

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
- **THEN** Source font size returns to 14px, Reading font size returns to 14px, Paragraph spacing returns to 12px, and Line spacing returns to 100%
- **AND** all three font-family preferences clear to the follow-theme state
- **AND** visible document surfaces reflow to those defaults

#### Scenario: Preferences summary includes typography

- **WHEN** the user opens the preferences summary
- **THEN** it reports the current source font size, rendered font size, paragraph spacing, and line spacing using localized labels and the appropriate pixel/percentage values
