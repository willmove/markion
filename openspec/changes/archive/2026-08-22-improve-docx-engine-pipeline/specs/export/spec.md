## MODIFIED Requirements

### Requirement: Multi-format document export
The export engine SHALL export the document to Markdown, styled HTML, plain HTML, LaTeX, DOCX, PDF, and basic PNG/JPEG text snapshots, prompting the user for an output path and suggesting a filename based on the current document. For PDF and DOCX, the editor SHALL first attempt the absorbed Typune export engine (pandoc subprocess, with the PDF engine taken from the `[export] pdf_engine` config value, default `xelatex`); if the external tool is unavailable or the conversion fails, it SHALL silently fall back to the built-in simple implementations so export always succeeds without external dependencies. The status bar message for a successful PDF/DOCX export SHALL disclose which backend produced the file — the pandoc engine, or the built-in writer together with a hint that installing pandoc yields richer output. When the pandoc engine fails and the fallback is used, the status message SHALL additionally indicate the failure category (pandoc not found vs. conversion error). Export failures SHALL be reported with user-facing status messages.

#### Scenario: Engine-produced export is disclosed
- **WHEN** the user exports to PDF or DOCX and the pandoc engine succeeds
- **THEN** the status message names the output path and indicates the pandoc engine produced it

#### Scenario: Built-in fallback is disclosed with a hint
- **WHEN** the user exports to PDF or DOCX and the editor falls back to the built-in writer
- **THEN** the status message names the output path, indicates the built-in writer was used, and hints that installing pandoc improves output quality

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

## ADDED Requirements

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
