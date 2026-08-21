# Delta Spec: inline-name-editing

## Purpose

Defines the inline name editor used by file-tree create/rename and the tab-bar Rename action: where it renders, its editing model (cursor, selection, base-name pre-selection), and its input-routing contract so that mouse and key events never leak into the document editor while a name is being edited.

## ADDED Requirements

### Requirement: The name editor renders in place

While a create or rename name editor is open, it SHALL render in the file-tree row of the entry being renamed (replacing that row's label), and for create actions in a new row at the position the entry will occupy. When the file-tree panel is not visible (sidebar hidden or on another sidebar tab), the editor SHALL render directly below the tab bar so it stays visible for the tab-bar Rename action. The editor SHALL display the buffer with a visible caret and selection highlight, and its row SHALL NOT scroll out of the visible tree area while open.

#### Scenario: Rename replaces the row label

- WHEN the user triggers rename on a tree entry
- THEN the entry's row shows an editable name field containing the current name instead of the static label
- AND a caret and any selection in the field are visible

#### Scenario: Editor stays visible for tab-bar rename without the tree

- WHEN the tab-bar Rename action opens the name editor while the file-tree panel is hidden
- THEN the editor renders below the tab bar and remains fully visible

### Requirement: Rename pre-selects the base name

When the name editor opens for a rename, it SHALL pre-select the base name of the current file name and leave the extension unselected, so typing replaces the base name in one stroke while preserving the extension. When no extension exists, the whole name SHALL be selected. When the editor opens for a create action, the whole prefilled name SHALL be selected.

#### Scenario: Typing replaces only the base name

- WHEN the editor opens for renaming `report.md` and the user types `notes` and presses Enter
- THEN the entry is renamed to `notes.md`

#### Scenario: Whole name selected for create

- WHEN the editor opens for creating a file with the prefilled name `untitled.md`
- THEN the entire prefilled name is selected and typing replaces it

### Requirement: Input routing while the name editor is open

While the name editor is open, the editor SHALL own keyboard and pointer input as follows:

- Mouse-down inside the name editor SHALL position the caret (or extend the selection with Shift) within the name buffer and SHALL NOT dismiss the editor, open a document, or move the document caret.
- Pointer interaction in the source editor pane SHALL NOT move the document caret or start a document selection while the editor is open; the document content and selection remain unchanged.
- Clicking another file-tree row while the editor is open SHALL NOT open that file.
- Left, Right, Home, End, and Select-All key bindings SHALL operate on the name buffer's caret and selection, not on the document.
- Character input SHALL replace the current selection in the name buffer; Backspace and Delete SHALL edit the name buffer at its caret.
- Enter SHALL commit the name and Escape SHALL cancel, leaving the filesystem untouched.
- A left mouse-down outside the name editor SHALL commit the current buffer through the same pipeline as Enter when the buffer names a valid, changed name, and SHALL leave the editor open with a status message when the buffer is empty or the commit is refused.

#### Scenario: Mouse-down inside the field keeps it open

- WHEN the editor is open and the user presses the left mouse button on the field itself
- THEN the editor remains open and the caret moves to the clicked character position

#### Scenario: Dragging in the source pane does not touch the document

- WHEN the editor is open and the user presses and drags the left mouse button across the source editor pane
- THEN the document's caret and selection are unchanged and the name editor remains open

#### Scenario: Arrow keys edit the name, not the document

- WHEN the editor is open with the caret mid-name and the user presses Left or Right
- THEN the name buffer's caret moves within the name and the document caret does not move

#### Scenario: Click-away commits

- WHEN the editor is open with a valid changed name and the user clicks anywhere outside the field
- THEN the rename or create proceeds exactly as if Enter had been pressed

#### Scenario: Escape cancels

- WHEN the editor is open and the user presses Escape
- THEN the editor closes, the buffer is discarded, and the filesystem is untouched

### Requirement: Editor state survives until resolved

The name editor SHALL remain open until the user resolves it via Enter, click-away commit, or Escape, or until an explicit UI action replaces it (opening a context menu, switching the rename target, refreshing the tree). No incidental mouse or key event SHALL silently close the editor.

#### Scenario: Idle mouse movement does not dismiss

- WHEN the editor is open and the user moves the mouse without clicking
- THEN the editor remains open

#### Scenario: Replacing the editor with a new action

- WHEN the editor is open and the user right-clicks a different tree entry and chooses Rename
- THEN the editor closes and reopens targeting the newly selected entry
