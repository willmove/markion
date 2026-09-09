## ADDED Requirements

### Requirement: Rich text clipboard paste converts HTML to Markdown
When the system clipboard offers an HTML representation (`text/html` flavor) alongside plain text, Edit → Paste on the document editing surface SHALL convert the HTML to Markion-compatible Markdown and insert the result instead of the plain text. The conversion SHALL preserve headings 1–6, paragraphs and line breaks, bold/italic/strikethrough/underline/subscript/superscript (including style-attribute formatting), hyperlinks with safe schemes, images by reference, nested ordered/unordered/task lists, blockquotes, fenced and inline code, GFM tables with merged-cell tables preserved as one raw HTML table, and horizontal rules. Literal Markdown-significant characters in source text SHALL be escaped so pasted text does not form unintended Markdown structures. Non-content markup (scripts, styles, comments, document metadata) SHALL be skipped. The whole paste SHALL remain a single atomic undo step, and an empty conversion result SHALL fall back to inserting the accompanying plain text.

#### Scenario: Word content pasted as formatted Markdown
- **WHEN** the user copies formatted content (headings, bold, a nested list, a table) from Microsoft Word or Google Docs and pastes it into a Markion document
- **THEN** the inserted text is Markdown carrying the equivalent structure (e.g. `#`, `**`, list markers, a GFM table)
- **AND** undoing once removes the entire paste

#### Scenario: Plain-text-only clipboard pastes verbatim
- **WHEN** the clipboard contains only plain text with no HTML flavor
- **THEN** paste inserts the text byte-for-byte as before

#### Scenario: Empty conversion falls back to plain text
- **WHEN** the clipboard offers HTML that converts to empty output (for example only scripts or styles)
- **THEN** paste inserts the accompanying plain text instead

#### Scenario: Text fields and image paste unaffected
- **WHEN** a text input field (search, link editor, file name, git field) has focus, or the clipboard carries an image entry
- **THEN** paste behaves exactly as before: raw text into the focused field, or the managed-asset image import flow

#### Scenario: HTML-only clipboard pastes converted Markdown
- **WHEN** the clipboard offers HTML without a plain-text flavor
- **THEN** paste inserts the converted Markdown rather than reporting an empty clipboard
