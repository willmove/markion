## ADDED Requirements

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
