## ADDED Requirements

### Requirement: Stable typewriter positioning in small viewports

Typewriter mode SHALL keep the active row stable during consecutive typing in small editable viewports, including short documents, newline insertion, and IME composition. Once current geometry centers an unchanged caret row, subsequent refinement frames SHALL NOT move it away and back. A new row SHALL converge without alternating between incompatible scroll targets. Presentation-only scrolling SHALL preserve document state and per-version caches.

#### Scenario: Consecutive input in a short document
- **WHEN** typewriter mode is enabled in a small window and the user types into an initially empty document
- **THEN** characters on the same visual row do not cause the page to jump away from its centered position
- **AND** intermediate refinement frames preserve the centered row once current geometry is available

#### Scenario: Newline followed by IME composition
- **WHEN** the user inserts a newline in a short document and composes and commits text on the new row
- **THEN** the row converges to the viewport center without back-and-forth displacement
- **AND** the centering operation does not mutate text, undo history, or derived caches

#### Scenario: Small viewport with wrapped content
- **WHEN** the active paragraph wraps across multiple visual rows in a small viewport
- **THEN** typewriter centering follows the active row and remains stable across unchanged frames
- **AND** manual scrolling remains possible until the next caret activity
