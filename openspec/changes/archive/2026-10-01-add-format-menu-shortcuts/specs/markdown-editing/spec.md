## ADDED Requirements

### Requirement: Structural Format actions have default keyboard shortcuts
The editor SHALL provide platform-appropriate default keyboard shortcuts for the existing Ordered List, Unordered List, Task List, Blockquote, and Code Fence actions. On Windows and Linux the defaults SHALL be `Ctrl+Shift+[`, `Ctrl+Shift+]`, `Ctrl+Shift+X`, `Ctrl+Shift+Q`, and `Ctrl+Shift+K`, respectively; on macOS the same keys SHALL use `Cmd` in place of `Ctrl`. Invoking an action's effective shortcut SHALL perform the same Markdown transformation as invoking that action from the Format menu. Each action SHALL participate in the existing customizable-shortcut behavior and SHALL expose its effective binding both beside its Format-menu label and in the localized Editing section of the in-app shortcut reference.

#### Scenario: Windows and Linux defaults match the reference mapping
- **WHEN** the editor runs on Windows or Linux with no overrides for the five structural Format actions
- **THEN** Ordered List uses `Ctrl+Shift+[`, Unordered List uses `Ctrl+Shift+]`, Task List uses `Ctrl+Shift+X`, Blockquote uses `Ctrl+Shift+Q`, and Code Fence uses `Ctrl+Shift+K`

#### Scenario: macOS defaults use the platform modifier
- **WHEN** the editor runs on macOS with no overrides for the five structural Format actions
- **THEN** the same actions use `Cmd+Shift+[`, `Cmd+Shift+]`, `Cmd+Shift+X`, `Cmd+Shift+Q`, and `Cmd+Shift+K`, respectively

#### Scenario: Shortcut dispatch matches the Format menu action
- **WHEN** the user invokes the effective shortcut for one of the five actions while a document can be formatted
- **THEN** the editor applies the same ordered-list, unordered-list, task-list, blockquote, or fenced-code transformation that the matching Format-menu item applies

#### Scenario: New shortcuts are discoverable and customizable
- **WHEN** the user opens the Format menu or the Editing section of the in-app shortcut reference
- **THEN** all five actions display their effective platform-specific bindings
- **AND** assigning or resetting an override updates dispatch and both displayed locations through the existing shortcut-customization behavior

