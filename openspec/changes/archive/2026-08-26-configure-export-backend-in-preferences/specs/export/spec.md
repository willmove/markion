## MODIFIED Requirements

### Requirement: Multi-format document export
The export engine SHALL export the document to Markdown, styled HTML, plain HTML, LaTeX, DOCX, PDF, and basic PNG/JPEG text snapshots, prompting the user for an output path and suggesting a filename based on the current document. For PDF and DOCX, the producing implementation SHALL be selected by the `[export] backend` preference: `builtin` (the default) SHALL write the file directly through the built-in PDF writer / built-in DOCX writer without spawning any pandoc subprocess, while `pandoc` SHALL first attempt the absorbed Typune export engine (pandoc subprocess, with the PDF engine taken from the `[export] pdf_engine` config value, default `xelatex`) and silently fall back to the built-in implementation when the external tool is unavailable or the conversion fails, so export always succeeds without external dependencies. The status bar message for a successful PDF/DOCX export SHALL disclose which backend produced the file. When the backend preference is `builtin`, the built-in-writer message SHALL be neutral and SHALL NOT hint that installing pandoc yields richer output. When the `pandoc` preference falls back to the built-in writer, the status message SHALL retain the hint that installing pandoc yields richer DOCX output (PDF stays neutral because the built-in PDF writer is the rich default) and SHALL additionally indicate the failure category (pandoc not found vs. conversion error). Export failures SHALL be reported with user-facing status messages.

#### Scenario: Engine-produced export is disclosed
- **WHEN** the backend preference is `pandoc` and the user exports to PDF or DOCX with the pandoc engine succeeding
- **THEN** the status message names the output path and indicates the pandoc engine produced it

#### Scenario: Built-in preference exports without pandoc
- **WHEN** the backend preference is `builtin` (or unset) and the user exports to PDF or DOCX
- **THEN** the file is produced by the built-in writer, no pandoc subprocess is spawned, and the status message names the output path and indicates the built-in writer neutrally without hinting that installing pandoc improves output

#### Scenario: Built-in fallback is disclosed with a hint
- **WHEN** the backend preference is `pandoc` and the DOCX export falls back to the built-in writer
- **THEN** the status message names the output path, indicates the built-in writer was used, and hints that installing pandoc improves output quality

#### Scenario: Built-in PDF export is disclosed neutrally
- **WHEN** PDF is exported through the built-in writer, whether by preference or by fallback
- **THEN** the status message names the output path and indicates the built-in PDF engine produced it, without hinting that installing pandoc improves output quality

#### Scenario: Engine failure category is disclosed
- **WHEN** the backend preference is `pandoc` and the pandoc engine fails (binary missing or conversion error) so the fallback produces the file
- **THEN** the status message indicates which failure category occurred

#### Scenario: PDF engine is configurable via the config file
- **WHEN** `[export] pdf_engine` is set in `config.toml` (e.g. `"pdfroff"`, `"tectonic"`)
- **THEN** the pandoc invocation for PDF export uses that engine instead of the default `xelatex`

#### Scenario: Full-fidelity text exports
- **WHEN** the user exports to styled HTML, plain HTML, LaTeX, or DOCX
- **THEN** the export preserves headings, lists, tables (with parsed alignment for LaTeX/HTML), code blocks, math fallback, and footnote/highlight/superscript constructs as each format allows

#### Scenario: PDF and DOCX fallback without pandoc
- **WHEN** the backend preference is `pandoc` and the engine path fails (tool missing or conversion error)
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

### Requirement: User-facing DOCX export options
DOCX export options SHALL be configured in the Preferences panel rather than at export time: page size (A4, Letter, Legal), table of contents on the pandoc engine path (default off), and image embedding policy (embed local images vs. text fallback, default embed). Triggering a DOCX export SHALL go directly to the save-path prompt and apply the stored options. Both the pandoc engine path and the built-in writer SHALL honor the applicable options. The options SHALL persist across sessions via the `[export.docx]` config section.

#### Scenario: Options reach the engine path
- **WHEN** the backend preference is `pandoc`, pandoc is available, and the table of contents option is enabled
- **THEN** the pandoc invocation includes `--toc`

#### Scenario: Options reach the built-in writer
- **WHEN** the backend preference is `builtin` with Letter page size selected
- **THEN** the built-in writer's `w:sectPr` uses Letter dimensions instead of the A4 default

#### Scenario: Options reach the fallback path
- **WHEN** the backend preference is `pandoc`, the engine fails, and Letter page size is selected
- **THEN** the built-in fallback writer's `w:sectPr` uses Letter dimensions instead of the A4 default

#### Scenario: Image policy is honored
- **WHEN** the user selects the text-fallback image policy
- **THEN** local images export as `alt: url` text on both backends instead of being embedded

#### Scenario: Options persist across sessions
- **WHEN** the user changes a DOCX export option and later restarts the app
- **THEN** the Preferences panel presents the previously used options and the next export applies them

#### Scenario: Export goes straight to the save prompt
- **WHEN** the user triggers a DOCX export
- **THEN** no per-export options dialog precedes the save-path prompt

### Requirement: User-facing PDF export options
PDF export options SHALL be configured in the Preferences panel and persisted in the `[export.pdf]` config section: page size (A4, Letter, Legal; default A4), page margin in millimetres (default 25), table of contents (default off), and page-number footer (default on). The built-in writer SHALL apply all four options; the pandoc engine path SHALL map page size to `--variable=geometry:` and the table of contents to `--toc`. Unknown or missing values SHALL fall back to the defaults.

#### Scenario: Options reach the built-in writer
- **WHEN** Letter page size and a 20 mm margin are configured
- **THEN** the built-in PDF uses Letter geometry with 20 mm margins

#### Scenario: Table of contents is emitted
- **WHEN** the table-of-contents option is enabled and the document contains headings
- **THEN** the PDF opens with an outline page listing the headings with page numbers

#### Scenario: Options reach the pandoc engine path
- **WHEN** the pandoc engine runs with Legal page size and the table of contents enabled
- **THEN** the invocation includes the Legal geometry variable and `--toc`

### Requirement: Configurable pandoc binary path
The pandoc binary location SHALL be configurable via `[export] pandoc_path` in `config.toml` and through the Preferences panel Export tab (a file picker plus a reset action). When unset, the engine locates `pandoc` on the system PATH as today.

#### Scenario: Configured pandoc path is used
- **WHEN** `[export] pandoc_path` names an executable
- **THEN** the DOCX/PDF engine invocations use that executable instead of a PATH lookup

#### Scenario: Pandoc path is editable in the Preferences panel
- **WHEN** the user picks a pandoc binary through the Export tab's file picker
- **THEN** subsequent engine invocations use that path, the choice persists, and resetting returns to the PATH lookup
