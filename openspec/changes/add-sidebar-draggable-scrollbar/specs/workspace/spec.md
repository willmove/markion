## ADDED Requirements

### Requirement: Overflowing file tree exposes a draggable scrollbar
The Files sidebar list SHALL provide a visible, right-side vertical scrollbar whenever the currently rendered file-tree rows exceed the visible list height. Dragging that scrollbar with the left mouse button SHALL change the visible rows and SHALL update the thumb position to match the list's scroll offset. Wheel and trackpad scrolling SHALL continue to work. The thumb SHALL hide when the rendered rows fit. Workspace scanning, hidden-entry filtering, interactive expansion, the bounded number of rows built per frame, and horizontal overflow for long names SHALL remain unchanged.

#### Scenario: Overflowing file tree exposes a scrollbar
- **WHEN** the Files sidebar is visible
- **AND** the rendered file-tree rows do not fit in the visible list height
- **THEN** a right-side vertical scrollbar thumb is shown
- **AND** dragging the thumb with the left mouse button scrolls the file-tree rows

#### Scenario: Fitting file tree hides the scrollbar
- **WHEN** the Files sidebar is visible
- **AND** the rendered file-tree rows fit in the visible list height
- **THEN** no vertical scrollbar thumb is shown for the file tree

#### Scenario: File tree wheel scrolling is preserved
- **WHEN** the Files sidebar is visible
- **AND** the user scrolls the file-tree list with the mouse wheel or trackpad
- **THEN** the list scrolls as it did before the draggable scrollbar was added
- **AND** the scrollbar thumb, when shown, moves to reflect the new scroll offset

#### Scenario: File tree bounded rendering is unchanged
- **WHEN** a workspace scan yields more matching entries than the per-frame row cap
- **THEN** the file tree still renders only that bounded number of rows plus the existing overflow hint
- **AND** the scrollbar extent tracks those rendered rows rather than forcing a full uncapped tree
