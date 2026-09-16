## ADDED Requirements

### Requirement: Rendered superscript and subscript geometry

Rendered superscript and subscript SHALL use a reduced font size and a raised or lowered baseline relative to surrounding text, including inside links and tables. Geometry SHALL scale with document typography. Selection, pointer placement, search highlighting, visual navigation and IME bounds SHALL use the actual shaped fragment coordinates. Revealed source SHALL use normal source presentation without mutating document state.

#### Scenario: Script glyphs move and shrink
- **WHEN** rendered text contains H followed by subscript 2 or x followed by superscript 2
- **THEN** the script glyph is smaller and respectively lower or higher than the surrounding baseline
- **AND** following normal text returns to its original baseline

#### Scenario: Script interaction remains source backed
- **WHEN** users select or focus rendered script content at different document font sizes
- **THEN** hit testing and caret/IME bounds correspond to the displayed glyphs
- **AND** source reveal and undo preserve the authored markup and UTF-8 boundaries
