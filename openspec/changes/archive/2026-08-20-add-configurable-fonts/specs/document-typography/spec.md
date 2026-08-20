## ADDED Requirements

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
