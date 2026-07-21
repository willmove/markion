## ADDED Requirements

### Requirement: Built-in HTML export renders static math
The built-in styled and plain HTML exporters SHALL render valid inline and display math through the same GPUI-free math renderer used by native preview. Each formula SHALL be emitted in a stable inline/display container containing its byte-identical authored payload (modulo HTML attribute escaping) in `data-latex`, style/validity metadata, an accessible authored-source label or fallback, and sanitized self-contained SVG. Exported math SHALL require no script, browser-side renderer, network resource, event handler, external font, or interactive runtime. A rendering failure SHALL NOT fail the document export and SHALL instead preserve exact escaped authored syntax in a stable error container. Export formats other than styled and plain HTML SHALL retain their existing math behavior in this change.

#### Scenario: Styled HTML contains self-contained formula SVG
- **WHEN** a document with valid inline and display math is exported as styled HTML
- **THEN** each formula is represented by sanitized self-contained SVG with the correct inline or display semantics
- **AND** the document contains no client-side math runtime or remote renderer reference

#### Scenario: Plain HTML contains static math without default document CSS
- **WHEN** the same document is exported as plain HTML
- **THEN** it contains the same static formula semantics and source metadata
- **AND** it continues to omit Markion's default document stylesheet

#### Scenario: Authored LaTeX survives extended-inline processing
- **WHEN** a paragraph contains `$a^2+b^2=c^2$` together with extended inline syntax such as `x^2^`
- **THEN** the formula container carries `data-latex="a^2+b^2=c^2"` byte-identically after HTML escaping while `x^2^` renders as superscript
- **AND** generated SVG and formula payload are not rewritten by later Markdown text transformations

#### Scenario: Invalid math falls back without failing export
- **WHEN** the math renderer rejects an expression during styled or plain HTML export
- **THEN** export succeeds with a stable error container containing the exact escaped authored math syntax and validity metadata
- **AND** no stale SVG from another expression is emitted

#### Scenario: Formula SVG is inert and self-contained
- **WHEN** exported formula SVG is inspected
- **THEN** it contains no script, event handler, external link, external font, or network-loaded resource
- **AND** authored text and metadata cannot inject markup

#### Scenario: Other export formats keep current behavior
- **WHEN** a document with math is exported to Markdown, LaTeX, DOCX, PDF, PNG, or JPEG
- **THEN** that format follows its pre-existing source-preserving, Unicode fallback, pandoc, or text-snapshot behavior rather than claiming the new static-SVG path
