## MODIFIED Requirements

### Requirement: Document typography preferences SHALL persist safely
The editor SHALL persist source font size as `editor_font_size`, rendered font size as `rendered_font_size`, and rendered paragraph spacing as `paragraph_spacing` in `config.toml`. Defaults SHALL be 14px, 14px, and 12px respectively. Font sizes SHALL normalize to 10–32px inclusive and paragraph spacing SHALL normalize to 0–32px inclusive. Missing or non-numeric fields SHALL use their defaults, numeric out-of-range fields SHALL clamp to the nearest bound, and reset SHALL restore all three defaults.

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

#### Scenario: Reset restores typography defaults
- **WHEN** the user resets preferences after changing typography
- **THEN** Source font size returns to 14px, Reading font size returns to 14px, and Paragraph spacing returns to 12px
- **AND** visible document surfaces reflow to those defaults

#### Scenario: Preferences summary includes typography
- **WHEN** the user opens the preferences summary
- **THEN** it reports the current source font size, rendered font size, and paragraph spacing using localized labels and pixel values
