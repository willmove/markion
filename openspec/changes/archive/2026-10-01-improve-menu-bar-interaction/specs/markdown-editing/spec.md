## ADDED Requirements

### Requirement: Redo uses one keyboard shortcut
The editor SHALL bind the Redo action exactly once using the platform-mapped `secondary-y` combination: Ctrl+Y on Windows/Linux and Cmd+Y on macOS. The editor MUST NOT bind Ctrl/Cmd+Shift+Z to Redo. The in-window Edit menu and the localized keyboard shortcut reference SHALL display only the active Redo combination.

#### Scenario: Redo uses Ctrl+Y on Windows and Linux
- **WHEN** the editor runs on Windows or Linux and the user presses Ctrl+Y
- **THEN** the editor invokes the existing Redo action
- **AND** Ctrl+Shift+Z does not invoke Redo

#### Scenario: Redo uses the mapped key on macOS
- **WHEN** the editor runs on macOS and the user presses Cmd+Y
- **THEN** the editor invokes the existing Redo action
- **AND** Cmd+Shift+Z does not invoke Redo

#### Scenario: Redo shortcut surfaces show one combination
- **WHEN** the user views Edit -> Redo or opens the keyboard shortcut reference
- **THEN** Redo is documented with Ctrl+Y on Windows/Linux or Cmd+Y on macOS
- **AND** no second Redo shortcut is shown

#### Scenario: Redo behavior is otherwise unchanged
- **WHEN** the user invokes Redo through Ctrl/Cmd+Y or the Edit menu
- **THEN** the existing redo history operation and status feedback are used
