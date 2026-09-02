## ADDED Requirements

### Requirement: Comprehensive default Markdown example
When Markion creates its initial in-memory welcome document, it SHALL provide a structured `# Welcome to Markion` Markdown example that demonstrates the application's supported Markdown authoring syntax. The example SHALL include headings; paragraphs; emphasis, strong emphasis, strikethrough, inline code, links, and image syntax; blockquotes and thematic breaks; ordered, unordered, nested, and task lists; GFM tables; fenced code blocks; inline and display math; footnotes; the supported highlight, superscript, and subscript inline extensions; a raw HTML `<table>` that includes header cells and at least one `colspan` or `rowspan`; and other raw HTML tags that the HTML preview pipeline already renders (including emphasis/strong, a line break, a list, and `<kbd>` or equivalent). The Markdown image destination SHALL be the bundled branding raster `assets/markion.png` (or an equivalent packaged path under `assets/`) so Split Preview, Read mode, and Visual Edit can resolve the file from the application resource root without a saved document path and without a network fetch. The sample text SHALL use Markion-appropriate, self-contained language and SHALL NOT promote social-media, messaging-platform, or unrelated tool branding.

#### Scenario: Fresh document presents a broad Markdown tour
- **WHEN** the application creates its initial untitled document or replaces the last closed tab with a fresh document
- **THEN** the document starts with `# Welcome to Markion`
- **AND** contains an organized example for every required block and inline syntax category
- **AND** contains a raw HTML table sample and a raw HTML sample of other supported tags

#### Scenario: Welcome image resolves from bundled assets
- **WHEN** the welcome document is shown in Split Preview, Read mode, or Visual Edit and no file path is associated with the tab
- **THEN** the Markdown image whose destination is `assets/markion.png` loads the packaged branding PNG from the application resource root
- **AND** the preview does not present that image as a missing local file

#### Scenario: Starter content remains non-localized document text
- **WHEN** the application language is changed
- **THEN** the welcome Markdown sample remains fixed document content
- **AND** only user-interface chrome is localized

#### Scenario: Visual editing handles the example conservatively
- **WHEN** the welcome document is opened in Visual Edit mode
- **THEN** ordinary prose and list content remains source-backed and visually editable
- **AND** constructs requiring conservative source editing retain their source-editing affordance
