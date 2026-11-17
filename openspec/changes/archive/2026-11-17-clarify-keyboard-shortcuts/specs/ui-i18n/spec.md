## ADDED Requirements

### Requirement: Keyboard shortcut reference presents platform-specific tables
The system SHALL present the localized keyboard shortcut reference in a prompt-safe table form that separates Windows/Linux shortcuts from macOS shortcuts. The reference SHALL use user-facing key names such as Ctrl, Cmd, Alt, Option, and Shift instead of GPUI-internal `Secondary-*` names, and SHALL NOT rely on Markdown table syntax that the native prompt displays as raw source text.

#### Scenario: Shortcut reference separates platform keys
- **WHEN** the user opens Help -> Keyboard Shortcuts
- **THEN** the shortcut reference lists each documented action with distinct Windows/Linux and macOS shortcut columns
- **AND** the table is visible as plain dialog text rather than Markdown source such as `|---|---|---|`

#### Scenario: Shortcut reference remains localized
- **WHEN** the active interface language changes
- **THEN** the shortcut reference keeps localized section/action labels while preserving explicit platform key names
