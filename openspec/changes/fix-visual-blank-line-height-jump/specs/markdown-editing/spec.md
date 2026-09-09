## MODIFIED Requirements

### Requirement: Visual Edit whitespace activation
The system SHALL keep source-backed whitespace ranges available for exact caret mapping. In Visual Edit, a `Whitespace` row SHALL behave as a first-class empty line: it occupies the rendered body paragraph line height per covered newline up to the existing pathological bound, participates with adjacent paragraph rows in one same-context body-flow group that contributes paragraph spacing exactly once at its end, presents an I-beam pointer, and accepts pointer placement onto an existing offset inside its source range. Clicking a whitespace row SHALL move the caret into that range and MUST NOT insert a newline or otherwise mutate the document text, version, dirty state, undo history, or derived Markdown caches. When the source caret owns a whitespace row — because the user clicked it, pressed Enter onto a new insertion line, or moved into it with keyboard navigation — Visual Edit SHALL present the same empty-paragraph layout plus a thin insertion caret line visually consistent with the caret in a paragraph or heading, and SHALL accept subsequent typed text at the exact source caret position. Visual Edit SHALL NOT wrap a whitespace row in a source-island box (border, padding, monospace styling, or differentiated background). Source islands SHALL remain reserved for blocks whose source has no rendered visual form (frontmatter, code, HTML, unsupported constructs) or for inline runs whose source/display mapping is ambiguous. Landing offsets SHALL lie inside the whitespace source range. For a single-newline gap between two rendered blocks, the caret SHALL land at `Whitespace.source_range.start` (the authored separator newline), not the first content byte of the following block.

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

#### Scenario: Typing into a blank line preserves following content position
- **WHEN** the user clicks an authored blank-line row in Visual Edit and enters single-line paragraph text
- **THEN** the contiguous paragraph/whitespace body flow has the same occupied vertical extent before and after the edit, including when adjacent paragraph rows merge
- **AND** rendered content below that row does not jump vertically

#### Scenario: Structural Enter activates an insertion line
- **WHEN** the user presses Enter from a heading in Visual Edit and the structural edit creates a new source-backed insertion line
- **THEN** the owning visual row presents the caret and accepts subsequent typed text at the exact source position regardless of whether the parser retains the newline in the heading range

#### Scenario: Intentional source caret movement preserves whitespace editing
- **WHEN** keyboard navigation or reveal logic moves the source caret into an existing whitespace-only range
- **THEN** the owning whitespace row provides the source-backed editing affordance without recomputing the document's cached Markdown-derived state

#### Scenario: Whitespace row owning the caret renders a caret line, not a source island
- **WHEN** the source caret owns a whitespace row in Visual Edit — for example after clicking it, after creating a blank line by pressing Enter, or after pressing Down or Up onto an existing blank line
- **THEN** the row is rendered at stable empty-paragraph height with a thin insertion caret line and no border, padding, monospace styling, or differentiated background
- **AND** typed text is inserted into the canonical Markdown source at the caret position through the same dirty-state, undo/redo, autosave, and per-tab isolation paths as any other edit

#### Scenario: Whitespace row not owning the caret stays an empty line
- **WHEN** a whitespace row does not own the source caret
- **THEN** it still occupies stable empty-paragraph height and remains pointer-editable
- **AND** it does not paint an insertion caret until it owns the caret
