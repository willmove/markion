## ADDED Requirements

### Requirement: Visual Edit whitespace activation
The system SHALL keep source-backed whitespace ranges available for exact caret mapping while treating whitespace between rendered blocks as passive layout until the source caret intentionally enters that range.

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
