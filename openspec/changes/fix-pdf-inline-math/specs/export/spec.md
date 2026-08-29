## MODIFIED Requirements

### Requirement: Built-in PDF writer renders math
The built-in PDF writer SHALL render valid inline and display math as vector graphics through the same GPUI-free math renderer used by native preview and HTML export, embedded as SVG, so exported formulas match the preview. Inline math SHALL participate in prose layout as a single measured atom aligned to the surrounding text baseline and SHALL NOT split across a line break. When the math renderer rejects a formula, the writer SHALL emit the byte-identical authored LaTeX source in code styling — a code-styled block for display math, a code-styled in-flow run for inline math — and the export SHALL still succeed.

#### Scenario: Display math matches the preview
- **WHEN** a document contains a valid `$$`-fenced formula
- **THEN** the PDF embeds the same sanitized SVG the preview renders, as a display equation

#### Scenario: Inline math matches the preview
- **WHEN** a paragraph contains a valid `$…$` formula mixed with surrounding prose
- **THEN** the PDF embeds the same sanitized SVG the preview renders, as a baseline-aligned inline atom rather than the authored `$…$` source as code-styled text

#### Scenario: Unrenderable math preserves its source
- **WHEN** the math renderer rejects a formula
- **THEN** the PDF contains the byte-identical authored LaTeX in code styling and the export succeeds
