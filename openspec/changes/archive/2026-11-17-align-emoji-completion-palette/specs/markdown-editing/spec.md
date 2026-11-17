## MODIFIED Requirements

### Requirement: Colon emoji shortcode completion
When the user types `:` at a word boundary (start of line, whitespace, or opening punctuation) on the source editor or Visual Edit prose, outside fenced-code / math / diagram payloads and outside an IME composition, the editor SHALL open a localized shortcode palette filtered by the ASCII prefix after that colon against the maintained emoji shortcode table. Confirming a row (Enter, Tab, or pointer) SHALL replace the open `:query` with the canonical `:name:` (including the closing colon) as one undoable source edit; the preview and unfocused Visual Edit SHALL render the matching emoji glyph through the existing shortcode parser. Escape or moving the caret so the token is no longer an open shortcode SHALL dismiss the palette without writing. Opening, filtering, moving the highlight, and dismissing SHALL NOT increment document version. If a slash-command palette is already open for a `/` query, that palette SHALL take priority and this completer SHALL NOT open.

The palette SHALL appear beside the current caret with the same below-source offset, compact width, theme colors, border, rounding, shadow, padding, and button spacing as slash completion, adjusted to fit the viewport. Emoji candidates SHALL use compact glyph buttons with shortcode tooltips and an active-shortcode caption. Source-editor completion SHALL follow its own caret geometry rather than stale Visual Edit bounds.

The unfiltered emoji palette SHALL show six rows of six candidates (36 per visible window), increasing its overall height to approximately twice the initial two-row popup while retaining its width and button dimensions. Filtered results SHALL use only the rows needed for their matching candidates. Keyboard and pointer-wheel navigation SHALL advance through all matching results using the larger candidate window.

Recognized line-leading `:name:` tokens SHALL take precedence over definition-list markers, including consecutive emoji lines and mixed documents containing genuine definitions. Visual Edit, preview and HTML export SHALL show the same glyph without consuming the token's opening colon. A caret remaining at the confirmed token boundary or moving elsewhere within its block SHALL NOT reveal another emoji merely because it shares that block. Explicit caret entry into a token SHALL retain the configured progressive-inline reveal behavior. Completion SHALL support the existing Markdown library's broader emoji catalog as well as all legacy application aliases and glyph choices. Every filtered candidate SHALL be reachable while preserving the compact popup dimensions.

#### Scenario: Colon at a word boundary opens the palette
- **WHEN** the user types `:smi` in Visual Edit prose after a space
- **THEN** a palette lists shortcodes whose names start with `smi` (for example `smile`)
- **AND** document version is unchanged until a row is confirmed

#### Scenario: Confirm inserts the shortcode
- **WHEN** the palette is open on `smile` and the user confirms
- **THEN** the source at that token is `:smile:`
- **AND** unfocused Visual Edit and preview render the smile emoji
- **AND** one Undo restores the text from before the confirmation

#### Scenario: Escape dismisses without writing
- **WHEN** the emoji palette is open and the user presses Escape
- **THEN** the palette closes
- **AND** the authored `:query` remains as typed
- **AND** document version is unchanged by the dismiss

#### Scenario: Time and URLs do not open the palette
- **WHEN** the user types `12:30` or `https://example.com`
- **THEN** the emoji palette does not open

#### Scenario: Slash commands win on a slash query
- **WHEN** the slash-command palette is open because the line starts with `/`
- **THEN** typing `:` inside that query does not open the emoji palette

#### Scenario: Block-start emoji completion matches slash placement
- **WHEN** the user types `:` at a Visual Edit block start
- **THEN** the emoji palette opens beside the caret using slash completion's placement and compact styling
- **AND** all displayed candidate buttons remain inside the popup and usable viewport

#### Scenario: Shortcodes remain discoverable in the compact palette
- **WHEN** the user hovers a glyph or navigates results with the keyboard
- **THEN** the highlighted result and its shortcode caption agree
- **AND** the hovered glyph has a shortcode tooltip
- **AND** source, document version, undo history, and derived-cache identity are unchanged

#### Scenario: Source completion follows the source caret
- **WHEN** the user types an eligible colon query in the source editor, including after scrolling or switching from Visual Edit
- **THEN** the compact palette opens beside the source caret within the viewport
- **AND** confirmation and Undo preserve the existing shortcode replacement semantics

#### Scenario: Consecutive emoji lines retain their opening colons
- **WHEN** a block contains line-leading `:bug:`, `:+1:`, `:book:`, `:-1:`, and `:check:` after another emoji or prose line
- **THEN** Visual Edit and preview render the corresponding glyphs while the caret remains in that block
- **AND** HTML export renders those glyphs and canonical source retains every complete shortcode
- **AND** an actual `: definition` line still renders as a definition

#### Scenario: More emoji remain reachable in a compact popup
- **WHEN** the user filters for a newly supported name or navigates beyond the initial 36 candidates
- **THEN** the matching candidates can be selected and confirmed through the compact palette
- **AND** completion, Visual Edit and preview use the same glyph for that name

### Requirement: Definition lists SHALL render in preview, Read, and Visual Edit
The Markdown parser SHALL recognize definition lists: a term line followed by one or more definitions, each opened by a `:` marker at the start of a line. Split Preview and Read mode SHALL render each term in bold and each definition indented below it. Inline formatting inside terms and definitions SHALL render as in paragraphs. A definition that spans several paragraphs SHALL render as one definition with a line break between its paragraphs. A non-paragraph block nested in a definition (for example a list or code block) SHALL end that definition's own text and render as an ordinary block after it. Visual Edit SHALL render terms and definitions the same way, with the `:` marker hidden as a block prefix that is revealed when the caret enters the definition. Terms and definitions SHALL edit as ordinary source-backed inline text with exact caret mapping and one source mutation per edit. Definition lists inside a blockquote SHALL render inside the quote. Every source byte of a definition list SHALL have exactly one visual owner.

A complete, recognized emoji shortcode at a potential definition marker position SHALL remain inline emoji content, including consecutive shortcode lines and indented or quoted contexts. An explicit definition marker followed by whitespace and an emoji token SHALL remain a definition containing that emoji.

#### Scenario: Term and definition render in preview
- **WHEN** a document contains `Apple` followed by `: Red fruit` and is shown in Split Preview or Read mode
- **THEN** "Apple" renders in bold and "Red fruit" renders indented below it
- **AND** the `:` marker is not shown

#### Scenario: Several definitions and terms
- **WHEN** a term has two `:` definitions and is followed by a second term with its own definition
- **THEN** each term renders once and each definition renders as its own indented row below its term

#### Scenario: Loose multi-paragraph definition
- **WHEN** a definition is followed by an indented continuation paragraph
- **THEN** both paragraphs render inside the same definition, separated by a line break

#### Scenario: Visual Edit hides and reveals the marker
- **WHEN** a definition list is shown in Visual Edit and the caret is outside the definition
- **THEN** the definition renders indented without its `:` marker
- **AND** moving the caret into the definition reveals the authored marker for editing
- **AND** typing inside the definition edits exactly that source text

#### Scenario: Nested block in a definition
- **WHEN** a definition contains a nested list
- **THEN** the definition row owns only the bytes before the list, and the list renders as an ordinary list after it

#### Scenario: Emoji names do not become definition bodies
- **WHEN** consecutive lines begin with recognized tokens such as `:book:` and `:bug:`
- **THEN** those complete tokens render as emoji without treating their first colon as a definition marker
- **AND** a genuine `: definition` marker continues to define a term
