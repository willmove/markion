## Context

`ThemePalette` previously exposed general window/panel/surface colors (`app_bg`, `panel_bg`, `surface_bg`, `text`, `muted`, `border`, `active_bg`, `active_text`) without dedicated table semantics. As a result, `src/app/preview.rs` hardcoded light hex colors in `PreviewBlock::Table`, `visual_table_view`, `html_table_grid_view`, `visual_html_editor`, `visual_table_grip`, `visual_table_edge_strip`, and `visual_table_menu_view`.

## Goals / Non-Goals

**Goals:** Resolve all table surfaces, borders, selection/drop highlights, and interactive controls centrally through `ThemePalette` so built-in dark themes, built-in light themes, and custom `.toml` themes render with readable contrast and update immediately on theme switch.

**Non-Goals:** Changing table AST/HTML parsing, changing `ThemeColors` `.toml` schema requirements for custom themes, or introducing per-table theme overrides.

## Decisions

### Centralize semantic table tokens on `ThemePalette`

Add ten semantic fields (`table_header_bg`, `table_body_bg`, `table_border`, `table_selected_bg`, `table_drop_target_bg`, `table_control_bg`, `table_control_hover_bg`, `table_control_text`, `table_control_hover_text`, `destructive_text`) computed in `ThemePalette::resolve` using theme polarity (`is_dark`), surface mixing, and luminance/contrast checks (`contrast_ratio` >= 4.5 for foreground text against table header/body/control backgrounds, distinct surface separation between header and body, and minimum border/drop-target visibility).

Data flow remains unchanged:

`ThemeDefinition / custom .toml -> ThemePalette::resolve -> app.palette() -> preview.rs GFM & HTML table views + Visual Edit controls`

Theme palette resolution happens purely at render/palette lookup time and never touches canonical document text, document version, undo history, or `Arc`-cached derived Markdown/preview blocks.

### Preserve backward compatibility for custom `.toml` themes

Custom themes continue to specify only the existing `ThemeColors` keys (`#[serde(default)]`). Both complete and sparse custom `.toml` themes inherit well-contrasted semantic table tokens automatically without requiring new TOML keys.

## Risks / Trade-offs

- [Header and body table backgrounds look identical on flat custom themes] -> `resolve_table_header_bg` blends toward `panel_bg`/`text` with polarity awareness to preserve distinct header separation while keeping `contrast_ratio(text, bg) >= 4.5`.
- [Light-theme regression] -> Keep the resolver balanced for light polarity and assert contrast and brightness polarity across all 14 built-in themes (`Paper`, `Solar`, `GitHub Light`, `Solarized Light`, `One Light`, `Tokyo Night Light`, `Ink`, `Forest`, `Rose`, `Graphite`, `GitHub Dark`, `Solarized Dark`, `One Dark`, `Tokyo Night`).

## Migration Plan

No configuration or document migration is required. Existing preferences and custom `.toml` theme files work unchanged.
