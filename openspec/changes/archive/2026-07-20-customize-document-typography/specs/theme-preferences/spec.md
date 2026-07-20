## ADDED Requirements

### Requirement: Preferences panel SHALL expose document typography controls
The Preferences panel SHALL expose localized numeric controls for Source font size, Reading font size, and Paragraph spacing. Each control SHALL display its current logical-pixel value, provide decrement and increment actions in 1px steps, disable actions at the supported bound, use active-theme colors, apply a changed value immediately, and persist it through the existing preferences save path.

#### Scenario: Typography controls show current values
- **WHEN** the Preferences panel is open
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

### Requirement: Document typography preferences SHALL persist safely
The editor SHALL persist source font size as `editor_font_size`, rendered font size as `rendered_font_size`, and rendered paragraph spacing as `paragraph_spacing` in `config.toml`. Defaults SHALL be 15px, 14px, and 12px respectively. Font sizes SHALL normalize to 10–32px inclusive and paragraph spacing SHALL normalize to 0–32px inclusive. Missing or non-numeric fields SHALL use their defaults, numeric out-of-range fields SHALL clamp to the nearest bound, and reset SHALL restore all three defaults.

#### Scenario: Typography values round-trip
- **WHEN** preferences containing `editor_font_size = 18`, `rendered_font_size = 20`, and `paragraph_spacing = 16` are saved and reloaded
- **THEN** all three values are restored exactly and reflected by the Preferences controls

#### Scenario: Older config uses current defaults
- **WHEN** an existing `config.toml` omits all typography fields
- **THEN** the editor starts with 15px source text, 14px rendered body text, and 12px rendered paragraph spacing

#### Scenario: Invalid and out-of-range values are safe
- **WHEN** typography fields are non-numeric or outside their supported ranges
- **THEN** non-numeric values use defaults and numeric values clamp to their nearest supported bound
- **AND** the preferences file does not prevent the editor from starting

#### Scenario: Reset restores typography defaults
- **WHEN** the user resets preferences after changing typography
- **THEN** Source font size returns to 15px, Reading font size returns to 14px, and Paragraph spacing returns to 12px
- **AND** visible document surfaces reflow to those defaults

#### Scenario: Preferences summary includes typography
- **WHEN** the user opens the preferences summary
- **THEN** it reports the current source font size, rendered font size, and paragraph spacing using localized labels and pixel values
