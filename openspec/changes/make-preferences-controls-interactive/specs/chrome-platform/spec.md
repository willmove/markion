## ADDED Requirements

### Requirement: In-window menus SHALL follow the active theme
The in-window menu bar and dropdown menus SHALL derive their backgrounds, text colors, borders, separators, and active states from the active theme palette so both light and dark themes remain readable and visually consistent with the editor chrome.

#### Scenario: Menu bar adapts to a dark theme
- **WHEN** the active theme is a dark theme such as One Dark or GitHub Dark
- **THEN** the in-window menu bar and dropdown menus render with dark-compatible backgrounds and readable text

#### Scenario: Menu bar adapts to a light theme
- **WHEN** the active theme is a light theme such as Paper or GitHub Light
- **THEN** the in-window menu bar and dropdown menus render with light-compatible backgrounds and readable text

#### Scenario: Changing theme updates menus
- **WHEN** the user selects a different theme from Preferences
- **THEN** the in-window menu bar and any subsequently opened dropdown use the newly active theme palette
