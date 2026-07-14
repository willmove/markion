## ADDED Requirements

### Requirement: Multi-document tab model
The editor SHALL hold zero or more open documents as tabs within a single window (`tabs: Vec<EditorTab>` + an `active_tab` index), rather than a single document per window. Each tab SHALL carry its own isolated document, cursor/selection, scroll position, undo/redo history, IME composition state, layout caches, dirty flag, and autosave/recovery tracking — switching tabs SHALL NOT disturb another tab's state. A tab bar SHALL be rendered only when more than one tab is open; with a single tab the editor looks identical to the pre-tab single-document layout. Tabs are session-only: they are not persisted across launches (restarting returns to a single untitled document).

#### Scenario: Opening files creates switchable tabs with isolated state
- **WHEN** the user opens a second file (via the file tree, or the OpenInNewTab action)
- **THEN** a new tab is appended and activated, and switching back to the first tab restores its exact cursor position, scroll offset, and undo history

#### Scenario: File→Open replaces the active tab
- **WHEN** the user invokes File→Open and picks a file
- **THEN** the active tab's document is replaced (after a dirty-guard on that tab), matching the single-document behavior, rather than spawning a new tab

#### Scenario: Tab navigation and closing
- **WHEN** the user presses the next/previous tab shortcut (Ctrl+Tab / Ctrl+Shift+Tab) or clicks a tab / its close button
- **THEN** the active tab switches in opening order, or the targeted tab closes; closing the last tab creates a fresh untitled document rather than closing the window

#### Scenario: Closing an unsaved tab prompts for confirmation
- **WHEN** the user closes a tab whose document has unsaved changes
- **THEN** the editor prompts for confirmation before discarding those changes

#### Scenario: Quitting with multiple unsaved tabs
- **WHEN** the user quits or closes the window while two or more tabs have unsaved changes
- **THEN** the editor detects the unsaved tabs and prompts before discarding them

#### Scenario: Autosave targets the tab that was active when scheduled
- **WHEN** an autosave timer fires after the user has switched tabs
- **THEN** the autosave writes the tab whose generation was captured at schedule time, not whichever tab is now active

#### Scenario: Single-tab layout is unchanged
- **WHEN** only one tab is open
- **THEN** no tab bar is rendered and the editor's appearance matches the pre-tab single-document layout
