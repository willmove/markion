## ADDED Requirements

### Requirement: Terminal Visual Edit newlines own a blank caret row

When an unquoted paragraph or heading reaches the document tail, Visual Edit SHALL represent its terminal line ending and any following whitespace as a source-backed `Whitespace` row. The first terminal Enter SHALL therefore place the source caret on a real blank visual row immediately, and further terminal Enter presses SHALL grow that same tail row instead of alternating between a hidden paragraph marker and a whitespace row. This derived ownership SHALL preserve the canonical Markdown bytes and contiguous, non-overlapping visual source coverage.

#### Scenario: First Enter after terminal prose creates the blank row
- **WHEN** the user places the Visual Edit caret after terminal paragraph text and presses Enter once
- **THEN** the inserted line ending is owned by a terminal `Whitespace` row whose range reaches the document end
- **AND** the caret paints at the blank row rather than at the end of the preceding text

#### Scenario: Consecutive terminal Enter presses keep one row model
- **WHEN** the user presses Enter repeatedly at the document tail
- **THEN** every press extends the source-backed terminal whitespace range by one authored line ending
- **AND** Visual Edit does not alternate between paragraph-marker and whitespace-row caret layouts

#### Scenario: Terminal line-ending ownership preserves source fidelity
- **WHEN** terminal prose uses LF or CRLF line endings
- **THEN** the paragraph or heading and terminal whitespace blocks provide contiguous, non-overlapping coverage through the document end
- **AND** the canonical source bytes are unchanged by the visual partition
