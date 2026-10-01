## ADDED Requirements

### Requirement: Find / Replace overlay SHALL be movable within the window
The Find / Replace overlay SHALL provide a visible move grip. Dragging the grip with the left mouse button SHALL move the whole overlay with the pointer. Fields, toggles, and buttons inside the overlay SHALL keep their existing mouse behavior and SHALL NOT start a move. The overlay SHALL first appear at its default upper-right position; after a move, closing and reopening the overlay in the same session SHALL reopen it at the moved position. The moved position SHALL NOT be persisted across launches. The whole overlay SHALL stay inside the window: positions SHALL be clamped while dragging and whenever the window size changes. Double-clicking the grip SHALL return the overlay to its default position. The grip's tooltip SHALL be localized in every supported UI language. Moving the overlay SHALL NOT change workspace layout and SHALL NOT mutate document text, dirty state, undo history, document version, or derived Markdown caches.

#### Scenario: Dragging the grip moves the overlay
- **WHEN** the Find / Replace overlay is visible and the user drags its grip to another point in the window
- **THEN** the overlay follows the pointer and stays where the drag ends
- **AND** the tab bar, editor pane, preview pane, and status bar keep their layout positions
- **AND** the document text, version, dirty state, and undo history are unchanged

#### Scenario: Overlay controls do not start a move
- **WHEN** the user presses and drags on a find or replace field, a toggle, or a button inside the overlay
- **THEN** the overlay does not move
- **AND** that control behaves as it did before this change (for example, a field drag selects text)

#### Scenario: Moved position is kept for the session only
- **WHEN** the user moves the overlay, closes it, and opens Find or Replace again
- **THEN** the overlay reopens at the moved position
- **AND** after the application restarts, the overlay opens at its default upper-right position

#### Scenario: Overlay cannot leave the window
- **WHEN** the user drags the grip past a window edge, or shrinks the window while the overlay sits near an edge
- **THEN** the whole overlay remains inside the window

#### Scenario: Double-clicking the grip resets the position
- **WHEN** the overlay has been moved and the user double-clicks its grip
- **THEN** the overlay returns to its default upper-right position
