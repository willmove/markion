## MODIFIED Requirements

### Requirement: Native tables SHALL use the active document palette
Native Markdown and unstyled HTML table previews SHALL use readable table surfaces, header backgrounds, body backgrounds, borders, cell selection fills, and drop-target highlights resolved semantically from the active theme's `ThemePalette` across Visual Edit, Read, and Split Preview. Visual Edit table controls (row/column grips, edge add strips, and grip context menus including destructive actions) SHALL also resolve their normal, hover, and foreground colors from the active theme. Both dark and light built-in themes as well as complete or sparse custom `.toml` themes SHALL maintain readable foreground contrast on table surfaces and controls without mutating document state or per-version derived caches when themes switch.

#### Scenario: Dark theme table surfaces
- **WHEN** a dark theme (such as Ink, GitHub Dark, Solarized Dark, One Dark, or Tokyo Night) displays Markdown and unstyled HTML tables in Visual Edit, Read, or Split Preview
- **THEN** header, body, border, selection, and drop-target colors follow the dark theme's surface polarity with WCAG AA readable text contrast
- **AND** table controls and grip context menus in Visual Edit use the same active theme palette

#### Scenario: Light themes and custom themes stay readable
- **WHEN** a light built-in theme (such as Paper) or a complete or sparse custom `.toml` theme is active
- **THEN** Markdown tables, HTML tables, and table controls derive readable header, body, border, and hover colors from that theme's palette

#### Scenario: Live theme switching updates tables without mutating the document
- **WHEN** the user switches between light and dark themes while tables are visible in Visual Edit, Read, or Split Preview
- **THEN** all rendered tables and table controls update immediately to the new theme palette
- **AND** document text, document version, undo history, and derived caches remain unchanged
