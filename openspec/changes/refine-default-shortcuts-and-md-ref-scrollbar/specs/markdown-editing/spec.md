## MODIFIED Requirements

### Requirement: View mode switching shortcuts
The editor SHALL provide keyboard shortcuts for switching to each view mode directly, using platform-appropriate modifier conventions. The editor MAY also retain an existing shortcut that cycles through the view modes. With no stored override, the factory defaults SHALL be: Edit / source mode `Ctrl+/` on Windows and Linux and `Cmd+/` on macOS; Visual Edit mode `Ctrl+E` on Windows and Linux and `Cmd+E` on macOS; Split Preview mode `Ctrl+P` on Windows and Linux and `Cmd+P` on macOS; Read mode `Ctrl+R` on Windows and Linux and `Cmd+R` on macOS. Cycle-mode SHALL keep its existing factory default (`Ctrl+Shift+V` / `Cmd+Shift+V`). These actions SHALL participate in the existing customizable-shortcut registry so an override replaces the factory default everywhere the effective binding is shown or dispatched.

#### Scenario: Direct shortcut enters Edit mode
- **WHEN** the user presses the Edit mode shortcut
- **THEN** the active view mode becomes Edit
- **AND** status feedback identifies Edit mode

#### Scenario: Direct shortcut enters Visual Edit mode
- **WHEN** the user presses the Visual Edit mode shortcut
- **THEN** the active view mode becomes Visual Edit
- **AND** status feedback identifies Visual Edit mode

#### Scenario: Direct shortcut enters Split Preview mode
- **WHEN** the user presses the Split Preview mode shortcut
- **THEN** the active view mode becomes Split Preview
- **AND** status feedback identifies Split Preview mode

#### Scenario: Direct shortcut enters Read mode
- **WHEN** the user presses the Read mode shortcut
- **THEN** the active view mode becomes Read
- **AND** status feedback identifies Read mode

#### Scenario: Mode shortcuts follow platform conventions
- **WHEN** the editor runs on macOS versus Windows/Linux
- **THEN** the view mode shortcuts use the same `secondary` modifier convention as other application shortcuts

#### Scenario: Factory defaults for source, Visual Edit, split, and Read
- **WHEN** the editor runs with no overrides for `set-edit-mode`, `set-visual-edit-mode`, `set-split-preview-mode`, or `set-read-mode`
- **THEN** Edit / source mode uses `Ctrl+/` on Windows and Linux and `Cmd+/` on macOS
- **AND** Visual Edit mode uses `Ctrl+E` on Windows and Linux and `Cmd+E` on macOS
- **AND** Split Preview mode uses `Ctrl+P` on Windows and Linux and `Cmd+P` on macOS
- **AND** Read mode uses `Ctrl+R` on Windows and Linux and `Cmd+R` on macOS
- **AND** cycle-mode keeps its previous factory default

#### Scenario: View-mode shortcuts remain customizable
- **WHEN** the user assigns or resets an override for Edit, Visual Edit, Split Preview, or Read mode
- **THEN** keymap dispatch, View-menu labels, and the in-app shortcut reference all use the same effective binding

## ADDED Requirements

### Requirement: Inline code factory shortcut uses Ctrl+Shift+backtick
With no stored override, the inline-code formatting action SHALL use the factory-default shortcut `Ctrl+Shift+\`` on Windows and Linux and `Cmd+Shift+\`` on macOS, registered as GPUI keystroke `` secondary-shift-` `` under stable id `inline-code`. An override for `inline-code` SHALL replace the factory binding everywhere the effective binding is shown or dispatched.

#### Scenario: Inline code factory default uses Ctrl+Shift+backtick
- **WHEN** the editor runs with no override for `inline-code`
- **THEN** the Format menu, keymap, and shortcut catalog show `Ctrl+Shift+\`` on Windows and Linux and `Cmd+Shift+\`` on macOS
- **AND** pressing that chord applies the inline-code formatting action

#### Scenario: Inline code shortcut remains customizable
- **WHEN** the user assigns or resets an override for `inline-code`
- **THEN** keymap dispatch, Format-menu labels, and the shortcut reference all use the same effective binding
