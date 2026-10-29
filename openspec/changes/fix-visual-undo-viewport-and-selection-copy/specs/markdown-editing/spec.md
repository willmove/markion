## ADDED Requirements

### Requirement: Visual Edit history preserves the editing viewport
Undo and Redo in Visual Edit SHALL preserve the current visible editing location when the restored caret remains visible. History restoration SHALL NOT reset the document viewport to its beginning. If the restored caret is outside the viewport, the editor SHALL reveal it with bounded scrolling. These behaviors SHALL apply to both full and compact history entries while preserving exact source/selection restoration, monotonic document versions, and per-version derived caches.

#### Scenario: Undo and redo a visible text edit
- **WHEN** the user edits text in a scrolled Visual Edit document and invokes Undo or Redo
- **THEN** source and selection are restored and the same surrounding content stays at the same viewport position when layout permits
- **AND** the restored caret is visible without jumping to the document beginning

#### Scenario: Restore a caret outside the current viewport
- **WHEN** Undo or Redo restores a caret outside the currently visible document region
- **THEN** the viewport moves enough to reveal that caret rather than resetting to the document beginning

### Requirement: Visual Edit text selection exposes context-menu Copy
The Visual Edit context menu SHALL expose a localized Copy command for a non-empty canonical text selection, including selections across blocks. Copy SHALL write the same text as the existing editor Copy shortcut and dismiss the menu without changing source, selection, document version, dirty state, history, derived cache identity, or document scroll position. An empty selection SHALL NOT expose an enabled selection Copy command.

#### Scenario: Copy a selection with the pointer
- **WHEN** the user selects Visual Edit text, right-clicks a block, and chooses Copy
- **THEN** the clipboard receives the selected text and the context menu closes
- **AND** document state, selection and viewport remain unchanged

#### Scenario: Copy a selection across blocks
- **WHEN** a Visual Edit selection spans multiple blocks and the context menu opens
- **THEN** Copy remains available even when inline formatting is unavailable for that selection

#### Scenario: Empty selection
- **WHEN** the Visual Edit context menu opens with an empty text selection
- **THEN** it does not expose an enabled selection Copy command
