## MODIFIED Requirements

### Requirement: Configurable source-editor font size
The editor SHALL use a global source-editor font-size preference, expressed in logical pixels, for Markdown source text in Edit mode and the source pane of Split Preview mode. The default SHALL be 14px, the supported range SHALL be 10–32px inclusive, and the resolved size SHALL drive source text shaping, wrapping, painting, caret placement, selection geometry, line-height calculation, scroll extents, and typewriter positioning consistently.

#### Scenario: Source font size applies in source surfaces
- **WHEN** the user changes the source-editor font size while Edit mode or Split Preview mode is visible
- **THEN** the source text reflows immediately at the selected size
- **AND** caret, selection, scrollbar, focus-mode, and typewriter-mode geometry remain aligned with the painted text

#### Scenario: Source font size is global across tabs
- **WHEN** the user changes the source-editor font size and switches to another document tab
- **THEN** the other tab's source surface uses the same selected size without modifying either document
