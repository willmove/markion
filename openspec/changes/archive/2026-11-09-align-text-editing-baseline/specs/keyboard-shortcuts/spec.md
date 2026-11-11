## REMOVED Requirements

### Requirement: Redo uses one keyboard shortcut
The previous single-binding Redo requirement is removed; both platform-conventional aliases are now supported.

## ADDED Requirements

### Requirement: Fixed baseline navigation bindings
The editor SHALL install fixed baseline bindings for word movement/deletion, line-boundary selection, and page navigation. On macOS, word movement/deletion SHALL use Option+Arrow/Backspace; on Windows and Linux it SHALL use Ctrl+Arrow/Backspace. Shift+Home/End and PageUp/PageDown SHALL use their conventional forms. These fixed baseline actions SHALL coexist with, but not require entries in, the customizable menu-action override registry.

#### Scenario: Platform word bindings
- **WHEN** the user invokes the platform word-navigation or word-deletion binding
- **THEN** the corresponding Unicode-aware baseline action runs
- **AND** applying or clearing a menu-action shortcut override leaves the fixed baseline bindings installed

### Requirement: Redo supports both platform-conventional aliases
The Redo action SHALL be installed through both `secondary-y` and `secondary-shift-z`: Ctrl+Y and Ctrl+Shift+Z on Windows/Linux, and Cmd+Y and Cmd+Shift+Z on macOS. Both SHALL dispatch the same history operation, and shortcut documentation SHALL list both. This supersedes the earlier requirement that prohibited the Shift+Z alias.

#### Scenario: Either Redo alias works
- **WHEN** the user presses either supported Redo combination
- **THEN** exactly one redo operation occurs for that key press
- **AND** menu-action shortcut rebinding does not remove the other fixed editing bindings
