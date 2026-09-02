## MODIFIED Requirements

### Requirement: Multi-document tab model
The editor SHALL hold zero or more open content tabs within a single window (`tabs` plus an `active_tab` index), rather than a single document per window. A Markdown or curated text tab SHALL carry its own isolated document, cursor/selection, scroll position, undo/redo history, IME composition state, layout caches, dirty flag, and autosave/recovery tracking; a read-only image tab SHALL carry only the state required to identify, load, present, scroll, and close that image. Switching tabs SHALL NOT disturb another tab's state. Tabs for filesystem-backed content SHALL be unique by file path within a window: when an open request targets a file that is already open in another tab, the editor SHALL focus that existing tab instead of opening a duplicate tab. A tab bar SHALL be rendered only when more than one tab is open; with a single tab the active content surface SHALL use the same space as the pre-tab layout. Tabs are session-only: they are not persisted across launches (restarting returns to a single untitled document).

Every open request that does not explicitly ask for a new tab — File → Open, a file-tree click, the file-tree context-menu Open action, drag-and-drop of supported documents onto a pane, and Open Recent — SHALL resolve its tab target from the persisted "Open documents in current tab" preference (default on). While the preference is on, such an open SHALL replace the active tab in place only when that tab is an image tab or a clean (non-dirty) document tab, including a pristine untitled or welcome document; a dirty editable document, whether or not it has a filesystem path, SHALL cause the open to append a new tab without prompting and without discarding text, undo/redo history, or a recovery snapshot. While the preference is off, every non-explicit open SHALL append a new tab. Explicit new-tab affordances — File → Open in New Tab, the tab bar's new-tab button, the new-tab shortcut, the file-tree context-menu Open in New Tab action, and Ctrl/Cmd+click on a file-tree row — SHALL always append a new tab regardless of the preference. Replacing a clean tab discards that tab's undo/redo history and scroll state but SHALL never discard unsaved content or delete a recovery snapshot belonging to a dirty document. When several supported files are opened in one batch, the first file SHALL follow the default rule above and every subsequent file SHALL append its own tab.

Closing a dirty document tab, quitting, or closing the window while any document tab is dirty SHALL present a three-way confirmation with localized Save, Don't Save, and Cancel actions. Save SHALL persist the affected dirty document(s) through the existing save path (Save As when a document has no path) and SHALL proceed with the close or quit only after those saves succeed; a failed save, an unresolved external-file conflict, or a cancelled Save As SHALL abort the close or quit and leave remaining dirty documents open. Don't Save SHALL close or quit through the existing discard path, retiring every affected tab's recovery snapshot. Cancel SHALL leave tabs, text, dirty flags, and recovery snapshots unchanged. Image tabs SHALL never participate in this prompt.

#### Scenario: Opening files creates switchable tabs with isolated state
- **WHEN** the user opens a second supported file via the Open in New Tab action, the tab bar's new-tab button, or any non-explicit open while the open-in-current-tab preference is off
- **THEN** a new document or image tab is appended and activated
- **AND** switching back to the first tab restores that tab's exact content-specific state

#### Scenario: Opening an already-open file focuses its existing tab
- **WHEN** the user opens a supported file by path and that same file is already open in a tab
- **THEN** the existing tab is activated
- **AND** no duplicate tab is appended or replaced
- **AND** an existing document tab preserves its text, dirty flag, cursor/selection, undo/redo history, editor scroll position, preview scroll position, and derived Markdown caches
- **AND** an existing image tab preserves its load result and presentation state

#### Scenario: File→Open follows the open-target preference
- **WHEN** the user invokes File → Open and picks a supported file that is not already open
- **THEN** with the open-in-current-tab preference on and a replaceable active tab, the active tab's content is replaced without a discard prompt
- **AND** with the preference off, or with a dirty active document tab, a new tab is appended and activated without a discard prompt
- **AND** replacing a read-only image tab does not require an unsaved-changes confirmation

#### Scenario: Default open replaces a clean active tab
- **WHEN** the open-in-current-tab preference is on, the active tab is an image tab or a clean document tab (including a pristine untitled or welcome document), and the user opens another supported file from the file tree, a drag-drop, or Open Recent
- **THEN** the active tab is replaced in place by the new content without any prompt
- **AND** the replaced tab's undo history and scroll state are discarded, but no unsaved content or recovery snapshot is lost

#### Scenario: Default open never discards dirty work
- **WHEN** the open-in-current-tab preference is on, the active editable document tab is dirty (named or untitled), and the user opens another supported file from the file tree, a drag-drop, Open Recent, or File → Open
- **THEN** a new tab is appended and activated without prompting
- **AND** the dirty tab keeps its text, dirty flag, undo/redo history, and recovery snapshot untouched

#### Scenario: Ctrl/Cmd+click forces a new tab
- **WHEN** the user Ctrl/Cmd+clicks a supported file in the file tree while the open-in-current-tab preference is on
- **THEN** the file opens in a new appended tab even though the active tab is clean
- **AND** plain clicks continue to follow the preference

#### Scenario: Multi-file drop replaces once then appends
- **WHEN** the open-in-current-tab preference is on and the user drops several supported documents at once
- **THEN** the first dropped file follows the default rule, replacing a replaceable active tab, and each subsequent file opens in its own appended tab
- **AND** the last opened file is the active tab

#### Scenario: Tab navigation and closing
- **WHEN** the user presses the next/previous tab shortcut (Ctrl+Tab / Ctrl+Shift+Tab) or clicks a tab / its close button
- **THEN** the active tab switches in opening order, or the targeted tab closes; closing the last tab creates a fresh untitled document rather than closing the window

#### Scenario: Closing an unsaved tab offers Save, Don't Save, and Cancel
- **WHEN** the user closes a document tab whose content has unsaved changes
- **THEN** the editor prompts with Save, Don't Save, and Cancel
- **AND** Save persists that tab (Save As when it has no path) and then closes it
- **AND** Don't Save closes the tab and discards its recovery snapshot
- **AND** Cancel leaves the tab open with its edits intact
- **AND** closing a read-only image tab never presents an unsaved-changes prompt

#### Scenario: Quitting with unsaved tabs offers Save, Don't Save, and Cancel
- **WHEN** the user quits or closes the window while one or more document tabs have unsaved changes and any number of image tabs are open
- **THEN** the editor prompts with Save, Don't Save, and Cancel covering those dirty document tabs
- **AND** Save persists every dirty document tab (Save As for each untitled tab) and then exits only if every save succeeds
- **AND** Don't Save exits and retires every dirty tab's recovery snapshot
- **AND** Cancel leaves the window open with every tab unchanged
- **AND** image tabs do not contribute to the dirty-tab count

#### Scenario: Autosave targets the tab that was active when scheduled
- **WHEN** an autosave timer fires after the user has switched tabs
- **THEN** the autosave writes the document tab whose generation was captured at schedule time, not whichever tab is now active
- **AND** an image tab never becomes an autosave target

#### Scenario: Single-tab layout is unchanged
- **WHEN** only one content tab is open
- **THEN** no tab bar is rendered and the active document or image surface occupies the normal document-workspace area
