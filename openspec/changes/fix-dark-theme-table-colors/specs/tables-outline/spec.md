## MODIFIED Requirements

### Requirement: Native tables SHALL use the active document palette
Native Markdown and unstyled HTML table previews SHALL use readable table surfaces, header backgrounds, body backgrounds, borders, cell selection fills, and drop-target highlights resolved semantically from the active theme's `ThemePalette` across Visual Edit, Read, and Split Preview. Visual Edit table controls (row/column grips, edge add strips, and grip context menus including destructive actions) SHALL also resolve their normal, hover, and foreground colors from the active theme. Dark and light built-in themes SHALL maintain readable foreground contrast on table surfaces and controls. Complete or sparse custom `.toml` themes SHALL derive the same colors from their own palette, using polarity-aware fallback mixes when a theme color fails the contrast or separation checks. Theme switches SHALL NOT mutate document state or per-version derived caches.

#### Scenario: Dark theme table surfaces
- **WHEN** a dark theme (such as Ink, GitHub Dark, Solarized Dark, One Dark, or Tokyo Night) displays Markdown and unstyled HTML tables in Visual Edit, Read, or Split Preview
- **THEN** header, body, border, selection, and drop-target colors follow the dark theme's surface polarity with WCAG AA readable text contrast
- **AND** table controls and grip context menus in Visual Edit use the same active theme palette

#### Scenario: Light themes and custom themes follow their own palette
- **WHEN** a light built-in theme (such as Paper) or a complete or sparse custom `.toml` theme is active
- **THEN** Markdown tables, HTML tables, and table controls keep that theme's colors where they pass the contrast and separation checks
- **AND** use polarity-aware fallback mixes where a theme color fails those checks

#### Scenario: Live theme switching updates tables without mutating the document
- **WHEN** the user switches between light and dark themes while tables are visible in Visual Edit, Read, or Split Preview
- **THEN** all rendered tables and table controls update immediately to the new theme palette
- **AND** document text, document version, undo history, and derived caches remain unchanged
