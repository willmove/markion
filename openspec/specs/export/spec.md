# export

## Purpose

Covers the multi-format export engine and YAML front matter metadata handling. The exports range from full-fidelity (HTML, DOCX, LaTeX) to deliberately limited (a simple single-page text PDF and basic text-snapshot PNG/JPEG). Rich image export fidelity is **not** part of this capability — it is a future candidate.
## Requirements
### Requirement: Multi-format document export
The export engine SHALL export the document to Markdown, styled HTML, plain HTML, LaTeX, DOCX, PDF, and basic PNG/JPEG text snapshots, prompting the user for an output path and suggesting a filename based on the current document. For PDF and DOCX, the editor SHALL first attempt the absorbed Typune export engine (pandoc subprocess, with the PDF engine taken from the `[export] pdf_engine` config value, default `xelatex`); if the external tool is unavailable or the conversion fails, it SHALL silently fall back to the built-in implementations (the rich built-in PDF writer, the built-in DOCX writer) so export always succeeds without external dependencies. The status bar message for a successful PDF/DOCX export SHALL disclose which backend produced the file. For DOCX, the built-in-writer message retains a hint that installing pandoc yields richer output; for PDF, the built-in-writer message SHALL NOT claim that pandoc yields richer output, because the built-in PDF writer is the rich default. When the pandoc engine fails and the fallback is used, the status message SHALL additionally indicate the failure category (pandoc not found vs. conversion error). Export failures SHALL be reported with user-facing status messages.

#### Scenario: Engine-produced export is disclosed
- **WHEN** the user exports to PDF or DOCX and the pandoc engine succeeds
- **THEN** the status message names the output path and indicates the pandoc engine produced it

#### Scenario: Built-in fallback is disclosed with a hint
- **WHEN** the user exports to DOCX and the editor falls back to the built-in writer
- **THEN** the status message names the output path, indicates the built-in writer was used, and hints that installing pandoc improves output quality

#### Scenario: Built-in PDF export is disclosed neutrally
- **WHEN** the user exports to PDF and the built-in PDF writer produced the file
- **THEN** the status message names the output path and indicates the built-in PDF engine produced it, without hinting that installing pandoc improves output quality

#### Scenario: Engine failure category is disclosed
- **WHEN** the pandoc engine fails (binary missing or conversion error) and the fallback produces the file
- **THEN** the status message indicates which failure category occurred

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
- **WHEN** a document with math is exported to Markdown, LaTeX, DOCX, PNG, or JPEG
- **THEN** that format follows its pre-existing source-preserving, Unicode fallback, pandoc, or text-snapshot behavior rather than claiming the new static-SVG path

#### Scenario: PDF math follows the PDF writer requirements
- **WHEN** a document with math is exported to PDF
- **THEN** the built-in writer applies the math behavior defined by the built-in PDF writer requirements, and the pandoc engine path keeps its native LaTeX math behavior — neither claims the static-SVG path

### Requirement: Built-in DOCX fallback package completeness
The built-in DOCX writer SHALL emit a structurally complete OOXML package in which every style, numbering, and relationship reference resolves. The package SHALL include `word/styles.xml` (with `docDefaults` and paragraph/character styles for Normal, Title, Heading1 through Heading6, Quote, and code), `word/numbering.xml`, `word/theme/theme1.xml`, `word/settings.xml`, and `word/fontTable.xml`, in addition to the existing content-types, relationships, core-properties, and document parts. Headings at every Markdown level (H1–H6) SHALL map to their own distinct heading style.

#### Scenario: All style references resolve
- **WHEN** a document containing headings H1–H6 is exported via the built-in DOCX writer
- **THEN** the package contains `word/styles.xml` defining Heading1 through Heading6 and Normal
- **AND** every `w:pStyle` value used in `word/document.xml` is defined in `word/styles.xml`

#### Scenario: Six distinct heading levels
- **WHEN** a document contains headings from `#` through `######`
- **THEN** `word/document.xml` references six distinct styles Heading1–Heading6 with no collapse of deeper levels

#### Scenario: Front matter title uses a Title style
- **WHEN** the document front matter provides a title
- **THEN** the exported document opens with a Title-styled paragraph and the title is still written to `docProps/core.xml`

### Requirement: Built-in DOCX fallback inline fidelity
The built-in DOCX writer SHALL preserve resolved inline styling by consuming the rich-text spans already computed for preview. Bold, italic, strikethrough, highlight, superscript, subscript, and inline code SHALL map to the corresponding `w:rPr` properties (with `w:highlight` for highlight and `w:vertAlign` for super/subscript), and links SHALL be emitted as real `w:hyperlink` elements backed by external relationships in `word/_rels/document.xml.rels` so the target URL survives. Inline code SHALL use a monospace font declaration.

#### Scenario: Inline styles are preserved
- **WHEN** a paragraph contains bold, italic, strikethrough, highlight, superscript, subscript, and inline code spans
- **THEN** `word/document.xml` carries `w:b`, `w:i`, `w:strike`, `w:highlight`, `w:vertAlign` (superscript/subscript), and a monospace `w:rFonts` run respectively, rather than flattening to plain text

#### Scenario: Links keep their targets
- **WHEN** a paragraph contains `[label](https://example.com)`
- **THEN** the document contains a `w:hyperlink` for `label` whose relationship in `word/_rels/document.xml.rels` targets `https://example.com`

#### Scenario: Nested inline styles compose
- **WHEN** a span carries multiple styles (e.g. bold italic)
- **THEN** the run carries all corresponding `w:rPr` properties simultaneously

### Requirement: Built-in DOCX fallback structural lists
The built-in DOCX writer SHALL render unordered, ordered, and task lists as real Word list paragraphs. `word/numbering.xml` SHALL define abstract bullet and decimal numbering formats; list items SHALL reference them via `w:numPr` with `w:ilvl` reflecting the item's nesting depth, so nested lists keep their hierarchy and remain editable in Word. Task-list items SHALL keep a checked/unchecked marker prefix until Word checkbox content controls are adopted.

#### Scenario: Nested lists keep depth
- **WHEN** a document contains a bullet list nested two levels deep
- **THEN** items at each depth reference the numbering definition with the matching `w:ilvl` and render with increasing indentation in Word

#### Scenario: Ordered lists are auto-numbered
- **WHEN** a document contains an ordered list
- **THEN** items use a decimal `w:numFmt` and no literal `1. ` text is emitted for the marker

### Requirement: Built-in DOCX fallback typography and page setup
The built-in DOCX writer SHALL declare east-asian fonts for CJK text. `docDefaults` and the heading styles SHALL carry `w:rFonts` with an `w:eastAsia` attribute (body, heading, and code styles each with a sensible default), so Chinese text renders with a controlled font instead of Word fallback heuristics. The default page setup SHALL be A4 with 2.54 cm margins instead of hard-coded US Letter.

#### Scenario: CJK fonts are declared
- **WHEN** any document is exported via the built-in DOCX writer
- **THEN** `word/styles.xml` `docDefaults` and heading styles carry `w:rFonts` `w:eastAsia` declarations

#### Scenario: Default page is A4
- **WHEN** a document is exported with default options
- **THEN** `w:sectPr` in `word/document.xml` specifies A4 dimensions (11906×16838 twips) and 1440-twip margins

### Requirement: Built-in DOCX fallback embeds local images
The built-in DOCX writer SHALL embed local image files into the package. A Markdown image whose source resolves to an existing local file (relative paths resolved against the document's directory) SHALL be copied into `word/media/` with a unique name, declared in `[Content_Types].xml`, and referenced from `word/document.xml` as a `w:drawing` sized in EMUs so that images wider than the text column are scaled down to fit while narrower images keep their natural pixel size (at 96 DPI). The image's alt text SHALL be preserved as the drawing's description. Remote (`http(s)`) and data-URI images SHALL keep the existing text fallback.

#### Scenario: Local image is embedded
- **WHEN** a document references `![diagram](images/diagram.png)` and the file exists relative to the document
- **THEN** the package contains `word/media/` with the image bytes, a relationship, a content-type entry for the extension, and a `w:drawing` in the document flow

#### Scenario: Oversized image is scaled to the column
- **WHEN** an embedded image's natural width at 96 DPI exceeds the text column width
- **THEN** the `wp:extent` scales it down proportionally to fit the column

#### Scenario: Missing or remote images keep the text fallback
- **WHEN** an image source is remote, a data URI, or a local path that does not exist
- **THEN** the writer emits the existing `alt: url` text paragraph and the export still succeeds

### Requirement: Built-in DOCX fallback table fidelity
The built-in DOCX writer SHALL render tables with a bold header row marked `w:tblHeader` (repeat as header on page breaks), per-column horizontal alignment (`w:jc`) derived from the parsed Markdown separator row, preserved inline styles within cells, and a table width fitted to the text column with proportional column widths. Raw HTML `<table>` blocks SHALL be parsed into the same table structure rather than flattening each cell into a scattered paragraph.

#### Scenario: Header row repeats and is bold
- **WHEN** a table is exported via the built-in writer
- **THEN** the first `w:tr` carries `w:tblHeader` and its runs carry `w:b`

#### Scenario: Column alignment follows the separator row
- **WHEN** a table declares `|:--|:-:|--:|`
- **THEN** the columns render left/center/right aligned via `w:jc` in their cells

#### Scenario: HTML table keeps its structure
- **WHEN** the document contains a raw `<table>` block
- **THEN** the export contains a real `w:tbl` with the parsed rows and cells instead of one paragraph per cell

### Requirement: Built-in DOCX fallback renders math as OMML
The built-in DOCX writer SHALL convert inline and display math into OMML (`m:oMath` / `m:oMathPara`) so equations open as native editable Word equations. When a formula cannot be converted, the math zone SHALL contain the authored LaTeX source as its text (not a Unicode approximation, and without a `Math: ` prefix), and the export SHALL still succeed.

#### Scenario: Display math becomes a native equation
- **WHEN** a document contains a `$$`-fenced formula
- **THEN** `word/document.xml` contains an `m:oMathPara` representing the formula

#### Scenario: Unconvertible math preserves its source
- **WHEN** a formula uses constructs the converter does not support
- **THEN** the math zone text is the byte-identical authored LaTeX and the export succeeds

### Requirement: Built-in DOCX fallback footnotes, rules, and alerts
The built-in DOCX writer SHALL emit real footnotes: `word/footnotes.xml` carries the definitions and the body carries `w:footnoteReference` marks, so references and definitions stay linked in Word. Horizontal rules SHALL render as a paragraph with a bottom border rather than literal `----------` text. GFM alert blockquotes SHALL render as styled callout paragraphs (accented left border with a bold kind label) rather than `> `-prefixed text.

#### Scenario: Footnote reference links to its definition
- **WHEN** a document contains `[^a]` and its definition
- **THEN** the body carries a `w:footnoteReference w:id` and `word/footnotes.xml` contains the matching `w:footnote`

#### Scenario: Horizontal rule is a border
- **WHEN** the document contains `---` as a thematic break
- **THEN** the export contains a paragraph with `w:pBdr` bottom border and no literal dash text

#### Scenario: GFM alert renders as a callout
- **WHEN** the document contains `> [!WARNING]` with a body
- **THEN** the export renders a bold kind label and the body with callout-style borders/indentation

### Requirement: DOCX package is deflate-compressed
The built-in DOCX writer SHALL compress package entries with deflate instead of storing them uncompressed.

#### Scenario: Package entries are compressed
- **WHEN** any document is exported via the built-in writer
- **THEN** the ZIP local file headers use compression method 8 (deflate) and the package opens correctly in Word

### Requirement: Pandoc engine styling and resources
The DOCX pandoc engine path SHALL style its output through a reference document: a bundled default reference docx with CJK-friendly typography is used unless `[export] reference_doc` in `config.toml` points to a user-supplied file. The engine invocation SHALL pass `--resource-path` set to the current document's directory so relative image paths resolve, SHALL enable the pandoc Markdown extensions corresponding to Markion's extended inline syntax (at minimum `mark`, `superscript`, `subscript`), and SHALL apply a code `--highlight-style`. A table of contents (`--toc`) SHALL be emitted when the export options request it (default off).

#### Scenario: Bundled reference doc styles the output
- **WHEN** pandoc is available and no `reference_doc` is configured
- **THEN** the DOCX pandoc invocation includes `--reference-doc` pointing at the bundled template

#### Scenario: User reference doc overrides the bundled one
- **WHEN** `[export] reference_doc` names an existing file
- **THEN** the invocation uses that file instead of the bundled template

#### Scenario: Relative images resolve on the engine path
- **WHEN** the document references a relative image path and the pandoc engine runs
- **THEN** the invocation includes `--resource-path` containing the document's directory

#### Scenario: Extended inline syntax is enabled for pandoc
- **WHEN** the document contains `==highlight==`, `^superscript^`, or `~subscript~`
- **THEN** the pandoc invocation enables the `mark`, `superscript`, and `subscript` extensions so the engine output preserves them

### Requirement: Configurable pandoc binary path
The pandoc binary location SHALL be configurable via `[export] pandoc_path` in `config.toml`. When unset, the engine locates `pandoc` on the system PATH as today.

#### Scenario: Configured pandoc path is used
- **WHEN** `[export] pandoc_path` names an executable
- **THEN** the DOCX/PDF engine invocations use that executable instead of a PATH lookup

### Requirement: User-facing DOCX export options
The DOCX export flow SHALL offer user-facing options before writing the file: page size (A4, Letter, Legal), table of contents on the pandoc engine path (default off), and image embedding policy (embed local images vs. text fallback, default embed). Both the pandoc engine path and the built-in fallback SHALL honor the applicable options. The last-used DOCX export options SHALL persist across sessions via the `[export.docx]` config section.

#### Scenario: Options reach the engine path
- **WHEN** the user exports to DOCX with the pandoc engine available and has enabled the table of contents
- **THEN** the pandoc invocation includes `--toc`

#### Scenario: Options reach the fallback path
- **WHEN** the user exports to DOCX via the built-in fallback with Letter page size selected
- **THEN** the fallback writer's `w:sectPr` uses Letter dimensions instead of the A4 default

#### Scenario: Image policy is honored
- **WHEN** the user selects the text-fallback image policy
- **THEN** local images export as `alt: url` text on both backends instead of being embedded

#### Scenario: Options persist across sessions
- **WHEN** the user changes a DOCX export option and later restarts the app
- **THEN** the export dialog presents the previously used options

### Requirement: Built-in PDF writer pagination and document structure
The built-in PDF writer SHALL produce a true multi-page document: content is laid out with UAX#14 line breaking (CJK text may break between characters, Latin text at word boundaries), flows across as many pages as needed with no truncation, and respects the configured page size and margins. Headings SHALL generate PDF outline bookmarks matching their hierarchy, and the document metadata (title, author, date from YAML front matter, with the file-stem title fallback) SHALL be written to the PDF document properties. An optional page-number footer SHALL be emitted when enabled (default on).

#### Scenario: Long document paginates without loss
- **WHEN** a document whose rendered length exceeds one page is exported via the built-in writer
- **THEN** the PDF contains all content across multiple pages with no text truncated

#### Scenario: CJK paragraph wraps correctly
- **WHEN** a paragraph of Chinese text without spaces exceeds the line width
- **THEN** the text breaks between characters at the margin instead of overflowing the page

#### Scenario: Headings become outline bookmarks
- **WHEN** a document with `#`, `##`, and `###` headings is exported
- **THEN** the PDF outline contains matching nested bookmarks

#### Scenario: Front matter metadata is embedded
- **WHEN** the document front matter provides title, author, and date
- **THEN** the PDF document properties carry those values

### Requirement: Built-in PDF writer fonts and CJK support
The built-in PDF writer SHALL embed subsetted fonts covering every rendered glyph; it SHALL NOT substitute placeholder characters (such as `?`) for any Unicode content. Font resolution SHALL use ordered fallback stacks — a configured or per-OS system CJK font (Microsoft YaHei, PingFang SC, Noto Sans CJK SC) before a bundled OFL-licensed Noto Sans SC subset as the guaranteed fallback — declared separately for body, heading, and code text. Document language SHALL be detected well enough to select CJK-aware line breaking and CJK–Latin spacing when the document is predominantly Chinese.

#### Scenario: Chinese text renders without substitution
- **WHEN** a document containing Chinese text is exported via the built-in writer
- **THEN** the PDF embeds a font covering those glyphs and no character is replaced by a placeholder

#### Scenario: Export works without any system CJK font
- **WHEN** the built-in writer exports Chinese text on a system with no CJK system font
- **THEN** the bundled fallback font still renders the common Han glyphs

#### Scenario: Code blocks use a monospace stack
- **WHEN** a document contains a fenced code block
- **THEN** the code text renders with the monospace fallback stack, distinct from the body stack

### Requirement: Built-in PDF writer block fidelity
The built-in PDF writer SHALL render the cached preview blocks with structural fidelity: distinct heading levels H1–H6; bulleted, ordered (auto-numbered, no literal marker text), and task lists with nesting preserved; blockquotes as indented, visually set-off blocks; GFM alerts as styled callouts with a bold kind label; fenced code blocks in monospace with syntax highlighting and no mid-block page break when avoidable; tables with a bold header row repeated across page breaks and per-column alignment from the separator row, including parsed raw-HTML tables; horizontal rules as graphical rules; footnotes as real page footnotes linked from their references. Diagram fences SHALL render as code blocks.

#### Scenario: Ordered list is auto-numbered
- **WHEN** a document contains an ordered list
- **THEN** the PDF shows generated numbers with no literal `1.` marker text

#### Scenario: Table header repeats across pages
- **WHEN** a table spans a page break
- **THEN** the header row renders again at the top of the continued table

#### Scenario: GFM alert renders as a callout
- **WHEN** the document contains `> [!WARNING]` with a body
- **THEN** the PDF renders a bold WARNING label and a styled callout block rather than `> `-prefixed text

#### Scenario: Footnote reference resolves on the page
- **WHEN** a document contains `[^a]` and its definition
- **THEN** the reference links to a footnote rendered at the bottom of the page

### Requirement: Built-in PDF writer inline fidelity
The built-in PDF writer SHALL preserve resolved inline styling from the rich-text spans: bold, italic, strikethrough, highlight, superscript, subscript, and inline code map to the corresponding PDF text styling (weight, style, background, baseline offset, monospace stack), nested styles compose, and links render as clickable PDF links whose targets survive. Inline code SHALL use the monospace stack.

#### Scenario: Inline styles are preserved
- **WHEN** a paragraph contains bold, italic, strikethrough, highlight, superscript, subscript, and inline code spans
- **THEN** the PDF renders each with its distinct styling rather than flattening to plain text

#### Scenario: Links keep their targets
- **WHEN** a paragraph contains `[label](https://example.com)`
- **THEN** the PDF contains a clickable link on `label` targeting `https://example.com`

### Requirement: Built-in PDF writer embeds local images
The built-in PDF writer SHALL embed local image files (PNG, JPEG, SVG) whose paths resolve against the document's directory, scaling images wider than the text column down to fit while narrower images keep their natural size at 96 DPI, and preserving the alt text as the image's accessibility description. Remote (`http(s)`), data-URI, and unresolvable images SHALL fall back to an `alt: url` text paragraph without failing the export.

#### Scenario: Local image is embedded and scaled
- **WHEN** a document references `![diagram](images/diagram.png)` wider than the text column and the file exists relative to the document
- **THEN** the PDF embeds the image scaled to the column width

#### Scenario: Missing or remote images keep the text fallback
- **WHEN** an image source is remote, a data URI, or a local path that does not exist
- **THEN** the writer emits the `alt: url` text and the export still succeeds

### Requirement: Built-in PDF writer renders math
The built-in PDF writer SHALL render valid inline and display math as vector graphics through the same GPUI-free math renderer used by native preview and HTML export, embedded as SVG, so exported formulas match the preview. When the math renderer rejects a formula, the writer SHALL emit the byte-identical authored LaTeX source in a code-styled block and the export SHALL still succeed.

#### Scenario: Display math matches the preview
- **WHEN** a document contains a valid `$$`-fenced formula
- **THEN** the PDF embeds the same sanitized SVG the preview renders, as a display equation

#### Scenario: Unrenderable math preserves its source
- **WHEN** the math renderer rejects a formula
- **THEN** the PDF contains the byte-identical authored LaTeX in code styling and the export succeeds

### Requirement: User-facing PDF export options
PDF export SHALL honor user-facing options persisted in a new `[export.pdf]` config section: page size (A4, Letter, Legal; default A4), page margin in millimetres (default 25), table of contents (default off), and page-number footer (default on). The built-in writer SHALL apply all four options; the pandoc engine path SHALL map page size to `--variable=geometry:` and the table of contents to `--toc`. Unknown or missing values SHALL fall back to the defaults.

#### Scenario: Options reach the built-in writer
- **WHEN** `[export.pdf]` sets Letter page size and a 20 mm margin
- **THEN** the built-in PDF uses Letter geometry with 20 mm margins

#### Scenario: Table of contents is emitted
- **WHEN** the table-of-contents option is enabled and the document contains headings
- **THEN** the PDF opens with an outline page listing the headings with page numbers

#### Scenario: Options reach the pandoc engine path
- **WHEN** the pandoc engine runs with Legal page size and the table of contents enabled
- **THEN** the invocation includes the Legal geometry variable and `--toc`

### Requirement: Pandoc PDF engine fonts and resources
The pandoc PDF engine path SHALL configure CJK-capable fonts: when the document contains CJK text, the invocation SHALL pass a `CJKmainfont` variable naming an available per-OS system font (Microsoft YaHei on Windows, PingFang SC on macOS, Noto Sans CJK SC on Linux), overridable via config. The invocation SHALL pass `--resource-path` set to the current document's directory so relative image paths resolve, SHALL NOT pass flags that only affect HTML output (such as `--katex`), and SHALL honor the configured PDF page size via the geometry variable.

#### Scenario: CJK document gets a CJK font
- **WHEN** a document containing Chinese text is exported via the pandoc engine
- **THEN** the invocation includes a `CJKmainfont` variable naming a platform-appropriate font

#### Scenario: Relative images resolve on the engine path
- **WHEN** the document references a relative image path and the pandoc engine runs
- **THEN** the invocation includes `--resource-path` containing the document's directory

#### Scenario: No HTML-only flags are passed
- **WHEN** any PDF pandoc invocation is built
- **THEN** it does not contain `--katex`

