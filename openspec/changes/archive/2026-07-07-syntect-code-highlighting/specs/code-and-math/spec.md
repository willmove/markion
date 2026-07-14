## MODIFIED Requirements

### Requirement: Fenced code block highlighting
The editor SHALL advertise support for a broad set of fenced code block language identifiers (50+) and SHALL apply grammar-based (syntect) syntax coloring to code blocks whose language identifier is covered by the bundled grammar registry, falling back to the hand-written token-class lexer for identifiers the registry does not cover. Colors SHALL continue to be derived from Markion's theme-mapped token classes (`HighlightKind`), never from a fixed syntect color theme, so all application themes color code consistently. Grammar loading SHALL be lazy with a background warm-up at startup so the first highlight never blocks the typing path, and highlighting results SHALL remain memoized per language/code pair.

#### Scenario: Grammar-covered language is highlighted by syntect
- **WHEN** a fenced code block carries a language identifier covered by the grammar registry (directly or via alias)
- **THEN** its content is colored by token classes derived from syntect scopes, including multi-line constructs such as block comments and multi-line strings

#### Scenario: Registry-uncovered language falls back to the lexer
- **WHEN** a fenced code block carries a recognized language identifier that the grammar registry does not cover
- **THEN** the hand-written token-class lexer colors it exactly as before

#### Scenario: Unspecified language yields plain text
- **WHEN** a fenced code block has no language identifier
- **THEN** the block renders as plain monospaced text with no syntax coloring

#### Scenario: Original indentation is preserved
- **WHEN** a code block contains leading whitespace or blank lines
- **THEN** the original indentation and whitespace are preserved in the rendered output
