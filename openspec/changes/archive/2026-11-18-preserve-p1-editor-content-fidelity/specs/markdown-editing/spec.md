## ADDED Requirements

### Requirement: List subtree operations SHALL preserve authored ownership
The editor SHALL use the complete parser-owned list item for structural moves and indentation, including paragraph content following nested items. Indentation SHALL change one level and preserve the subtree hierarchy or leave the source unchanged when the change cannot be proven safe.

#### Scenario: Reindent a complete subtree
- **WHEN** a list item is reindented and owns child items followed by a continuation paragraph
- **THEN** both the child items and paragraph keep their original owner

#### Scenario: Move a list item before independent prose
- **WHEN** sibling subtrees exchange places before an independent paragraph
- **THEN** the paragraph remains outside the list and a reverse move leaves it in place

### Requirement: Imported and escaped prose SHALL retain its literal meaning
Rich paste and document import SHALL preserve literal text that resembles Markdown syntax. Authored escaping SHALL protect Markion's inline extensions throughout preview and export. Real source styles and list structure SHALL remain available.

#### Scenario: Adjacent inline text forms a list lookalike
- **WHEN** pasted HTML supplies an ordered marker across adjacent inline nodes
- **THEN** the pasted text remains an ordinary paragraph with the complete marker visible

#### Scenario: Literal inline notation
- **WHEN** imported text contains literal entity, superscript, or strike notation
- **THEN** the preview and exported HTML display the original characters
