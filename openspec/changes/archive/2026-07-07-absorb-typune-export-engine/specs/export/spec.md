## MODIFIED Requirements

### Requirement: Multi-format document export
The export engine SHALL export the document to Markdown, styled HTML, plain HTML, LaTeX, DOCX, PDF, and basic PNG/JPEG text snapshots, prompting the user for an output path and suggesting a filename based on the current document. The exports SHALL preserve code highlighting, math metadata/error states, and table formatting to the extent each format supports, and SHALL report failures with user-facing status messages. For PDF and DOCX, the editor SHALL first attempt the absorbed Typune export engine (pandoc subprocess); if the external tool is unavailable or the conversion fails for any reason, it SHALL silently fall back to the built-in simple implementations (single-page text PDF; minimal hand-written DOCX) so export always succeeds without external dependencies.

#### Scenario: Full-fidelity text exports
- **WHEN** the user exports to styled HTML, plain HTML, LaTeX, or DOCX
- **THEN** the export preserves headings, lists, tables (with parsed alignment for LaTeX/HTML), code blocks, math fallback, and footnote/highlight/superscript constructs as each format allows

#### Scenario: Engine-first PDF and DOCX export with pandoc available
- **WHEN** the user exports to PDF or DOCX and pandoc (plus its PDF engine, for PDF) is installed and succeeds
- **THEN** the output is produced by the Typune export engine via pandoc, preserving rich document structure beyond the built-in fallback's fidelity

#### Scenario: PDF and DOCX fallback without pandoc
- **WHEN** the user exports to PDF or DOCX and the pandoc engine path fails (tool missing or conversion error)
- **THEN** the editor silently falls back to the built-in implementation (simple single-page text PDF; minimal DOCX) and the export still succeeds

#### Scenario: Basic image snapshot export
- **WHEN** the user exports to PNG or JPEG
- **THEN** a basic text snapshot of the document is produced (a bitmap-font text rendering, not a styled rich export)

#### Scenario: Output path is chosen by the user
- **WHEN** the user triggers an export
- **THEN** the editor prompts for a save location and suggests a filename derived from the current document

#### Scenario: Export failures are reported
- **WHEN** an export step fails
- **THEN** the editor shows a user-facing status message describing the failure
