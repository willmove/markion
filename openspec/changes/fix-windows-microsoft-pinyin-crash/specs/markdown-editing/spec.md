## MODIFIED Requirements

### Requirement: Visual Edit IME composition fidelity
Visual Edit SHALL treat the active IME marked range as first-class projection and rendering state. The marked source SHALL remain visibly identified, precisely mapped, and correctly positioned for the platform candidate window throughout composition, including UTF-16 input containing CJK text, emoji, or combining characters. Native Windows IME preedit SHALL remain in the running editor process from the first IME-owned key through commit or cancellation.

#### Scenario: Marked text is visible in the mixed projection
- **WHEN** an IME composition creates or updates a non-empty marked range inside rendered inline content
- **THEN** Visual Edit reveals any exact containing syntax needed to identity-map the marked source
- **AND** the painted marked range uses the platform composition underline without losing its inline content

#### Scenario: Candidate geometry follows the active marked range
- **WHEN** GPUI requests bounds for the active composition after the owning visual row has been laid out
- **THEN** Visual Edit returns geometry derived from the requested projected range
- **AND** the surface-level fallback is used only while exact row geometry is unavailable

#### Scenario: One IME composition is one undoable action
- **WHEN** an IME session produces multiple intermediate marked-text replacements and then commits
- **THEN** one Undo restores the source and selection from before that composition began
- **AND** one Redo reapplies the committed composition result

#### Scenario: UTF-16 composition remains UTF-8 safe
- **WHEN** IME replacement or selection ranges include CJK text, emoji, or combining characters
- **THEN** boundary conversion, projection, and marked-range painting resolve to valid canonical UTF-8 boundaries
- **AND** no partial code point is inserted, selected, or underlined

#### Scenario: Microsoft Pinyin begins composition without terminating the editor
- **WHEN** a Windows user starts Microsoft Pinyin preedit at a valid Visual Edit caret, including beside non-ASCII source text
- **THEN** Markion remains running while the IME owns and updates the composition
- **AND** committing or cancelling the composition leaves a valid canonical source selection
