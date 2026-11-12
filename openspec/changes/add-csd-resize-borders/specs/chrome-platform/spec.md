## ADDED Requirements

### Requirement: Mouse edge and corner resizing on client-side-decorated sessions
Where the platform reports client-side decorations, the main window SHALL reserve a transparent client margin around the visible window and SHALL provide resize grips in that margin for every non-tiled edge and corner. Hovering a grip SHALL show the matching native resize cursor, and pressing it SHALL start a compositor resize for that edge or corner. The margin and grips SHALL be absent when the window is maximized or fullscreen, and on each edge the compositor reports tiled. Grips SHALL NOT cover interactive content: edge grips lie wholly outside the content rectangle, and corner grips sit below the content layer in hit order. Platforms reporting server-side decorations SHALL render and behave exactly as before.

#### Scenario: Hovering an edge shows the resize cursor
- **WHEN** the pointer rests on the right or left margin of a floating CSD window
- **THEN** the cursor is the horizontal resize cursor
- **WHEN** the pointer rests on the top or bottom margin
- **THEN** the cursor is the vertical resize cursor

#### Scenario: Dragging an edge or corner resizes the window
- **WHEN** the user presses and drags a margin grip
- **THEN** the compositor resizes the window from that edge or corner

#### Scenario: Corner grips use diagonal cursors
- **WHEN** the pointer rests on a corner grip
- **THEN** the cursor is the matching diagonal resize cursor

#### Scenario: Content hit areas are untouched
- **WHEN** the user drags the preview scrollbar or clicks a file-tree row at the content edge
- **THEN** the interaction behaves exactly as before, because no grip overlaps the content rectangle

#### Scenario: Maximized, fullscreen, and tiled windows have no grips
- **WHEN** the window is maximized or fullscreen, or an edge is tiled
- **THEN** the margin is zero on the affected edges and no grip is present there

#### Scenario: Server-side-decorated platforms are unchanged
- **WHEN** the platform reports server-side decorations
- **THEN** no margin, grips, or shadow are added and resizing remains the platform's own