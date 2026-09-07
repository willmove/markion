## MODIFIED Requirements

### Requirement: Visual Edit IME composition fidelity
Visual Edit SHALL treat the active IME marked range as first-class projection and rendering state. The marked source SHALL remain visibly identified, precisely mapped, and correctly positioned for the platform candidate window throughout composition, including UTF-16 input containing CJK text, emoji, or combining characters. Native Windows IME preedit SHALL remain in the running editor process from the first IME-owned key through commit or cancellation. On Linux X11, valid XIM COMPOUND_TEXT preedit and commit payloads for Chinese GB2312 and Korean KS C 5601 SHALL decode to UTF-8 and follow the same composition path. An undecodable XIM payload MUST NOT terminate Markion or insert undecoded bytes into canonical source; any active composition SHALL end at its last successfully decoded source and selection state before the failed XIM connection is abandoned.

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

#### Scenario: Linux X11 accepts legacy Chinese and Korean compound text
- **WHEN** an XIM server sends a valid GB2312 or KS C 5601 COMPOUND_TEXT preedit or commit while a Linux X11 document surface owns text input
- **THEN** Markion remains running and presents the corresponding UTF-8 composition or committed text
- **AND** the decoded text participates in the existing marked-range, selection, and undo behavior

#### Scenario: Undecodable XIM payload fails safely
- **WHEN** an XIM server sends malformed or unsupported COMPOUND_TEXT during an active Linux X11 composition
- **THEN** Markion remains running and does not apply the undecodable payload to canonical source
- **AND** the active marked range is cleared while canonical source and selection remain at their last successfully decoded state
- **AND** direct keyboard input remains usable after the failed XIM connection is abandoned
