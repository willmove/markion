## ADDED Requirements

### Requirement: Effective bindings resolve from defaults plus overrides
The editor SHALL maintain a registry of customizable menu-action shortcuts, each with a stable kebab-case action id, a default GPUI keystroke binding, and curated default display labels for Windows/Linux and macOS. The effective binding of an action SHALL be its stored override when present and valid, otherwise the default. The keymap, in-window menu shortcut labels, and the shortcut reference list SHALL all render and dispatch from the same effective binding.

#### Scenario: Defaults apply with no overrides
- **WHEN** no override exists for an action
- **THEN** the action is bound to its default keystroke and menus and the reference list show the curated default label

#### Scenario: Override applies everywhere
- **WHEN** an action has a valid override
- **THEN** pressing the override keystroke dispatches the action, the default keystroke no longer dispatches it, and both the menu label and the reference list show the override binding

### Requirement: Shortcut editing UI in the Preferences panel
The Preferences panel SHALL provide a Shortcuts tab listing every customizable action in the categorized reference layout (platform tabs and category sidebar). Each row SHALL allow capture-based reassignment: activating the row's binding prompts for a key press, and the next key combination becomes the candidate binding. Escape SHALL cancel capture. A row with an active override SHALL offer a per-action reset that restores the default binding.

#### Scenario: Capture assigns a new binding
- **WHEN** the user activates a binding and presses a valid new key combination
- **THEN** the override is applied, persisted, and visible in the row and menus

#### Scenario: Bare printable key is rejected
- **WHEN** the captured keystroke is a printable key without modifiers and not a function key (F1-F12)
- **THEN** the assignment is rejected with localized inline feedback and the previous binding remains

#### Scenario: Conflicting binding is rejected
- **WHEN** the captured keystroke equals another action's effective binding or a reserved fixed binding
- **THEN** the assignment is rejected with localized inline feedback naming the conflicting target, and both actions keep their prior bindings

#### Scenario: Escape cancels capture
- **WHEN** the user presses Escape while a row is capturing
- **THEN** capture mode ends with no change to any binding

#### Scenario: Per-action reset restores the default
- **WHEN** the user resets an action that has an override
- **THEN** the override is removed, the default binding applies again, and the change persists

### Requirement: Shortcut override persistence
The editor SHALL persist shortcut overrides in `config.toml` as a `[shortcuts]` table mapping action id to GPUI keystroke string, omitting the table when no overrides exist. On load, entries with unknown action ids or keystroke strings that fail parsing SHALL be dropped with a diagnostic log line, and the affected actions SHALL use their defaults. The global preferences reset SHALL clear all shortcut overrides.

#### Scenario: Overrides round-trip
- **WHEN** the user sets an override and restarts the editor
- **THEN** the override is loaded from `config.toml` and the action uses the overridden binding

#### Scenario: Invalid entries fall back to defaults
- **WHEN** `config.toml` contains an unknown action id or an unparseable keystroke in `[shortcuts]`
- **THEN** that entry is ignored with a log line, the action uses its default binding, and the editor starts normally

#### Scenario: Missing table means defaults
- **WHEN** `config.toml` has no `[shortcuts]` table
- **THEN** every action uses its default binding

#### Scenario: Preferences reset clears overrides
- **WHEN** the user resets preferences
- **THEN** all shortcut overrides are removed and every action returns to its default binding

### Requirement: Rebinding applies live without restart
Changing, resetting, or clearing shortcut overrides SHALL update the application's active keymap immediately via a full rebind, without requiring a restart. The rebind SHALL restore the complete binding set, so fixed core-editing keys and file-tree keys keep working after any shortcut change.

#### Scenario: New binding works immediately
- **WHEN** the user assigns a new binding to an action
- **THEN** the new keystroke dispatches the action in the same session without restarting

#### Scenario: Fixed keys survive rebinding
- **WHEN** any shortcut override is applied or cleared
- **THEN** core editing keys (arrows, backspace, enter, tab) and file-tree keys continue to work as before
