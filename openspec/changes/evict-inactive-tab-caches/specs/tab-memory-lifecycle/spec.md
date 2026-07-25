## ADDED Requirements

### Requirement: Inactive tabs may drop derived caches
Markion SHALL be allowed to drop per-tab derived Markdown caches and editor layout snapshots for any tab that is not the active tab. Dormancy MUST retain the document's canonical text, path, dirty flag, and text version, as well as the tab's selection, undo/redo history, and scroll-position handles. Dormancy MUST NOT mark the document dirty and MUST NOT bump `text_version`.

#### Scenario: Switching away dormants the previous tab
- **WHEN** the user activates a different tab after the previous tab has populated visual or preview derived caches
- **THEN** the previous tab's derived Markdown caches and shaped-line snapshots are cleared
- **AND** its text, dirty flag, text version, selection, and undo history remain unchanged

#### Scenario: Active tab stays warm
- **WHEN** a tab is active and its view mode requires derived blocks
- **THEN** those caches may remain populated for the active tab and are not cleared solely because other tabs were dormanted

### Requirement: Reactivation rebuilds lazily
Activating a dormant tab SHALL rebuild whatever derived state the current view mode needs through the existing lazy accessors. After reactivation, editing, navigation, and undo/redo SHALL behave as they did before dormancy for the retained selection and history. A dormant round-trip MUST NOT invent document edits.

#### Scenario: Visual Edit reactivation restores editable blocks
- **WHEN** a dormant tab is reactivated in Visual Edit mode
- **THEN** visual blocks are derived again for the current text version
- **AND** the retained selection still refers to the same source offsets

#### Scenario: Undo survives dormancy
- **WHEN** a tab with a non-empty undo stack is dormanted and later reactivated
- **THEN** undo still restores the pre-edit text from that history

### Requirement: Dormancy is measurable
Memory accounting SHALL report dormant tabs with unpopulated derived-cache sites (zero estimated bytes for those sites) while still reporting document text bytes. Harness or GPUI tests SHALL prove that opening a second warmed tab and then switching away from it reduces that tab's accounted derived bytes without reducing the active tab's text bytes.

#### Scenario: Harness observes per-tab drop after deactivate
- **WHEN** two tabs are warmed and the first is made active
- **THEN** the inactive tab's preview/visual/shaped-line accounted sites are zero
- **AND** the active tab still reports its document text bytes
