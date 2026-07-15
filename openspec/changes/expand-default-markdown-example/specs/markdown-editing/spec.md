## ADDED Requirements

### Requirement: Comprehensive default Markdown example
When Markion creates its initial in-memory welcome document, it SHALL provide a structured `# Welcome to Markion` Markdown example that demonstrates the application's supported Markdown authoring syntax. The example SHALL include headings; paragraphs; emphasis, strong emphasis, strikethrough, inline code, links, and image syntax; blockquotes and thematic breaks; ordered, unordered, nested, and task lists; tables; fenced code blocks; inline and display math; footnotes; and the supported highlight, superscript, and subscript inline extensions. The sample text SHALL use Markion-appropriate, self-contained language and SHALL NOT promote social-media, messaging-platform, or unrelated tool branding.

#### Scenario: Fresh document presents a broad Markdown tour
- **WHEN** the application creates its initial untitled document or replaces the last closed tab with a fresh document
- **THEN** the document starts with `# Welcome to Markion`
- **AND** contains an organized example for every required block and inline syntax category

#### Scenario: Starter content remains non-localized document text
- **WHEN** the application language is changed
- **THEN** the welcome Markdown sample remains fixed document content
- **AND** only user-interface chrome is localized

#### Scenario: Visual editing handles the example conservatively
- **WHEN** the welcome document is opened in Visual Edit mode
- **THEN** ordinary prose and list content remains source-backed and visually editable
- **AND** constructs requiring conservative source editing retain their source-editing affordance
