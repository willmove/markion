# export

## Purpose

Covers the multi-format export engine and YAML front matter metadata handling. The exports range from full-fidelity (HTML, DOCX, LaTeX) to deliberately limited (a simple single-page text PDF and basic text-snapshot PNG/JPEG). Rich image export fidelity is **not** part of this capability — it is a future candidate.
## Requirements
### Requirement: Multi-format document export
The export engine SHALL export the document to Markdown, styled HTML, plain HTML, LaTeX, DOCX, PDF, and basic PNG/JPEG text snapshots, prompting the user for an output path and suggesting a filename based on the current document. For PDF and DOCX, the editor SHALL first attempt the absorbed Typune export engine (pandoc subprocess, with the PDF engine taken from the `[export] pdf_engine` config value, default `xelatex`); if the external tool is unavailable or the conversion fails, it SHALL silently fall back to the built-in simple implementations so export always succeeds without external dependencies. The status bar message for a successful PDF/DOCX export SHALL disclose which backend produced the file — the pandoc engine, or the built-in writer together with a hint that installing pandoc yields richer output. Export failures SHALL be reported with user-facing status messages.

#### Scenario: Engine-produced export is disclosed
- **WHEN** the user exports to PDF or DOCX and the pandoc engine succeeds
- **THEN** the status message names the output path and indicates the pandoc engine produced it

#### Scenario: Built-in fallback is disclosed with a hint
- **WHEN** the user exports to PDF or DOCX and the editor falls back to the built-in writer
- **THEN** the status message names the output path, indicates the built-in writer was used, and hints that installing pandoc improves output quality

#### Scenario: PDF engine is configurable via the config file
- **WHEN** `[export] pdf_engine` is set in `config.toml` (e.g. `"pdfroff"`, `"tectonic"`)
- **THEN** the pandoc invocation for PDF export uses that engine instead of the default `xelatex`

#### Scenario: Full-fidelity text exports
- **WHEN** the user exports to styled HTML, plain HTML, LaTeX, or DOCX
- **THEN** the export preserves headings, lists, tables (with parsed alignment for LaTeX/HTML), code blocks, math fallback, and footnote/highlight/superscript constructs as each format allows

#### Scenario: PDF and DOCX fallback without pandoc
- **WHEN** the user exports to PDF or DOCX and the pandoc engine path fails (tool missing or conversion error)
- **THEN** the editor silently falls back to the built-in implementation and the export still succeeds

#### Scenario: Basic image snapshot export
- **WHEN** the user exports to PNG or JPEG
- **THEN** a basic text snapshot of the document is produced

#### Scenario: Output path is chosen by the user
- **WHEN** the user triggers an export
- **THEN** the editor prompts for a save location and suggests a filename derived from the current document

#### Scenario: Export failures are reported
- **WHEN** an export step fails
- **THEN** the editor shows a user-facing status message describing the failure

### Requirement: YAML front matter parsing and export metadata
The parser SHALL recognize a leading `---`-delimited YAML front matter block, parse key/value pairs, hide the block in the preview, and use recognized metadata (title, author, date) in the HTML export. The parser does not perform full YAML schema validation.

#### Scenario: Front matter is recognized and parsed
- **WHEN** the document begins with a `---`-delimited block
- **THEN** the parser extracts the key/value metadata and the preview hides the front matter block

#### Scenario: HTML export uses title, author, date
- **WHEN** the document is exported to HTML and the front matter contains title, author, or date
- **THEN** the HTML export incorporates that metadata into the rendered document

### Requirement: Native HTML/LaTeX export fidelity
The built-in HTML export SHALL keep inline and display math payloads byte-identical to the authored LaTeX (modulo HTML escaping) — extended inline syntax (superscript, subscript, emoji, autolink, highlight) SHALL NOT rewrite text inside math containers. The built-in LaTeX export SHALL preserve resolved inline styling (bold, italic, strikethrough, highlight, superscript, subscript, inline code, links), derive table column alignment from the Markdown separator row, render fenced code as `lstlisting` blocks (naming the language only when listings supports it), and place consecutive list items of the same kind in a single list environment with task-list checkboxes rendered as checkbox symbols.

#### Scenario: Inline math survives the superscript extension
- **WHEN** a paragraph contains `$a^2+b^2=c^2$` together with extended inline syntax such as `x^2^`
- **THEN** the exported HTML carries `data-latex="a^2+b^2=c^2"` unmodified while `x^2^` still renders as `<sup>2</sup>`

#### Scenario: LaTeX preserves inline styles
- **WHEN** a paragraph with bold, strikethrough, highlight, superscript, and link spans is exported to LaTeX
- **THEN** the output uses `\textbf`, `\sout`, `\hl`, `\textsuperscript`, and `\href` rather than flattening to plain text

#### Scenario: LaTeX table alignment follows the separator row
- **WHEN** a table declares `|:--|:-:|--:|`
- **THEN** the LaTeX `longtable` column spec is `{lcr}`

#### Scenario: Task list renders as one environment with checkboxes
- **WHEN** consecutive task-list items are exported to LaTeX
- **THEN** they share a single `itemize` environment and render `$\boxtimes$`/`$\square$` markers

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

