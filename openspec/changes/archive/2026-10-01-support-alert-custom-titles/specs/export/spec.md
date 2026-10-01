## ADDED Requirements

### Requirement: Exports SHALL show alert titles
Built-in exports SHALL label every GFM alert with its title: the custom title when the alert has one, otherwise the kind label (Note, Tip, Important, Warning, Caution). DOCX and PDF SHALL put the title in the existing bold alert label line. LaTeX SHALL emit the title as a bold first line inside the alert's quote environment. HTML SHALL render every alert as a blockquote carrying the `markdown-alert` and `markdown-alert-<kind>` classes whose first child is a `<p class="markdown-alert-title">` with the escaped title. The marker line of a titled alert SHALL NOT appear in any export's alert body.

#### Scenario: Titled alert in DOCX and PDF
- **WHEN** a document containing `> [!NOTE] 注意` followed by a quoted body line is exported to DOCX or PDF with the built-in exporters
- **THEN** the alert's bold label reads "注意" and the body does not contain `[!NOTE]`

#### Scenario: Titled alert in HTML
- **WHEN** the same document is exported to HTML
- **THEN** the output contains a blockquote with the `markdown-alert-note` class whose first child is `<p class="markdown-alert-title">注意</p>`
- **AND** the body does not contain `[!NOTE]`

#### Scenario: Untitled alert label in HTML and LaTeX
- **WHEN** a document containing `> [!TIP]` followed by a body line is exported to HTML or LaTeX
- **THEN** the alert shows the default label "Tip" as its title
