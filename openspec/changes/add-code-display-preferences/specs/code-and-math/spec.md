## ADDED Requirements

### Requirement: Code display theme
The editor SHALL provide a persisted Light/Dark code display theme governing fenced code-block presentation — token colors, block background, default text color, line-number gutter, language label, and copy affordance — across Read mode, Split Preview's rendered pane, and the Visual Edit direct code editor. The default SHALL be Dark, matching the pre-change appearance. Switching the code display theme SHALL apply immediately on the next render, SHALL NOT alter document content or derived document state, and SHALL NOT require re-tokenizing code, because token-class highlighting is memoized independently of color mapping. Both palettes SHALL keep every token class readable against their respective block backgrounds.

#### Scenario: Default is Dark
- **WHEN** the editor starts with default preferences or loads a config written before this change
- **THEN** fenced code blocks render with the dark chrome and dark token palette identical to the pre-change appearance

#### Scenario: Light applies across every code surface
- **WHEN** the user selects the Light code display theme
- **THEN** fenced code blocks in Read mode, Split Preview's rendered pane, and the Visual Edit direct code editor repaint with light chrome (background, text, gutter, language label, copy affordance) and the light token palette on the next render

#### Scenario: Switching is presentation-only
- **WHEN** the user switches between Light and Dark
- **THEN** document text, dirty state, undo history, selections, and derived Markdown cache identity are unchanged
- **AND** visible code blocks repaint without re-tokenizing their content

### Requirement: Code block long-line wrapping
The editor SHALL provide a persisted wrap preference for fenced code blocks on reading surfaces (Read mode and Split Preview's rendered pane). When wrapping is enabled (the default), long code lines soft-wrap within the block. When wrapping is disabled, code lines SHALL NOT soft-wrap; the complete content SHALL remain reachable through bounded horizontal scrolling of the code block, and the line-number gutter, when visible, SHALL keep exactly one number per logical line. The Visual Edit direct code editor SHALL continue to soft-wrap regardless of this preference. Changing the preference SHALL be presentation-only and SHALL apply on the next render.

#### Scenario: Default keeps soft wrapping
- **WHEN** a code block contains a line longer than the block width and wrapping is on (default)
- **THEN** the line soft-wraps within the block and no horizontal scrolling is offered

#### Scenario: Disabling wrap enables horizontal scrolling
- **WHEN** the user turns wrap off and a code block contains a line longer than the block width
- **THEN** the line renders unwrapped and the block's full content is reachable by scrolling the block horizontally
- **AND** the line-number gutter, when visible, numbers logical lines without duplication or omission

#### Scenario: Visual Edit direct editor keeps wrapping
- **WHEN** wrapping is disabled and a long code line is edited in the Visual Edit direct code editor
- **THEN** that editor still soft-wraps the line and caret, selection, and IME geometry remain unchanged from before the preference change

#### Scenario: The wrap preference persists
- **WHEN** the user toggles the wrap preference and restarts the editor
- **THEN** reading-surface code blocks render with the persisted wrapping behavior

## MODIFIED Requirements

### Requirement: Fenced code block highlighting
The editor SHALL first compare the first fenced info-string token against the aliases in the registered diagram backend registry using ASCII case-insensitive matching. A matching block SHALL follow the diagram-rendering capability instead of ordinary syntax highlighting while retaining its authored code and source range; when diagram rendering is pending or fails, its source fallback SHALL preserve the original indentation and whitespace. All other fenced code blocks SHALL apply grammar-based (syntect) syntax coloring when their language identifier is covered by the bundled extended grammar registry (syntect defaults plus the two-face extended syntax set — including TypeScript, TOML, Kotlin, Swift, Dockerfile, PowerShell and other modern mainstream languages), falling back to the hand-written token-class lexer for identifiers the registry does not cover. The advertised code-language list SHALL be the union of the syntax registry's actual grammar names and the lexer-fallback identifiers, so the advertisement reflects real code-highlighting coverage and does not imply diagram-backend compatibility. Colors SHALL continue to be derived from Markion's theme-mapped token classes (`HighlightKind`), never from a fixed syntect color theme; the on-screen color for each token class SHALL come from the Light or Dark palette of the active code display theme (see the Code display theme requirement). Grammar loading SHALL remain lazy with a background warm-up at startup, and highlighting results SHALL remain memoized per language/code pair.

#### Scenario: Registered diagram fence bypasses code highlighting
- **WHEN** a fenced code block's first info-string token matches a registered diagram backend alias such as `mermaid`
- **THEN** the block is dispatched to diagram rendering rather than syntect or the hand-written lexer

#### Scenario: Diagram source fallback preserves whitespace
- **WHEN** registered diagram rendering is pending or fails for a block containing leading whitespace or blank lines
- **THEN** the source fallback preserves the authored indentation and whitespace

#### Scenario: Extended-set language is highlighted by syntect
- **WHEN** a non-diagram fenced code block carries a modern mainstream identifier from the extended set (e.g. `typescript`, `toml`, `dockerfile`)
- **THEN** its content is colored by token classes derived from syntect scopes rather than the legacy lexer

#### Scenario: Grammar-covered language is highlighted by syntect
- **WHEN** a non-diagram fenced code block carries a language identifier covered by the grammar registry (directly or via alias)
- **THEN** its content is colored by token classes derived from syntect scopes, including multi-line constructs such as block comments and multi-line strings

#### Scenario: Registry-uncovered language falls back to the lexer
- **WHEN** a non-diagram fenced code block carries a language identifier that neither the diagram registry nor the syntax grammar registry covers
- **THEN** the hand-written token-class lexer colors it exactly as before

#### Scenario: Advertised list reflects real coverage
- **WHEN** the supported code-language list is queried
- **THEN** it contains every syntax registry grammar name (lowercased) and every lexer-fallback identifier, deduplicated and sorted

#### Scenario: Unspecified language yields plain text
- **WHEN** a fenced code block has no language identifier
- **THEN** the block renders as plain monospaced text with no syntax coloring

#### Scenario: Original indentation is preserved
- **WHEN** an ordinary code block contains leading whitespace or blank lines
- **THEN** the original indentation and whitespace are preserved in the rendered output
