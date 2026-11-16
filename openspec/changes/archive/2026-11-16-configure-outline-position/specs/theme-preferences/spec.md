# Spec Delta

## ADDED Requirements

### Requirement: Preferences panel SHALL expose Outline position

General preferences SHALL provide a localized Outline position control with mutually exclusive Left and Right choices, using active-theme colors. Selecting a choice SHALL immediately place the visible Outline sidebar on that side of the document workspace. Files SHALL remain on the left, using the existing shared sidebar and Files/Outline tabs.

#### Scenario: Default placement and choices

- **WHEN** the user opens General preferences without an outline-position preference
- **THEN** Outline position offers Left and Right with Left selected
- **AND** the outline remains to the left of the document workspace

#### Scenario: Changing the side takes effect immediately

- **WHEN** the outline is visible and the user selects Right or Left
- **THEN** the sidebar and its inner-edge divider move to the selected side on the next render in Edit, Visual Edit, Split Preview, and Read modes
- **AND** document tabs remain above the document workspace
- **AND** outline navigation, folding, and scrolling remain available

#### Scenario: Files stays on the left

- **WHEN** Outline position is Right and the user switches to Files
- **THEN** the shared sidebar displays Files on the left
- **AND** switching back to Outline restores its right-side placement

#### Scenario: Hidden sidebar retains visibility state

- **WHEN** the user changes Outline position while the sidebar is hidden
- **THEN** the sidebar stays hidden
- **AND** showing Outline later places it on the selected side

#### Scenario: Sidebar resizing follows its side

- **WHEN** the user drags the divider of a left-side sidebar to the right or a right-side outline to the left
- **THEN** the sidebar becomes wider within the existing width limits
- **AND** the shared width remains persistent and double-clicking the divider restores the default width

#### Scenario: Position changes preserve document state

- **WHEN** the user changes Outline position
- **THEN** document text, version, dirty state, selection, undo history, per-document folding, and version-cached derived Markdown state remain unchanged

### Requirement: Outline position SHALL persist safely

The editor SHALL persist Outline position as `outline_position = "left"` or `"right"` in `config.toml` and restore it on startup. Missing, unknown, or non-string values SHALL fall back to Left without losing other valid preferences. Reset Preferences SHALL restore Left without resetting the saved sidebar width.

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
- **AND** saved sidebar width is preserved
