## MODIFIED Requirements

### Requirement: Multi-document tab model
The editor SHALL hold zero or more open content tabs within a single window (`tabs` plus an `active_tab` index), rather than a single document per window. A Markdown or curated text tab SHALL carry its own isolated document, cursor/selection, scroll position, undo/redo history, IME composition state, layout caches, dirty flag, and autosave/recovery tracking; a read-only image tab SHALL carry only the state required to identify, load, present, scroll, and close that image. Switching tabs SHALL NOT disturb another tab's state. Tabs for filesystem-backed content SHALL be unique by file path within a window: when an open request targets a file that is already open in another tab, the editor SHALL focus that existing tab instead of opening a duplicate tab. A tab bar SHALL be rendered only when more than one tab is open; with a single tab the active content surface SHALL use the same space as the pre-tab layout. Tabs are session-only: they are not persisted across launches (restarting returns to a single untitled document).

#### Scenario: Opening files creates switchable tabs with isolated state
- **WHEN** the user opens a second supported file via the file tree or the Open in New Tab action
- **THEN** a new document or image tab is appended and activated
- **AND** switching back to the first tab restores that tab's exact content-specific state

#### Scenario: Opening an already-open file focuses its existing tab
- **WHEN** the user opens a supported file by path and that same file is already open in a tab
- **THEN** the existing tab is activated
- **AND** no duplicate tab is appended or replaced
- **AND** an existing document tab preserves its text, dirty flag, cursor/selection, undo/redo history, editor scroll position, preview scroll position, and derived Markdown caches
- **AND** an existing image tab preserves its load result and presentation state

#### Scenario: File→Open replaces the active tab
- **WHEN** the user invokes File → Open and picks a supported file that is not already open
- **THEN** the active tab's content is replaced after applying a dirty guard when that tab contains an editable document, rather than spawning a new tab
- **AND** replacing a read-only image tab does not require a dirty confirmation

#### Scenario: Tab navigation and closing
- **WHEN** the user presses the next/previous tab shortcut (Ctrl+Tab / Ctrl+Shift+Tab) or clicks a tab / its close button
- **THEN** the active tab switches in opening order, or the targeted tab closes; closing the last tab creates a fresh untitled document rather than closing the window

#### Scenario: Closing an unsaved tab prompts for confirmation
- **WHEN** the user closes a document tab whose content has unsaved changes
- **THEN** the editor prompts for confirmation before discarding those changes
- **AND** closing a read-only image tab never presents an unsaved-changes prompt

#### Scenario: Quitting with multiple unsaved tabs
- **WHEN** the user quits or closes the window while two or more document tabs have unsaved changes and any number of image tabs are open
- **THEN** the editor detects the unsaved document tabs and prompts before discarding them
- **AND** image tabs do not contribute to the dirty-tab count

#### Scenario: Autosave targets the tab that was active when scheduled
- **WHEN** an autosave timer fires after the user has switched tabs
- **THEN** the autosave writes the document tab whose generation was captured at schedule time, not whichever tab is now active
- **AND** an image tab never becomes an autosave target

#### Scenario: Single-tab layout is unchanged
- **WHEN** only one content tab is open
- **THEN** no tab bar is rendered and the active document or image surface occupies the normal document-workspace area
