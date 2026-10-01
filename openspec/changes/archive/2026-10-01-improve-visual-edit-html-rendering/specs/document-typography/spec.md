## ADDED Requirements

### Requirement: HTML structural preview uses rendered typography
HTML preview parts in Read mode, Split Preview, and Visual Edit SHALL derive heading sizes for `<h1>`–`<h6>`, list-marker metrics, and `<pre>` code-slot font size and line height from the resolved rendered-document body size, preserving the same visual proportions Markdown headings, lists, and fenced code use. Changing typography preferences SHALL NOT mutate document text or per-version derived Markdown caches.

#### Scenario: HTML heading scales with body size
- **WHEN** the user changes the rendered-document font size
- **THEN** HTML `<h1>`–`<h6>` parts in preview and Visual Edit scale with Markdown headings
- **AND** document version and derived caches remain unchanged

#### Scenario: HTML pre uses the code slot
- **WHEN** an HTML `<pre>` block is visible
- **THEN** it uses the code-slot font family and the typography metrics derived from the rendered body size
