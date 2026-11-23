## Context

The fix for #27 shipped in v0.6.2. Native table surfaces now use the active theme. Drag targets still use a light blue fill, selection can match the surrounding cells, and destructive menu actions and collapsible source chrome still have hardcoded colors.

## Design

Resolve `table_header_bg`, `table_body_bg`, `table_selected_bg`, and `table_drop_target_bg` from the active theme. Use `control_*` and `destructive_text` for shared borders, source editor chrome, and menu controls. The resolver prefers theme colors that pass the contrast, polarity, and separation checks. Fixed fallback mixes preserve a warm selection in Solarized Light and a blue selection in GitHub Dark.

Cache the resulting `ThemePalette` on `MarkionApp`. Resolve it at launch, on selection or reset, and when reloading custom theme definitions changes the active colors or polarity. Views read the cached palette directly. Theme changes leave document text, undo history, and derived Markdown state untouched.

Custom theme files use the existing `ThemeColors` keys. The palette assertions cover the 14 built-ins and full and sparse custom examples.
