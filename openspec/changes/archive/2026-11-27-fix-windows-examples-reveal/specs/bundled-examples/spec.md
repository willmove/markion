## MODIFIED Requirements

### Requirement: Discoverable and non-destructive example access
Both the native and in-window Help menus SHALL offer a localized Examples and Templates action without a default shortcut. Invoking it SHALL reveal the bundled example index in the system file manager, dismiss the active menu, and leave the workspace and all open documents unchanged. A missing collection SHALL produce localized status feedback without crashing or creating an empty directory. If the Windows file manager cannot be launched, Markion SHALL show localized launch failure feedback in the status bar. The welcome document SHALL identify the Help entry. The index SHALL instruct users to copy examples before editing because installed resources may be read-only or replaced by upgrades, and SHALL explain that image-bearing copies must retain their resource directory.

#### Scenario: Open examples while editing
- **WHEN** the user invokes Help → Examples and Templates with unsaved edits in an open document
- **THEN** the system file manager reveals the bundled index
- **AND** document text, dirty flags, undo history, workspace selection, and derived caches are unchanged

#### Scenario: Examples are missing
- **WHEN** the example index cannot be found in application resources
- **THEN** Markion reports a localized missing-examples status and dismisses the menu
- **AND** no document or directory is created or replaced

#### Scenario: Windows resource path contains spaces or non-ASCII characters
- **WHEN** the user invokes Help → Examples and Templates and the Windows resource path contains spaces or non-ASCII characters
- **THEN** Explorer opens the bundled examples directory and selects `README.md`

#### Scenario: Windows file manager cannot launch
- **WHEN** the bundled index exists and Markion cannot launch the Windows file manager for Help → Examples and Templates
- **THEN** Markion dismisses the menu and displays a localized file manager error in the status bar
