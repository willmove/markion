## ADDED Requirements

### Requirement: Visual Edit caret moves become visible without follow-up input
When Visual Edit accepts a caret-moving keyboard action or edit, the system SHALL update the canonical source selection exactly once and SHALL paint the caret at the resolved source-backed target without requiring another key press, text input, pointer event, or unrelated redraw. If the target row already has current layout geometry, the next presented frame SHALL show the caret at that target. If a virtualized target must first be revealed and measured, the measurement pass SHALL independently schedule bounded post-layout completion and a subsequent draw; the first presented frame after current target geometry becomes available SHALL show the moved caret. Presentation-only completion and follow-up frames MUST NOT change document text, version, dirty state, undo/redo history, or per-version derived cache identity.

#### Scenario: Enter presents the new insertion-line caret after one press
- **WHEN** the user presses Enter once at a supported Visual Edit caret position
- **THEN** the canonical source receives exactly the structural or plain newline required by the existing editing rules
- **AND** the newly owning rendered or whitespace row paints the caret without another user action

#### Scenario: Cached cross-block navigation paints after one action
- **WHEN** the user presses Up or Down across a visual-block boundary whose target row has current layout geometry
- **THEN** the source caret moves to the closest valid position in the adjacent block
- **AND** the next presented frame paints the caret at that position without waiting for a second navigation action

#### Scenario: Virtualized cross-block navigation completes after measurement
- **WHEN** a single Up or Down action targets a visual block that is not currently measured
- **THEN** Visual Edit reveals and measures the target row
- **AND** it completes the pending source-caret move and schedules the resulting paint without further input
- **AND** stale deferred work is discarded if the document version, active tab, target block identity, or navigation request changes before completion

#### Scenario: Selection navigation uses the same presentation guarantee
- **WHEN** Select Up or Select Down crosses a visual-block boundary
- **THEN** the canonical selection head resolves through the same current or deferred geometry path as ordinary navigation
- **AND** the painted selection and caret reflect that result without another user action

#### Scenario: Blank-line navigation keeps one authored stop
- **WHEN** a single Up or Down action moves from rendered content onto an adjacent authored blank-line row
- **THEN** that whitespace row immediately becomes the caret-owning painted row
- **AND** a second action is still required to continue to the rendered block on the far side, preserving the existing first-class blank-line navigation contract

#### Scenario: Idle compositor does not delay the caret
- **WHEN** a caret move requires a post-layout frame and no other hover, animation, background notification, or user input occurs
- **THEN** Visual Edit still schedules and presents the frame containing the resolved caret

#### Scenario: Repaint completion preserves document caches
- **WHEN** Visual Edit performs deferred caret completion or a bounded caret-follow frame without changing source text
- **THEN** document text, version, dirty state, undo/redo history, shared Markdown and Visual Edit cache identities, memoized highlighting, and the cached text handle remain unchanged
