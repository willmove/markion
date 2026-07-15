## ADDED Requirements

### Requirement: Keyboard shortcut help uses a theme-aware navigable panel
The editor SHALL open Help -> Keyboard Shortcuts and the F1 shortcut as an in-app modal panel rather than a native plain-text prompt. The panel SHALL provide Windows/Linux and macOS platform tabs, category navigation for all documented shortcut groups, a scrollable action list with visually distinct shortcut labels, active-theme styling, and explicit dismissal.

#### Scenario: Help action opens the shortcut panel
- **WHEN** the user activates Help -> Keyboard Shortcuts or presses F1
- **THEN** a modal shortcut panel opens above the editor workspace
- **AND** the native plain-text shortcut prompt is not shown

#### Scenario: Panel defaults to the running platform
- **WHEN** the shortcut panel opens on macOS
- **THEN** the macOS platform tab is selected and the first shortcut category is active
- **AND** when the panel opens on Windows or Linux, the Windows/Linux tab is selected instead

#### Scenario: Platform tab changes displayed shortcuts
- **WHEN** the user selects the other platform tab
- **THEN** the visible action list keeps the selected category and changes each shortcut to that platform's combination

#### Scenario: Category navigation changes the action list
- **WHEN** the user selects File, Tabs, Editing, View, Search, Tables, or Export in the category sidebar
- **THEN** the action list displays the documented shortcuts for that category and the selected platform

#### Scenario: Long shortcut content remains usable
- **WHEN** the selected category contains more actions than fit in the panel body or localized labels require additional space
- **THEN** the action list remains legible and scrollable without expanding beyond the application window

#### Scenario: Panel follows the active theme
- **WHEN** the shortcut panel opens under a light or dark theme
- **THEN** its scrim, surface, text, borders, active tabs, hover states, category selection, and shortcut labels use the active theme palette

#### Scenario: Panel can be dismissed
- **WHEN** the user activates the close control or presses Escape while the shortcut panel is open
- **THEN** the panel closes without changing document content, selection, undo history, or view mode

### Requirement: Sidebar toggle shortcut is ergonomic and documented
The editor SHALL bind the sidebar visibility toggle to Ctrl+Shift+B on Windows/Linux and Cmd+Shift+B on macOS. The keyboard shortcut panel SHALL document that shortcut consistently in both platform views.

#### Scenario: Sidebar toggle uses platform shortcut
- **WHEN** the user presses Ctrl+Shift+B on Windows/Linux or Cmd+Shift+B on macOS
- **THEN** the sidebar visibility toggles

#### Scenario: Sidebar shortcut avoids bold conflict
- **WHEN** the user presses Ctrl+B on Windows/Linux or Cmd+B on macOS
- **THEN** the editor applies bold formatting instead of toggling the sidebar
