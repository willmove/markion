## ADDED Requirements

### Requirement: Configurable code font size
The editor SHALL use a global, optional code font-size preference, expressed in logical pixels, for fenced code blocks across Read mode, Split Preview's rendered pane, and the Visual Edit direct code editor. When the preference is absent, code text and line metrics SHALL derive from the rendered-document font size exactly as before this change (12px at the 14px default, preserving the current proportions). When set, the value SHALL normalize to 10–32px inclusive and code line height SHALL scale proportionally with the resolved code size. Changing or clearing the code font size SHALL apply immediately to every rendered code-block surface and MUST remain presentation-only: no document mutation, no document-version increment, no derived-cache rebuild, no memoized-highlighting invalidation, and approximate scroll positions preserved across remeasurement.

#### Scenario: Explicit code size applies across code surfaces
- **WHEN** the user sets an explicit code font size
- **THEN** fenced code blocks in Read mode, Split Preview's rendered pane, and the Visual Edit direct code editor reflow at that size on the next render
- **AND** code line height scales proportionally

#### Scenario: Absent preference follows the reading size
- **WHEN** `config.toml` omits the code font-size key
- **THEN** code text and line metrics derive from the rendered-document font size exactly as before this change
- **AND** changing the reading font size continues to rescale code proportionally

#### Scenario: Out-of-range values clamp
- **WHEN** a stored code font size is below 10 or above 32
- **THEN** the resolved size clamps to the nearest bound

#### Scenario: Code size changes are presentation-only
- **WHEN** the user changes or clears the code font size with multiple tabs open
- **THEN** every tab retains its text, dirty state, undo/redo history, selection, and derived Markdown cache identity
- **AND** visible code surfaces repaint using the new size while keeping their approximate scroll position

#### Scenario: Clearing returns to the derived size
- **WHEN** the user clears an explicit code font size
- **THEN** code blocks return to deriving their size from the rendered-document font size on the next render

## MODIFIED Requirements

### Requirement: Configurable rendered-document font size
The editor SHALL use a global rendered-document font-size preference, expressed in logical pixels, as the body-text basis for Visual Edit, the preview pane of Split Preview, and Read mode. The default SHALL be 14px and the supported range SHALL be 10–32px inclusive. Headings, lists, block quotes, tables, Visual Edit surfaces (including rendered editors, progressive-reveal runs, and transitional source views for WYSIWYG coverage gaps), and inline/display math SHALL derive their text and line metrics from the resolved body size while preserving the current default visual proportions. Fenced code blocks SHALL instead follow the configurable code font size, whose absent-preference default continues to derive from the resolved body size (see the Configurable code font size requirement).

#### Scenario: Reading font size applies across rendered modes
- **WHEN** the user changes the rendered-document font size
- **THEN** Visual Edit, Split Preview's rendered pane, and Read mode use the selected body size on their next render
- **AND** dependent heading, list, quote, table, and math typography scales consistently
- **AND** code follows the code font-size preference, which still tracks the body size when no explicit code size is set

#### Scenario: Rendered selection and editing geometry follows the size
- **WHEN** rendered text wraps differently at a non-default rendered-document font size
- **THEN** preview selection, Visual Edit pointer placement, caret geometry, and inline editing remain aligned with the visible glyphs
