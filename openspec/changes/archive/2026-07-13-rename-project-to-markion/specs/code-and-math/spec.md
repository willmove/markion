## MODIFIED Requirements

### Requirement: Fenced code block highlighting
The editor SHALL apply grammar-based (syntect) syntax coloring to code blocks whose language identifier is covered by the bundled extended grammar registry (syntect defaults plus the two-face extended syntax set — including TypeScript, TOML, Kotlin, Swift, Dockerfile, PowerShell and other modern mainstream languages), falling back to the hand-written token-class lexer for identifiers the registry does not cover. The advertised language list SHALL be the union of the registry's actual grammar names and the lexer-fallback identifiers, so the advertisement reflects real coverage. Colors SHALL continue to be derived from Markion's theme-mapped token classes (`HighlightKind`), never from a fixed syntect color theme. Grammar loading SHALL remain lazy with a background warm-up at startup, and highlighting results SHALL remain memoized per language/code pair.

#### Scenario: Extended-set language is highlighted by syntect
- **WHEN** a fenced code block carries a modern mainstream identifier from the extended set (e.g. `typescript`, `toml`, `dockerfile`)
- **THEN** its content is colored by token classes derived from syntect scopes rather than the legacy lexer

#### Scenario: Grammar-covered language is highlighted by syntect
- **WHEN** a fenced code block carries a language identifier covered by the grammar registry (directly or via alias)
- **THEN** its content is colored by token classes derived from syntect scopes, including multi-line constructs such as block comments and multi-line strings

#### Scenario: Registry-uncovered language falls back to the lexer
- **WHEN** a fenced code block carries a language identifier that the grammar registry does not cover
- **THEN** the hand-written token-class lexer colors it exactly as before

#### Scenario: Advertised list reflects real coverage
- **WHEN** the supported-language list is queried
- **THEN** it contains every registry grammar name (lowercased) and every lexer-fallback identifier, deduplicated and sorted

#### Scenario: Unspecified language yields plain text
- **WHEN** a fenced code block has no language identifier
- **THEN** the block renders as plain monospaced text with no syntax coloring

#### Scenario: Original indentation is preserved
- **WHEN** a code block contains leading whitespace or blank lines
- **THEN** the original indentation and whitespace are preserved in the rendered output
