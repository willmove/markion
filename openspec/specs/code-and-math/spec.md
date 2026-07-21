# code-and-math

## Purpose

Covers fenced code block handling, syntax highlighting, line-number rendering, and Markdown math (`$...$` inline and `$$...$$` block). Real KaTeX/MathJax-quality math rendering is **not** part of this capability — math is parsed into structured data with a readable Unicode fallback and simple LaTeX validation; full-quality rendering is a future candidate.
## Requirements
### Requirement: Fenced code block highlighting
The editor SHALL first compare the first fenced info-string token against the aliases in the registered diagram backend registry using ASCII case-insensitive matching. A matching block SHALL follow the diagram-rendering capability instead of ordinary syntax highlighting while retaining its authored code and source range; when diagram rendering is pending or fails, its source fallback SHALL preserve the original indentation and whitespace. All other fenced code blocks SHALL apply grammar-based (syntect) syntax coloring when their language identifier is covered by the bundled extended grammar registry (syntect defaults plus the two-face extended syntax set — including TypeScript, TOML, Kotlin, Swift, Dockerfile, PowerShell and other modern mainstream languages), falling back to the hand-written token-class lexer for identifiers the registry does not cover. The advertised code-language list SHALL be the union of the syntax registry's actual grammar names and the lexer-fallback identifiers, so the advertisement reflects real code-highlighting coverage and does not imply diagram-backend compatibility. Colors SHALL continue to be derived from Markion's theme-mapped token classes (`HighlightKind`), never from a fixed syntect color theme. Grammar loading SHALL remain lazy with a background warm-up at startup, and highlighting results SHALL remain memoized per language/code pair.

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
- **WHEN** a fenced code block carries a language identifier that neither the diagram registry nor the syntax grammar registry covers
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

### Requirement: Optional code block line numbers
The editor SHALL provide a preference (persisted) that toggles line-number rendering for fenced code blocks in the preview.

#### Scenario: Line numbers toggle and persist
- **WHEN** the user toggles the code-line-numbers preference
- **THEN** fenced code blocks in the preview show or hide a numbered gutter, and the choice persists across launches

### Requirement: Markdown math parsing with validation and readable fallback
The parser SHALL recognize `$...$` inline math, `$$...$$` display math, and fenced blocks whose first info-string token is `math`, and SHALL extract the delimiter-free payload, delimiter kind, text/display style, and byte-exact authored source range as structured data. For valid expressions, Split Preview, Read, and unfocused Visual Edit SHALL render offline, high-quality typesetting with a KaTeX-compatible core and common AMS-style constructs, using text style and a measured surrounding-text baseline for inline math and display style for centered block math. Rendering SHALL require no browser, JavaScript runtime, network resource, external font, or TeX installation. Invalid or unsupported expressions SHALL show a localized, bounded fallback containing the exact authored source and useful error position when available. Built-in DOCX export SHALL retain its readable Unicode/source fallback in this change.

#### Scenario: Inline and display math retain semantic source data
- **WHEN** a document contains `$...$` or `$$...$$`
- **THEN** the parser emits a semantic math node with delimiter-free payload, delimiter kind, math style, and an exact source range
- **AND** inline math is not flattened into an ordinary text run before layout

#### Scenario: Math fence becomes display math
- **WHEN** a fenced block's first info-string token is `math`
- **THEN** its payload is parsed and rendered as display math
- **AND** its complete authored fence and payload remain available for copy, editing, and fallback

#### Scenario: Common mathematical notation is typeset faithfully
- **WHEN** a valid expression uses fractions, roots, super/subscripts, sums or integrals, matrices, cases or aligned rows, stretchy delimiters, Greek or styled alphabets, arrows, accents, or `\text{...}` Unicode text supported by the renderer's font set
- **THEN** the preview uses structured glyph layout rather than Unicode character substitution or raw-LaTeX display

#### Scenario: Inline and display styles differ correctly
- **WHEN** the same operator expression appears once inline and once as display math
- **THEN** the inline expression uses text-style limits and baseline metrics
- **AND** the display expression uses display-style limits and block layout

#### Scenario: Formula follows presentation inputs
- **WHEN** the text foreground, application theme, zoom, or display scale changes
- **THEN** the formula is rendered transparently with the resolved foreground and appropriate logical/raster scale
- **AND** glyphs remain crisp without persisting presentation data into the document

#### Scenario: Wide display math remains accessible
- **WHEN** a display expression is wider than its preview container
- **THEN** the complete formula is available through bounded horizontal overflow rather than clipping or shrinking to illegibility

#### Scenario: Invalid expression falls back locally
- **WHEN** the renderer rejects an empty, unbalanced, invalid, or unsupported expression
- **THEN** only that expression shows a localized bounded error fallback containing the exact authored source and an error position when available
- **AND** the preview and document remain usable without a crash or stale formula image

#### Scenario: Built-in DOCX behavior is unchanged
- **WHEN** a document containing math is exported through the built-in DOCX writer
- **THEN** the existing readable Unicode/source fallback is used rather than claiming native high-quality typesetting

### Requirement: Direct fenced-code editing in Visual Edit
Visual Edit SHALL present an ordinary, exactly ranged fenced code block as a syntax-highlighted direct code editor whose editable text is the authored code payload. Payload editing SHALL preserve the authored opening fence, closing fence, fence length, info-string spacing, language token, indentation, blank lines, and final-newline semantics outside the replaced payload. Registered diagram fences, unclosed fences, and ambiguous fenced constructs MUST retain their source-backed island behavior.

#### Scenario: Ordinary code payload is edited without exposing fences
- **WHEN** the user focuses and edits the payload of an exactly ranged ordinary fenced code block in Visual Edit
- **THEN** the code remains presented as a code editor with the appropriate memoized syntax highlighting
- **AND** one exact source replacement updates only the authored payload range
- **AND** the opening fence, info string, and closing fence remain byte-identical

#### Scenario: Code editor supports platform text input and IME
- **WHEN** normal input or an IME composition replaces a selection in the direct code editor
- **THEN** the canonical source selection and UTF-8-safe replacement path are used
- **AND** the composition is integrated with the existing candidate geometry and semantic undo contracts

#### Scenario: Registered diagram remains conservative
- **WHEN** a fenced info string resolves to a registered diagram backend
- **THEN** Visual Edit uses the existing complete source-backed fence island
- **AND** it does not attach the ordinary code direct editor

#### Scenario: Ambiguous fence falls back without loss
- **WHEN** the fence is unclosed or exact payload and delimiter boundaries cannot be proven
- **THEN** Visual Edit exposes the complete authored source island
- **AND** no delimiter or whitespace is synthesized or removed

### Requirement: Direct block-math editing in Visual Edit
Visual Edit SHALL present an exactly ranged block-math construct as a rendered formula together with a direct monospaced LaTeX editor for its authored payload. Editing the payload SHALL preserve its complete delimiters and unrelated whitespace, SHALL remain available while rendering is pending or invalid, and SHALL use the same validation, source selection, IME, and undo paths as other text input.

#### Scenario: Formula and LaTeX editor remain visible together
- **WHEN** an exactly ranged block-math construct appears in Visual Edit
- **THEN** its rendered formula or readable error state is shown together with the editable LaTeX payload
- **AND** focusing the payload does not replace the whole block with raw delimiter source

#### Scenario: Invalid LaTeX remains directly editable
- **WHEN** a payload edit makes the formula invalid or its render cache reports an error
- **THEN** the editor shows the validation or render error without discarding the payload editor
- **AND** the authored source remains available for correction

#### Scenario: Math payload edit is atomic and lossless
- **WHEN** the user replaces text within the LaTeX payload
- **THEN** one canonical source replacement changes only that payload range
- **AND** one Undo restores the prior formula source, selection, and delimiters

### Requirement: Math rendering is asynchronous, bounded, and reusable
Native formula typesetting and rasterization SHALL run outside the synchronous typing and Markdown-derivation path. The editor SHALL keep a bounded presentation cache whose identity includes the LaTeX payload, text/display style, logical font size, resolved foreground, effective zoom/display scale, and renderer/font-set version. Identical pending requests SHALL coalesce, ready and error results SHALL be reused across document versions, and late completions SHALL only update a still-matching cache entry and request repaint without mutating document state or derived Markdown caches.

#### Scenario: Typing does not synchronously typeset formulas
- **WHEN** an edit produces a new or changed math expression
- **THEN** Markdown-derived state records the semantic expression without waiting for typesetting or rasterization
- **AND** rendering is scheduled outside the keystroke path

#### Scenario: Unchanged formulas reuse cached output
- **WHEN** unrelated document text changes while a formula's complete presentation key remains identical
- **THEN** preview reuses the existing ready or error cache entry rather than typesetting it again

#### Scenario: Concurrent requests coalesce
- **WHEN** multiple visible consumers request the same uncached formula key while it is pending
- **THEN** at most one render job is scheduled for that key

#### Scenario: Late completion cannot corrupt current state
- **WHEN** a render job completes after its source, style, color, scale, tab, or view has changed
- **THEN** the result is committed only if the exact key is still pending
- **AND** document text, document version, derived `Arc` caches, undo history, and preview list state are unchanged

#### Scenario: Cache and raster allocation are bounded
- **WHEN** a document contains many distinct or pathologically large expressions
- **THEN** completed entries and output dimensions follow configured bounds with deterministic eviction or a localized fallback
- **AND** pending work does not cause unbounded cache growth or raster allocation

