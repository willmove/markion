## ADDED Requirements

### Requirement: Code block formatting SHALL use the customizable shortcut registry
The editor SHALL register the `format-code-block` action in the customizable shortcut registry with default binding `secondary-alt-l` (`Ctrl+Alt+L` on Windows/Linux, `Cmd+Option+L` on macOS). Its effective binding SHALL appear in menus and the shortcut reference and participate in conflict detection, persistence under `[shortcuts]`, per-action/global reset, and live rebinding without requiring an application restart.

#### Scenario: Default and overridden shortcut dispatches code formatting
- **WHEN** the user presses the effective `format-code-block` keystroke inside an editable fenced code block
- **THEN** the editor invokes `FormatCodeBlock` on that block
- **AND** reassigning or resetting the binding in Preferences -> Shortcuts updates active key dispatch and menu labels immediately
