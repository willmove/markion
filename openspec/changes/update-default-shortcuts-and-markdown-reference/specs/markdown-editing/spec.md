## MODIFIED Requirements

### Requirement: View mode switching shortcuts
The editor SHALL provide keyboard shortcuts for switching to each view mode directly, using platform-appropriate modifier conventions. The editor MAY also retain an existing shortcut that cycles through the view modes. With no stored override, the factory defaults SHALL be: Edit / source mode `Ctrl+/` on Windows and Linux and `Cmd+/` on macOS; Split Preview mode `Ctrl+P` on Windows and Linux and `Cmd+P` on macOS; Read mode `Ctrl+Shift+R` on Windows and Linux and `Cmd+Shift+R` on macOS. Visual Edit mode SHALL keep its existing factory default (`Ctrl+Alt+4` / `Cmd+Option+4`). Cycle-mode SHALL keep its existing factory default (`Ctrl+Shift+V` / `Cmd+Shift+V`). These actions SHALL participate in the existing customizable-shortcut registry so an override replaces the factory default everywhere the effective binding is shown or dispatched.

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

#### Scenario: Factory defaults for source, split, and Read
- **WHEN** the editor runs with no overrides for `set-edit-mode`, `set-split-preview-mode`, or `set-read-mode`
- **THEN** Edit / source mode uses `Ctrl+/` on Windows and Linux and `Cmd+/` on macOS
- **AND** Split Preview mode uses `Ctrl+P` on Windows and Linux and `Cmd+P` on macOS
- **AND** Read mode uses `Ctrl+Shift+R` on Windows and Linux and `Cmd+Shift+R` on macOS
- **AND** Visual Edit mode and cycle-mode keep their previous factory defaults

#### Scenario: View-mode shortcuts remain customizable
- **WHEN** the user assigns or resets an override for Edit, Split Preview, or Read mode
- **THEN** keymap dispatch, View-menu labels, and the in-app shortcut reference all use the same effective binding

## ADDED Requirements

### Requirement: New Tab has a default keyboard shortcut
The editor SHALL bind File → New Tab to a factory-default shortcut of `Ctrl+Shift+N` on Windows and Linux and `Cmd+Shift+N` on macOS when no override is stored. Invoking that effective shortcut SHALL perform the same action as File → New Tab and the tab-bar "+" control: append and activate a new empty document tab. The action SHALL use the stable id `new-tab` in the customizable-shortcut registry, and the File menu SHALL show the effective binding beside the New Tab item. Open in New Tab SHALL keep its existing factory default (`Ctrl+T` / `Cmd+T`).

#### Scenario: Default New Tab shortcut opens an empty tab
- **WHEN** the user presses the New Tab shortcut and no override is stored for `new-tab`
- **THEN** a new empty document tab is appended and activated
- **AND** other open tabs keep their document text, dirty flags, and derived Markdown caches

#### Scenario: New Tab shortcut is discoverable
- **WHEN** the user opens the File menu or the Tabs section of the in-app shortcut reference
- **THEN** New Tab displays `Ctrl+Shift+N` on Windows and Linux and `Cmd+Shift+N` on macOS, unless an override is stored

#### Scenario: New Tab shortcut remains customizable
- **WHEN** the user assigns or resets an override for `new-tab`
- **THEN** keymap dispatch, the File-menu label, and the shortcut reference all use the same effective binding
