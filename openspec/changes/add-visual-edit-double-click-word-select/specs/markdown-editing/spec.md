## ADDED Requirements

### Requirement: Visual Edit double-click word selection
When Visual Edit is active, a double click of the primary pointer button on rendered editable text SHALL select the maximal contiguous run of same-class characters surrounding the clicked position in the displayed text, replacing the caret with that selection so immediate follow-up editing (formatting, replacement, copy) applies to the run. Character classes SHALL be word characters (letters and digits, including CJK ideographs and kana), punctuation, and whitespace, so a double click selects the word, the contiguous CJK run between whitespace or punctuation, or the punctuation/whitespace run respectively. The selection SHALL be the canonical contiguous source range whose rendered projection is that displayed run: hidden Markdown syntax at the selection edges SHALL be excluded, hidden syntax inside the run SHALL remain within the selection, and boundaries SHALL resolve to valid UTF-8 character boundaries. A double click on a rendered inline atom (such as inline math or an inline image) SHALL select the atom's authored source range. If no non-empty source range resolves for the clicked run, the editor SHALL fall back to the existing single-click caret placement. A pointer click with the Shift modifier held SHALL keep the existing extend-selection behavior regardless of click count, and a drag that continues after a double-click word selection SHALL extend the selection under the existing drag-selection rules. Double-click word selection is pointer-only interaction: it SHALL NOT change document text, the document version, the dirty flag, the undo/redo history, or any derived Markdown cache, and an in-viewport double click SHALL follow the existing viewport-preservation rules for pointer placement.

#### Scenario: Double-click selects the word at the pointer
- **WHEN** the user double-clicks inside an English word in a rendered Visual Edit paragraph
- **THEN** the whole word is selected, from its first to its last character
- **AND** the selection is ready for immediate formatting, replacement, or copy

#### Scenario: Double-click selects a contiguous CJK run
- **WHEN** rendered Visual Edit text contains a Chinese phrase surrounded by whitespace or punctuation and the user double-clicks any character of the phrase
- **THEN** the contiguous run of CJK characters is selected up to the surrounding whitespace or punctuation

#### Scenario: Double-click on punctuation or whitespace selects that run
- **WHEN** the user double-clicks a punctuation character or a whitespace sequence in rendered Visual Edit text
- **THEN** the contiguous punctuation or whitespace run at that position is selected instead of a word

#### Scenario: Word selection excludes hidden syntax at its edges
- **WHEN** rendered bold text such as `**word**` displays as "word" and the user double-clicks it
- **THEN** the selection covers exactly the displayed word content and not the hidden emphasis markers at the edges
- **AND** typing over the selection replaces the word content while the emphasis formatting is preserved

#### Scenario: Word selection spans hidden syntax inside the run
- **WHEN** split emphasis such as `bo**ld**` displays as "bold" and the user double-clicks the displayed word
- **THEN** the selection is one contiguous source range covering both visible halves and the hidden markers between them

#### Scenario: Double-click on a rendered inline atom selects its source
- **WHEN** the user double-clicks a rendered inline atom such as inline math or an inline image in Visual Edit
- **THEN** the atom's authored source range is selected

#### Scenario: Modifier and follow-up pointer behavior is unchanged
- **WHEN** the user Shift-clicks, single-clicks, or drag-selects Visual Edit text, including with repeated clicks
- **THEN** the existing placement, extend-selection, and drag-selection behaviors apply unchanged

#### Scenario: Double-click selection does not mutate document state
- **WHEN** the user double-clicks rendered Visual Edit text that is fully inside the viewport
- **THEN** the document text, version, dirty flag, undo/redo history, and derived Markdown caches remain unchanged
- **AND** the viewport does not move
