## ADDED Requirements

### Requirement: Tab bar context menu

When two or more tabs are open, the editor SHALL show a context menu when the user right-clicks a tab-bar tab. The menu SHALL offer actions targeting the right-clicked tab: Close Tab, Close Others, Close to the Right, Rename, Copy File Path, and Reveal in File Manager. Activating an action SHALL first make the clicked tab the active tab, then perform the action (the same switch-then-operate idiom as the tab close button). The menu SHALL close on click-away, on Escape-equivalent dismissal paths, and when any other menu opens, and only one context menu SHALL be open at a time. Items that require a file-backed tab (Rename, Copy File Path, Reveal in File Manager) SHALL be disabled for untitled tabs. Middle-clicking a tab SHALL close it with the same behavior as the Close Tab action.

#### Scenario: Right-click opens the menu targeting the clicked tab

- **WHEN** two or more tabs are open and the user right-clicks a tab that is not active
- **THEN** a context menu appears at the pointer with all tab actions
- **AND** choosing Close Tab first activates the clicked tab and then closes it, running the existing dirty-document confirmation

#### Scenario: Untitled tab disables file-backed items

- **WHEN** the user right-clicks a tab whose document has never been saved to disk
- **THEN** Rename, Copy File Path, and Reveal in File Manager are visually disabled and dispatch nothing
- **AND** the close actions remain available

#### Scenario: Middle-click closes a tab

- **WHEN** the user middle-clicks a tab
- **THEN** that tab activates and closes exactly as the Close Tab context-menu action would

#### Scenario: Menu closes and stays exclusive

- **WHEN** a tab context menu is open and the user clicks elsewhere in the window or opens another menu (menu bar, file tree, preview)
- **THEN** the tab context menu closes without dispatching an action

### Requirement: Batch tab closing preserves dirty tabs by default

The Close Others and Close to the Right actions SHALL close only tabs with no unsaved changes. If one or more of the tabs in scope are dirty, the editor SHALL keep every dirty tab open, clean up only the clean tabs, and show a summary dialog stating how many tabs were kept because of unsaved changes. The dialog SHALL offer an explicit "discard all" choice that, when confirmed, closes the kept dirty tabs and discards their changes through the existing discard path (including recovery-file cleanup); declining it SHALL leave the dirty tabs open. Dirty tabs SHALL never be closed silently.

#### Scenario: Clean tabs close silently

- **WHEN** the user chooses Close Others and every other tab is clean
- **THEN** all other tabs close immediately with no dialog, and the clicked tab becomes the only tab

#### Scenario: Dirty tabs are kept and reported

- **WHEN** the user chooses Close Others and two other tabs have unsaved changes
- **THEN** all clean other tabs close, the two dirty tabs remain open
- **AND** a summary dialog reports the kept dirty tabs and offers a discard-all confirmation

#### Scenario: Discard all closes the dirty tabs

- **WHEN** the summary dialog is confirmed with the discard-all choice
- **THEN** the kept dirty tabs close and their recovery snapshots are discarded via the existing close/discard path
- **AND** declining the dialog leaves the kept dirty tabs open with their edits intact

### Requirement: Tab rename reuses the file rename pipeline

The tab-context-menu Rename action SHALL rename the tab's file on disk through the same pipeline as the file-tree rename: an inline name prompt, unique-name collision avoidance, refusal while the document has unsaved changes (with a save-first status message), and re-pointing every open tab that referenced the old path to the renamed file. The inline prompt SHALL be visible regardless of whether the file-tree panel is currently shown.

#### Scenario: Renaming a clean saved tab

- **WHEN** the user picks Rename on a clean, file-backed tab and confirms a new name in the inline prompt
- **THEN** the file is renamed on disk and the tab (and any duplicate tab for the old path) now refers to the renamed file, keeping its open state

#### Scenario: Dirty tab refuses rename

- **WHEN** the user picks Rename on a tab with unsaved changes
- **THEN** no prompt opens and a status message instructs the user to save first
