## ADDED Requirements

### Requirement: Visual Edit renders HTML images
Visual Edit SHALL present raw-HTML images the same way Read mode does wherever Read mode renders them, and SHALL NOT collapse prose blocks into raw-source islands solely because they contain image tags. Standalone raw-HTML blocks containing `<img>` SHALL render read-only through the shared HTML-parts pipeline (text, images, tables) with the existing focused source-island editing affordance. Inline `<img>` tags inside paragraphs, headings, list items, blockquote leaves, and footnote text SHALL render as inline image atoms loaded through the same image pipeline as preview (workspace-relative paths, remote URLs, and data URIs), while the surrounding prose remains rendered and editable. Each inline image atom SHALL be source-backed: entering its byte-exact authored `<img>` tag range with the caret or a selection endpoint SHALL reveal the complete authored tag as one editable source run, and leaving the range SHALL restore the rendered atom without changing the document version. Prose blocks whose only inline HTML consists of complete `<img>` tags SHALL NOT use a whole-block HTML source island; any other inline HTML SHALL keep the whole-block source-island fallback. Images inside GFM table cells SHALL present the flattened alt/URL text exactly as Read mode does.

#### Scenario: Standalone HTML image block renders
- **WHEN** an unfocused Visual Edit document contains a raw-HTML block such as `<p align="center"><img src="logo.svg" alt="Logo"></p>`
- **THEN** the block renders through the shared HTML-parts pipeline showing the image and honoring centering
- **AND** focusing the block presents the existing conservative source island for editing its raw HTML

#### Scenario: Inline HTML image renders inside prose
- **WHEN** an unfocused Visual Edit paragraph, heading, list item, or blockquote line contains text and one or more complete `<img>` tags
- **THEN** each tag renders as an inline image atom between the surrounding rendered prose runs
- **AND** the block does not present a whole-block raw-source island

#### Scenario: Focused inline image reveals its exact source
- **WHEN** the caret or a selection endpoint enters the authored `<img …>` source range of an inline image atom
- **THEN** the complete byte-exact tag is revealed as one editable source run
- **AND** moving the caret out restores the rendered atom without a document-version change

#### Scenario: Other inline HTML keeps the conservative fallback
- **WHEN** a prose block mixes an `<img>` tag with any other inline HTML (for example `<br>` or `<em>…</em>`)
- **THEN** the block keeps the whole-block HTML source-island presentation
- **AND** no partial rendering mutates or misrepresents the authored source

#### Scenario: HTML image in a table cell matches Read mode
- **WHEN** a GFM table cell contains a complete `<img>` tag and the table contains no other inline HTML
- **THEN** the table renders with the cell showing the flattened alt/URL text as Read mode does
- **AND** the table does not collapse into a whole-table source island

#### Scenario: Inline HTML images share the preview image lifecycle
- **WHEN** an inline HTML image is visible in Visual Edit
- **THEN** its URL is claimed, preloaded, and evicted through the same preview image cache lifecycle as block-level images
- **AND** pending and failed loads present the same placeholders as Read mode
