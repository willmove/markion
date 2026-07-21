## MODIFIED Requirements

### Requirement: Visual Edit whitespace activation
The system SHALL keep source-backed whitespace ranges available for exact caret mapping while treating whitespace between rendered blocks as passive layout until the source caret intentionally enters that range. When the source caret owns a whitespace row — whether because the user pressed Enter at the end of a paragraph (whose source range excludes the trailing newline) or because keyboard navigation moved the caret into a whitespace-only range — Visual Edit SHALL present the row as the same passive-height layout it uses when unfocused, plus a thin insertion caret line visually consistent with the caret in a paragraph or heading, and SHALL accept subsequent typed text at the exact source caret position. Visual Edit SHALL NOT wrap a whitespace row that owns the caret in a source-island box (border, padding, monospace styling, or differentiated background), because such chrome misrepresents ordinary inter-paragraph spacing as a code-like block. Source islands SHALL remain reserved for blocks whose source has no rendered visual form (frontmatter, code, HTML, unsupported constructs) or for inline runs whose source/display mapping is ambiguous and therefore requires a conservative source-editing fallback.

#### Scenario: Clicking a passive gap between headings does not activate editing
- **WHEN** the Visual Edit caret belongs to a rendered heading and the user clicks the whitespace gap between that heading and another heading
- **THEN** the source selection and document content remain unchanged and the gap does not present an insertion caret

#### Scenario: Clicking a passive gap before a paragraph does not activate editing
- **WHEN** the Visual Edit caret belongs to a rendered block and the user clicks the whitespace gap between a heading and a paragraph
- **THEN** the source selection and document content remain unchanged and the gap does not become an editable typing area

#### Scenario: Structural Enter activates an insertion line
- **WHEN** the user presses Enter from a heading in Visual Edit and the structural edit creates a new source-backed insertion line
- **THEN** the owning visual row presents the caret and accepts subsequent typed text at the exact source position regardless of whether the parser retains the newline in the heading range

#### Scenario: Intentional source caret movement preserves whitespace editing
- **WHEN** keyboard navigation or reveal logic moves the source caret into an existing whitespace-only range
- **THEN** the owning whitespace row provides the source-backed editing affordance without recomputing the document's cached Markdown-derived state

#### Scenario: Whitespace row owning the caret renders a caret line, not a source island
- **WHEN** the source caret owns a whitespace row in Visual Edit — for example after creating a blank line by pressing Enter (so a second newline lands outside any paragraph range), or after pressing Down arrow across an existing blank line
- **THEN** the row is rendered as passive-height layout with a thin insertion caret line and no border, padding, monospace styling, or differentiated background
- **AND** typed text is inserted into the canonical Markdown source at the caret position through the same dirty-state, undo/redo, autosave, and per-tab isolation paths as any other edit

#### Scenario: Whitespace row not owning the caret remains passive
- **WHEN** a whitespace row does not own the source caret
- **THEN** it renders as passive layout without a caret, exactly as before, regardless of whether it owns the caret on other frames
