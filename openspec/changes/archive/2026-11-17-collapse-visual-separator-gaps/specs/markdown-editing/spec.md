## MODIFIED Requirements

### Requirement: Visual Edit whitespace activation
The system SHALL keep source-backed whitespace ranges available for exact caret mapping. In Visual Edit, a `Whitespace` row SHALL remain that source range's hit target and SHALL NOT be wrapped in a source-island box (border, monospace styling, or differentiated background). Source islands SHALL remain reserved for blocks whose source has no rendered visual form or for inline runs whose source/display mapping is ambiguous.

A whitespace row that lies strictly between two other blocks and contains at least one newline SHALL treat its first newline as the structural block separator: that separator occupies the rendered paragraph spacing and MUST NOT add a body-height blank line. Each further newline in the same row SHALL occupy one body paragraph line, capped at the existing pathological bound. A leading whitespace row, and a whitespace row that reaches the document end, SHALL occupy one body paragraph line per newline in the row and SHALL NOT add a further phantom line for the caret sitting at the document end. An empty document-end insertion row (the row that exists so the caret can sit after a block that already consumed its own line ending) SHALL NOT occupy a body-height blank line.

The row SHALL present an I-beam pointer and accept pointer placement onto an existing offset inside its source range, including a structural separator whose only extent is paragraph spacing. Clicking a whitespace row SHALL move the caret into that range and MUST NOT insert a newline or otherwise mutate the document text, version, dirty state, undo history, or derived Markdown caches. When the source caret owns a whitespace row, Visual Edit SHALL paint a thin insertion caret in that row and SHALL accept subsequent typed text at the exact source caret position. Typing into a structural separator SHALL insert at that existing newline; following content MAY move down by the height of the new line. Landing offsets SHALL lie inside the whitespace source range. For a single-newline gap between two rendered blocks, the caret SHALL land at `Whitespace.source_range.start` (the authored separator newline), not the first content byte of the following block.

#### Scenario: Clicking a blank line between headings places the caret without mutation
- **WHEN** the Visual Edit caret belongs to a rendered heading and the user clicks the blank-line `Whitespace` row between that heading and another heading
- **THEN** the caret moves onto an existing offset inside that whitespace range (`source_range.start` for a single-newline gap)
- **AND** the document text, version, dirty state, undo history, and derived Markdown cache identity remain unchanged
- **AND** the gap row presents an insertion caret

#### Scenario: Clicking a blank line between a heading and a paragraph places the caret without mutation
- **WHEN** the Visual Edit caret belongs to a rendered block and the user clicks the blank-line `Whitespace` row between a heading and a paragraph
- **THEN** the caret moves onto an existing offset inside that whitespace range
- **AND** the document text, version, dirty state, undo history, and derived Markdown cache identity remain unchanged
- **AND** the gap row becomes the caret-owning typing surface

#### Scenario: Typing after a gap click inserts at the existing newline
- **WHEN** the user clicks the blank-line row between `## [Unreleased]` and `## [16.1.7]` in a changelog-like document and types text
- **THEN** the typed bytes insert at the existing separator newline so a paragraph appears between the two headings
- **AND** the following heading’s first content byte is not consumed
- **AND** the edit does not insert an extra blank line beyond the newline that was already authored

#### Scenario: Structural Enter activates an insertion line
- **WHEN** the user presses Enter from a heading in Visual Edit and the structural edit creates a new source-backed insertion line
- **THEN** the owning visual row presents the caret and accepts subsequent typed text at the exact source position regardless of whether the parser retains the newline in the heading range

#### Scenario: Intentional source caret movement preserves whitespace editing
- **WHEN** keyboard navigation or reveal logic moves the source caret into an existing whitespace-only range
- **THEN** the owning whitespace row provides the source-backed editing affordance without recomputing the document's cached Markdown-derived state

#### Scenario: Whitespace row owning the caret renders a caret line, not a source island
- **WHEN** the source caret owns a whitespace row in Visual Edit — for example after clicking it, after creating a blank line by pressing Enter, or after pressing Down or Up onto an existing blank line
- **THEN** the row paints a thin insertion caret and no border, monospace styling, or differentiated background
- **AND** a structural separator keeps paragraph-spacing extent while the caret owns it
- **AND** typed text is inserted into the canonical Markdown source at the caret position through the same dirty-state, undo/redo, autosave, and per-tab isolation paths as any other edit

#### Scenario: Whitespace row not owning the caret stays an empty line
- **WHEN** a whitespace row does not own the source caret
- **THEN** a structural separator still occupies paragraph spacing and every additional authored blank line still occupies body line height
- **AND** the row remains pointer-editable
- **AND** it does not paint an insertion caret until it owns the caret

#### Scenario: A single inter-block newline is paragraph spacing
- **WHEN** Visual Edit displays two blocks separated by exactly one newline that belongs to a whitespace row
- **THEN** that row occupies the rendered paragraph spacing and does not add a body-height blank line
- **AND** increasing paragraph spacing increases the row's occupied extent
- **AND** the source text is unchanged

#### Scenario: Newlines beyond the separator stay body lines
- **WHEN** a whitespace row between two blocks contains more than one newline
- **THEN** the first newline occupies paragraph spacing
- **AND** each remaining newline occupies one body paragraph line

#### Scenario: A normal trailing newline does not paint a blank line
- **WHEN** a block ends with its own line ending at the document end, so the only following whitespace row has an empty source range
- **THEN** Visual Edit does not paint a body-height blank line for that row
- **AND** the caret can still be placed at the document end
- **AND** an additional authored newline after that block occupies one body paragraph line

#### Scenario: Typing into a structural separator moves following content
- **WHEN** the user types a line of text into a single-newline gap between two blocks
- **THEN** the new line occupies body line height
- **AND** rendered content below that gap moves down

### Requirement: Visual Edit renders unquoted list trailing blank lines as unindented whitespace rows
When an unquoted list item is followed by authored blank lines (at document end, before following content, or between loose-list items), Visual Edit SHALL present those blank lines as full-width `Whitespace` rows at the document's left margin — the same presentation blank lines already use between paragraphs and after quoted lists — instead of painting them inside the indented list-item row. The single newline that only separates the item from the next block SHALL occupy paragraph spacing and MUST NOT add a body-height blank line. Each further blank line SHALL occupy one body paragraph line. Each of those newlines SHALL keep its own source-backed, individually addressable position, and a caret on any of those rows SHALL paint at the row's left edge, aligned with the first content column of the surrounding document flow rather than with the list item's marker or text column. The caret at end-of-file after such a list SHALL keep its own empty insertion row when the last list line ends with a newline, and that empty row SHALL NOT add a body-height blank line.

#### Scenario: Caret on a blank line below a nested list sits at the left margin
- **WHEN** the document contains a nested list whose innermost item is followed by several blank lines and the user places the caret on any of those blank lines
- **THEN** the caret is painted at the blank row's left edge, not indented to the inner list level nor aligned with the item's text column
- **AND** clicking the row, typing, and Undo keep using that line's canonical source offset

#### Scenario: Loose-list separators and mid-document tails use the same geometry
- **WHEN** a single blank line separates two items of a loose list or follows a list before subsequent content
- **THEN** that newline renders as an unindented whitespace row that occupies paragraph spacing and does not add a body-height blank line
- **AND** the row is not duplicated before the next block
- **AND** each blank line beyond that separator occupies one body paragraph line
