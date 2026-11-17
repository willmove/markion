## ADDED Requirements

### Requirement: Sidebar toggle shortcut is ergonomic and documented
The editor SHALL bind the sidebar visibility toggle to Ctrl+Shift+B on Windows/Linux and Cmd+Shift+B on macOS. The Help -> Keyboard Shortcuts reference SHALL document that shortcut consistently with the active platform-specific shortcut table.

#### Scenario: Sidebar toggle uses platform shortcut
- **WHEN** the user presses Ctrl+Shift+B on Windows/Linux or Cmd+Shift+B on macOS
- **THEN** the sidebar visibility toggles

#### Scenario: Sidebar shortcut avoids bold conflict
- **WHEN** the user presses Ctrl+B on Windows/Linux or Cmd+B on macOS
- **THEN** the editor applies bold formatting instead of toggling the sidebar
