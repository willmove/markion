## CHANGED Requirements

### Requirement: Multi-format document export
The export engine SHALL export the document to Markdown, styled HTML, plain HTML, LaTeX, DOCX, PDF, and PNG/JPEG layout snapshots, prompting the user for an output path and suggesting a filename based on the current document. For PDF and DOCX, the editor SHALL first attempt the absorbed Typune export engine (pandoc subprocess, with the PDF engine taken from the `[export] pdf_engine` config value, default `xelatex`); if the external tool is unavailable or the conversion fails, it SHALL silently fall back to the built-in implementations (the rich built-in PDF writer, the built-in DOCX writer) so export always succeeds without external dependencies. The status bar message for a successful PDF/DOCX export SHALL disclose which backend produced the file. For DOCX, the built-in-writer message retains a hint that installing pandoc yields richer output; for PDF, the built-in-writer message SHALL NOT claim that pandoc yields richer output, because the built-in PDF writer is the rich default. When the pandoc engine fails and the fallback is used, the status message SHALL additionally indicate the failure category (pandoc not found vs. conversion error). Export failures SHALL be reported with user-facing status messages.

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

#### Scenario: Rich image snapshot export
- **WHEN** the user exports to PNG or JPEG
- **THEN** a layout snapshot of the rendered document is produced, with real fonts, Markdown structure, and CJK text rendered as actual glyphs rather than an ASCII-only text dump

#### Scenario: Output path is chosen by the user
- **WHEN** the user triggers an export
- **THEN** the editor prompts for a save location and suggests a filename derived from the current document

#### Scenario: Export failures are reported
- **WHEN** an export step fails
- **THEN** the editor shows a user-facing status message describing the failure

## ADDED Requirements

### Requirement: Rich PNG/JPEG snapshot fidelity
The built-in PNG/JPEG snapshot SHALL render the document through the same layout IR and font pipeline used by the built-in PDF writer, so a snapshot matches the PDF typography. It SHALL render headings H1–H6, paragraphs, bulleted/ordered/task lists with nesting, blockquotes, GFM alert callouts, fenced code blocks (with a light background and syntax-run coloring), tables with a bold header and per-column alignment, horizontal rules, and local PNG/JPEG/SVG images scaled to the text column. Text SHALL be shaped with the process-wide font system (per-OS CJK fonts, then the bundled Noto Sans SC subset, plus Latin/code fallbacks) so CJK and accented characters render as real glyphs and no character is substituted by a placeholder. Remote and data-URI images SHALL keep the existing text fallback. The document SHALL flow into one continuous canvas honoring the configured page size and margins; there SHALL be no pagination or page-number footer.

#### Scenario: CJK text renders as real glyphs
- **WHEN** a document containing Chinese text is exported to PNG or JPEG
- **THEN** the snapshot pixels contain the Han glyph shapes from a real font rather than hollow replacement boxes

#### Scenario: Markdown structure is preserved
- **WHEN** a document with headings, lists, a table, and a fenced code block is exported to PNG or JPEG
- **THEN** the snapshot shows the same block structure (distinct heading sizes, list indentation, a bordered table, and a code block with a background) rather than a flat monospaced text dump

#### Scenario: Embedded local images are included
- **WHEN** a document references an existing local PNG/JPEG/SVG image wider than the text column
- **THEN** the snapshot embeds the image scaled to the column width

#### Scenario: Remote or data-URI images keep the text fallback
- **WHEN** an image source is remote, a data URI, or a local path that does not exist
- **THEN** the snapshot emits the `alt: url` text and the export still succeeds
