## ADDED Requirements

### Requirement: In-view pointer selection in multiline Visual Edit table cells

Visual Edit SHALL support exact partial text selection in text-only GFM cells with wrapped Unicode text and authored HTML line breaks. Moving the caret or dragging a text selection within the viewport inset SHALL preserve the viewport even when the table spans multiple screenfuls. Clicks on either side of an authored break SHALL resolve to the corresponding source boundaries. Selection alone SHALL preserve the Markdown, version, dirty state, undo history, and shared per-version derivation.

#### Scenario: Click and drag below the first line in a tall table

- **WHEN** the user clicks or drags across visible text after a break in a cell of a GFM table taller than the viewport
- **THEN** the caret or selection resolves to the exact source text under the pointer
- **AND** the visible portion of the table does not scroll away

#### Scenario: Select text across an authored break

- **WHEN** the user drags from text before a cell break to text after it
- **THEN** the selected source includes the authored break and the intended portion of each line
- **AND** the selection does not alter the document
