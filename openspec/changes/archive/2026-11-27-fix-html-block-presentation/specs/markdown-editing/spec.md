## MODIFIED Requirements

### Requirement: Shared HTML preview preserves common document structure
HTML preview parts SHALL present `<h1>`–`<h6>` with heading typography derived from the rendered body size, `<ul>`/`<ol>`/`<li>` with bullet or decimal markers, and `<pre>` with preserved whitespace and code-slot font. Alignment SHALL honor `left`/`right`/`center` (attribute or `text-align`). Every line of an aligned HTML heading or paragraph SHALL follow its block alignment, including lines separated by `<br>` and lines that wrap within the pane. Inline `color` from allowlisted hex/`rgb()` values and underline from `<u>` SHALL paint on text spans. These presentations apply in Read, Split Preview, and Visual Edit HTML blocks.

#### Scenario: HTML heading uses heading size
- **WHEN** a raw HTML block contains `<h1>Title</h1>`
- **THEN** the text `Title` renders at the document's H1 size, not body size

#### Scenario: HTML list shows markers
- **WHEN** a raw HTML block contains `<ul><li>one</li><li>two</li></ul>`
- **THEN** the preview shows two marked list items rather than unmarked wrapped lines

#### Scenario: HTML pre keeps spaces and newlines
- **WHEN** a raw HTML block contains `<pre>  a\n    b</pre>`
- **THEN** the preview shows the leading spaces and the line break instead of collapsing them to a single space

#### Scenario: Right alignment and underline
- **WHEN** a raw HTML block contains `<p align="right"><u>note</u></p>`
- **THEN** the text is right-aligned and underlined

#### Scenario: README heading, description, and caption are centered
- **WHEN** a raw HTML block contains a centered heading, a centered paragraph with `<strong>`, `<br>`, and a longer second line, or a centered `<sub>` caption with a link
- **THEN** Read, Split Preview, and Visual Edit center each displayed line within the row, including when the description wraps
- **AND** inline styles and links retain their rendering and interaction

### Requirement: Visual Edit HTML blocks keep rendered view when focused
Focusing a `VisualBlockKind::Html` row SHALL keep the shared HTML-parts rendering visible. Rendered HTML SHALL flow without an enclosing editor border. An HTML table SHALL keep its own grid borders. The block SHALL expose a collapsible exact source payload editor over the complete authored HTML range (the same render-plus-payload pattern used for diagrams and display math). Expanding, collapsing, or hovering the source control SHALL NOT change document version or invalidate per-version derived caches. The block SHALL NOT be replaced by a bordered whole-block source island solely because it owns the caret.

#### Scenario: Focused HTML table stays rendered
- **WHEN** the caret enters a Visual Edit HTML block that renders as a table or image
- **THEN** the table or image remains visible
- **AND** an exact source payload for the authored HTML is available without replacing the rendered view

#### Scenario: HTML source payload is source-backed
- **WHEN** the user edits the HTML block's source payload
- **THEN** the mutation applies to `MarkdownDocument.text` through the existing dirty-state and undo path
- **AND** the next derived version re-renders the HTML parts from the updated source

#### Scenario: README HTML renders without outer frames
- **WHEN** Visual Edit renders raw HTML containing a README title, language links, badges, navigation links, or a caption
- **THEN** the rendered content appears without an enclosing editor border
- **AND** the source toggle and expanded source field remain available
