## MODIFIED Requirements

### Requirement: Keyboard shortcut system
The editor SHALL bind common formatting, file, view, and navigation operations to keyboard shortcuts, with platform-appropriate modifier conventions, and SHALL surface the full shortcut list in-app. Every action in the in-window File, Edit, View, Format, Export, and Help menus that has one or more active keyboard bindings SHALL display the current platform's user-facing shortcut combination or combinations beside its localized action label. Menu actions without an active binding MUST NOT display a shortcut marker.

#### Scenario: Shortcuts follow platform conventions
- **WHEN** the editor runs on macOS vs Windows/Linux
- **THEN** shortcuts use the platform-appropriate modifier key convention (Cmd vs Ctrl)

#### Scenario: Full shortcut reference is available in-app
- **WHEN** the user opens the keyboard shortcut reference from the Help menu
- **THEN** the editor displays the complete, localized list of shortcuts

#### Scenario: Bound in-window menu items display platform shortcuts
- **WHEN** the user opens an in-window application menu on Windows/Linux or macOS
- **THEN** every menu item with an active binding shows the corresponding user-facing shortcut combination or combinations beside its localized action label
- **AND** the displayed modifiers follow the current platform conventions

#### Scenario: Unbound in-window menu items omit shortcut markers
- **WHEN** the user opens an in-window application menu containing an action with no active keyboard binding
- **THEN** that action shows its localized label without an empty, placeholder, or inaccurate shortcut marker

#### Scenario: Conditional heading shortcuts match visible menu entries
- **WHEN** the configured Heading menu depth changes between H1–H5 and H1–H6
- **THEN** every visible bound heading item shows its matching platform shortcut
- **AND** no shortcut label is rendered for a heading item that is not present in the Format menu
