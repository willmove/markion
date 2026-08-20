# document-typography Specification

## Purpose
TBD - created by archiving change customize-document-typography. Update Purpose after archive.
## Requirements
### Requirement: Configurable source-editor font size
The editor SHALL use a global source-editor font-size preference, expressed in logical pixels, for Markdown source text in Edit mode and the source pane of Split Preview mode. The default SHALL be 14px, the supported range SHALL be 10–32px inclusive, and the resolved size SHALL drive source text shaping, wrapping, painting, caret placement, selection geometry, line-height calculation, scroll extents, and typewriter positioning consistently.

#### Scenario: Source font size applies in source surfaces
- **WHEN** the user changes the source-editor font size while Edit mode or Split Preview mode is visible
- **THEN** the source text reflows immediately at the selected size
- **AND** caret, selection, scrollbar, focus-mode, and typewriter-mode geometry remain aligned with the painted text

#### Scenario: Source font size is global across tabs
- **WHEN** the user changes the source-editor font size and switches to another document tab
- **THEN** the other tab's source surface uses the same selected size without modifying either document

### Requirement: Configurable rendered-document font size
The editor SHALL use a global rendered-document font-size preference, expressed in logical pixels, as the body-text basis for Visual Edit, the preview pane of Split Preview, and Read mode. The default SHALL be 14px and the supported range SHALL be 10–32px inclusive. Headings, lists, block quotes, tables, code, Visual Edit surfaces (including rendered editors, progressive-reveal runs, and transitional source views for WYSIWYG coverage gaps), and inline/display math SHALL derive their text and line metrics from the resolved body size while preserving the current default visual proportions.

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

### Requirement: Configurable document font families

The editor SHALL resolve a font family per document plane from three independent slots: a **source** slot for the Markdown source editor surface, a **rendered** slot for Visual Edit, Split Preview's rendered pane, and Read mode body text (including inline code spans), and a **code** slot for fenced code blocks, Visual Edit source islands, and reference-definition source views. Each slot SHALL resolve independently as: an explicit per-slot user preference, when set, over the active theme's font for that slot, when present, over the built-in default. The built-in defaults SHALL be the platform system UI font for the source and rendered slots and "JetBrains Mono" for the code slot, and the code slot SHALL carry a monospace fallback chain so code text never degrades to a proportional font when the primary family is unavailable. Application chrome (menus, sidebar, tab bar, status bar, panels) SHALL NOT follow any slot and SHALL keep the platform system UI font.

#### Scenario: An explicit preference overrides the theme

- **WHEN** the active theme defines a font for a slot and the user has also set an explicit font-family preference for that slot
- **THEN** that document plane renders with the preference's family

#### Scenario: Theme fonts apply when no preference is set

- **WHEN** the user has not set a font-family preference for a slot and the active theme defines a font for that slot
- **THEN** that document plane renders with the theme's family, without requiring any preference interaction

#### Scenario: Defaults apply with no preference and no theme font

- **WHEN** neither a preference nor the active theme defines a font for a slot
- **THEN** the source and rendered planes use the platform system UI font and code surfaces use "JetBrains Mono"
- **AND** the rendered appearance matches the pre-change behavior for documents and themes that never opted in

#### Scenario: Code keeps a monospace fallback

- **WHEN** the resolved code-slot family is not installed on the machine
- **THEN** code text renders with the first available monospace fallback family rather than silently degrading to a proportional font

#### Scenario: Font changes are presentation-only and re-measure layout

- **WHEN** any slot's resolved family changes, via a preference edit or a theme switch
- **THEN** every tab retains its text, dirty state, undo/redo history, selection, and derived Markdown cache identity
- **AND** layout measurements for affected surfaces recompute under the new family, including when the font size is unchanged, so cached heights keyed before the change are not reused
- **AND** the affected pane keeps its approximate scroll position

#### Scenario: Inline code follows the rendered slot

- **WHEN** the rendered slot resolves to a family different from the platform default
- **THEN** inline code spans in rendered surfaces draw with that family (code-block surfaces still follow the code slot)

#### Scenario: Chrome keeps the system font

- **WHEN** any slot is set to a family different from the platform system UI font
- **THEN** application chrome text (menus, sidebar, tab bar, preferences panel, status bar) still renders in the platform system UI font
