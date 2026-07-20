## ADDED Requirements

### Requirement: Configurable source-editor font size
The editor SHALL use a global source-editor font-size preference, expressed in logical pixels, for Markdown source text in Edit mode and the source pane of Split Preview mode. The default SHALL be 15px, the supported range SHALL be 10–32px inclusive, and the resolved size SHALL drive source text shaping, wrapping, painting, caret placement, selection geometry, line-height calculation, scroll extents, and typewriter positioning consistently.

#### Scenario: Source font size applies in source surfaces
- **WHEN** the user changes the source-editor font size while Edit mode or Split Preview mode is visible
- **THEN** the source text reflows immediately at the selected size
- **AND** caret, selection, scrollbar, focus-mode, and typewriter-mode geometry remain aligned with the painted text

#### Scenario: Source font size is global across tabs
- **WHEN** the user changes the source-editor font size and switches to another document tab
- **THEN** the other tab's source surface uses the same selected size without modifying either document

### Requirement: Configurable rendered-document font size
The editor SHALL use a global rendered-document font-size preference, expressed in logical pixels, as the body-text basis for Visual Edit, the preview pane of Split Preview, and Read mode. The default SHALL be 14px and the supported range SHALL be 10–32px inclusive. Headings, lists, block quotes, tables, code, source-backed Visual Edit islands, and inline/display math SHALL derive their text and line metrics from the resolved body size while preserving the current default visual proportions.

#### Scenario: Reading font size applies across rendered modes
- **WHEN** the user changes the rendered-document font size
- **THEN** Visual Edit, Split Preview's rendered pane, and Read mode use the selected body size on their next render
- **AND** dependent heading, list, quote, table, code, and math typography scales consistently

#### Scenario: Rendered selection and editing geometry follows the size
- **WHEN** rendered text wraps differently at a non-default rendered-document font size
- **THEN** preview selection, Visual Edit pointer placement, caret geometry, and inline editing remain aligned with the visible glyphs

### Requirement: Configurable rendered paragraph spacing
The editor SHALL use a global paragraph-spacing preference, expressed in logical pixels, as the bottom gap after rendered paragraph blocks in Visual Edit, Split Preview, and Read mode. The default SHALL be 12px and the supported range SHALL be 0–32px inclusive. Changing this preference MUST NOT insert, remove, or rewrite whitespace in the Markdown source, and MUST NOT add artificial paragraph gaps to the source editor.

#### Scenario: Paragraph spacing applies to rendered paragraphs
- **WHEN** the user changes rendered paragraph spacing
- **THEN** paragraph blocks in Visual Edit, Split Preview, and Read mode reflow immediately with the selected bottom gap
- **AND** non-paragraph block spacing retains its defined relationship to the selected typography

#### Scenario: Zero spacing does not change Markdown
- **WHEN** the user selects 0px paragraph spacing
- **THEN** adjacent rendered paragraph blocks have no added bottom gap
- **AND** the document text, dirty state, undo history, and authored blank lines remain unchanged

### Requirement: Typography changes preserve document and cache invariants
Applying any typography preference SHALL refresh only presentation layout and measurement state. It MUST NOT mutate a document, increment its Markdown document version, recompute per-version preview/outline/stat caches, discard memoized syntax highlighting, or replace the cached document text handle solely because typography changed. When remeasurement requires resetting virtualized rows, the editor SHALL preserve the affected pane's approximate scroll position.

#### Scenario: Typography change is presentation-only
- **WHEN** the user changes any typography preference with multiple tabs open
- **THEN** every tab retains its text, dirty state, undo/redo history, selection, and derived Markdown cache identity
- **AND** visible surfaces repaint using the new typography

#### Scenario: Long-document position survives remeasurement
- **WHEN** the user changes rendered typography while scrolled within a long Visual Edit or preview/read document
- **THEN** virtualized block heights and scrollbar extents are recomputed
- **AND** the viewport remains at approximately the same proportional document position rather than jumping to the beginning
