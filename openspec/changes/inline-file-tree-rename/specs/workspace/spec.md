# Delta Spec: workspace

## MODIFIED Requirements

### Requirement: Tab rename reuses the file rename pipeline

The tab-context-menu Rename action SHALL rename the tab's file on disk through the same pipeline as the file-tree rename: the inline in-row name editor (rendered below the tab bar when the file-tree panel is not visible), unique-name collision avoidance, refusal while the document has unsaved changes (with a save-first status message), and re-pointing every open tab that referenced the old path to the renamed file. The name editor SHALL be visible and operable regardless of whether the file-tree panel is currently shown.

#### Scenario: Renaming a clean saved tab

- WHEN the user picks Rename on a clean, file-backed tab and confirms a new name in the name editor
- THEN the file is renamed on disk and the tab (and any duplicate tab for the old path) now refers to the renamed file, keeping its open state

#### Scenario: Dirty tab refuses rename

- WHEN the user picks Rename on a tab with unsaved changes
- THEN no name editor opens and a status message instructs the user to save first

#### Scenario: Tab rename without the file tree visible

- WHEN the user picks Rename on a tab while the sidebar is hidden or shows a different tab
- THEN the name editor renders below the tab bar with the same editing and commit behavior as in the file tree
