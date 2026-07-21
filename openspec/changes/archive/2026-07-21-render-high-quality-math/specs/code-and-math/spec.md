## MODIFIED Requirements

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

## ADDED Requirements

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
