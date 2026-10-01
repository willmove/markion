## ADDED Requirements

### Requirement: Built-in exports render definition lists
Built-in HTML export SHALL emit definition lists as `<dl>`, `<dt>`, and `<dd>` elements. The built-in DOCX, PDF, and LaTeX exporters SHALL render each definition-list term as a bold paragraph and each definition as a following paragraph; DOCX and LaTeX SHALL indent the definition. Inline formatting inside terms and definitions SHALL be preserved as it is for paragraphs.

#### Scenario: HTML export uses definition list elements
- **WHEN** a document containing `Term` followed by `: Definition` is exported to HTML with the built-in exporter
- **THEN** the output contains a `<dl>` with a `<dt>` for the term and a `<dd>` for the definition

#### Scenario: DOCX, PDF, and LaTeX export bold terms
- **WHEN** the same document is exported to DOCX, PDF, or LaTeX with the built-in exporters
- **THEN** the term appears as bold text and the definition text appears in a following paragraph, indented in DOCX and LaTeX
