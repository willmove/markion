## ADDED Requirements

### Requirement: Keyboard shortcut reference presents platform-specific tables
The system SHALL present the localized keyboard shortcut reference in a table-oriented form that separates Windows/Linux shortcuts from macOS shortcuts. The reference SHALL use user-facing key names such as Ctrl, Cmd, Alt, Option, and Shift instead of GPUI-internal `Secondary-*` names.

#### Scenario: Shortcut reference separates platform keys
- **WHEN** the user opens Help -> Keyboard Shortcuts
- **THEN** the shortcut reference lists each documented action with distinct Windows/Linux and macOS shortcut columns

#### Scenario: Shortcut reference remains localized
- **WHEN** the active interface language changes
- **THEN** the shortcut reference keeps localized section/action labels while preserving explicit platform key names
