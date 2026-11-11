## ADDED Requirements

### Requirement: Explicit structured Visual Edit selection
Visual Edit SHALL distinguish text, contiguous source-block and rectangular GFM cell selections. Structured selections SHALL be isolated per tab, carry the document version and exact block identity, and be rejected when stale. Escape SHALL enter current-block selection from text and return to text from a structured selection. Shift with block navigation or block/cell pointer selection SHALL extend the corresponding selection. Ordinary text navigation, input and mode changes SHALL exit structured selection without changing document text merely to select content.

#### Scenario: Select and extend source blocks
- **WHEN** the user enters block selection and extends to an adjacent block
- **THEN** both complete source units are visibly selected, including owned nested descendants
- **AND** no document version or history entry is created

#### Scenario: Stale selection is not applied
- **WHEN** source changes after a structured selection target was captured
- **THEN** its destructive command is rejected or the selection is cleared without guessing ranges

### Requirement: Structured commands use one checked edit transaction
Copy, delete and move of a selected block range SHALL resolve the same complete canonical source units. Successful deletion or movement SHALL be one versioned mutation and one undo entry, preserve unrelated bytes and line endings, and provide an exact resulting text selection. A rejected transaction MUST NOT create an undo entry or change selection.

#### Scenario: Delete several blocks and undo
- **WHEN** a valid multi-block selection is deleted
- **THEN** its complete source units are removed once and one Undo restores exact source and prior selection

### Requirement: Structured copy publishes portable clipboard formats
Copy of selected visual blocks or cells SHALL publish Unicode visible text and equivalent HTML together when supported by the platform. Explicit Markdown copy SHALL retain authored Markdown. A cell rectangle SHALL copy a tab/newline matrix and HTML table. Copy alone MUST NOT mutate source, version, history or cache identity.

#### Scenario: Copy selected visual blocks
- **WHEN** the user copies selected formatted blocks
- **THEN** plain-text consumers receive readable text and HTML consumers receive formatting from the same selected source

#### Scenario: Copy a cell rectangle
- **WHEN** a rectangular cell selection is copied
- **THEN** its selected cells appear in row/column order in TSV and an HTML table

#### Scenario: Windows native clipboard retains both formats
- **WHEN** selected formatted blocks are copied on Windows
- **THEN** the native clipboard contains Unicode text and HTML Format with UTF-8 byte-correct fragment offsets
- **AND** brief clipboard contention is handled with bounded retries and native allocations transfer ownership only on successful publication

#### Scenario: macOS and Linux advertise the rich representation
- **WHEN** a text clipboard item carries HTML on macOS, X11 or Wayland
- **THEN** the existing native transport publishes `public.html` or `text/html` alongside the plain-text representation
- **AND** replacing it with an ordinary text-only copy does not retain stale HTML
