## ADDED Requirements

### Requirement: Open Folder has a default keyboard shortcut
The editor SHALL bind File → Open Folder to a factory-default shortcut of `Ctrl+Shift+O` on Windows and Linux and `Cmd+Shift+O` on macOS when no override is stored. Invoking that effective shortcut SHALL perform the same action as File → Open Folder: open the directory picker and, on selection, establish the workspace root. The action SHALL use the stable id `open-folder` in the customizable-shortcut registry, and the File menu SHALL show the effective binding beside the Open Folder item. Open document SHALL keep its existing factory default (`Ctrl+O` / `Cmd+O`).

#### Scenario: Default Open Folder shortcut opens the directory picker
- **WHEN** the user presses the Open Folder shortcut and no override is stored for `open-folder`
- **THEN** the Open Folder directory picker is presented
- **AND** confirming a directory establishes the workspace root as today

#### Scenario: Open Folder shortcut is discoverable
- **WHEN** the user opens the File menu or the Files section of the in-app shortcut reference
- **THEN** Open Folder displays `Ctrl+Shift+O` on Windows and Linux and `Cmd+Shift+O` on macOS, unless an override is stored

#### Scenario: Open Folder shortcut remains customizable
- **WHEN** the user assigns or resets an override for `open-folder`
- **THEN** keymap dispatch, the File-menu label, and the shortcut reference all use the same effective binding
