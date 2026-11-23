## ADDED Requirements

### Requirement: Collapsible Visual Edit source editors follow the active theme
The collapsible HTML, display math, YAML front matter, and diagram source editors SHALL use the active theme for the source editor background, border, and toggle normal and hover colors.

#### Scenario: Switching themes with a source editor open
- **WHEN** the user switches themes while a collapsible source editor is visible in Visual Edit
- **THEN** the source editor and toggle use the new theme's colors
- **AND** the current expansion state and authored text are preserved
