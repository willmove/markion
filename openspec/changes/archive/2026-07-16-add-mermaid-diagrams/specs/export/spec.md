## ADDED Requirements

### Requirement: Built-in HTML export renders static diagrams
The built-in styled and plain HTML exporters SHALL resolve registered diagram fences through the same GUI-free diagram registry used by preview. A valid Mermaid fence SHALL be replaced with sanitized inline SVG inside a stable diagram container after Markdown extended-inline and math transformations have completed. The exported document SHALL NOT add Mermaid.js, executable scripts, external diagram resources, network dependencies, or interactive event handlers. Diagram rendering failures SHALL NOT fail the entire HTML export; the exporter SHALL instead preserve the exact authored source as escaped fenced-code fallback content. Export formats other than styled and plain HTML SHALL retain their existing behavior in this change.

#### Scenario: Styled HTML contains inline Mermaid SVG
- **WHEN** a document with a valid Mermaid fence is exported as styled HTML
- **THEN** the output contains sanitized inline SVG for that fence and contains no Mermaid runtime script or remote renderer reference

#### Scenario: Plain HTML contains inline Mermaid SVG without default CSS
- **WHEN** a document with a valid Mermaid fence is exported as plain HTML
- **THEN** the output contains the same static diagram semantics while continuing to omit Markion's default document stylesheet

#### Scenario: Invalid Mermaid source falls back without failing export
- **WHEN** the Mermaid backend rejects a diagram during styled or plain HTML export
- **THEN** export succeeds with escaped code fallback containing the exact authored diagram source

#### Scenario: Diagram SVG bypasses Markdown text rewriting
- **WHEN** generated SVG labels contain characters that resemble Markion extended inline syntax or math delimiters
- **THEN** the sanitized SVG is inserted after those Markdown transformations and its label content is not rewritten by them

#### Scenario: Other export formats keep current behavior
- **WHEN** a document with a Mermaid fence is exported to Markdown, LaTeX, DOCX, PDF, PNG, or JPEG
- **THEN** that format follows its pre-existing code-block or text-snapshot behavior rather than claiming rich diagram rendering
