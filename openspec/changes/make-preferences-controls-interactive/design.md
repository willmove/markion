## Context

The Preferences panel already owns immediate, persisted setting changes for theme, language, and Preview adaptive width. The same app state and persistence path also exists for focus mode, typewriter mode, code line numbers, sidebar visibility, and sidebar tab, but those panel rows currently read as a summary rather than controls. The in-window menu bar is rendered by the app rather than the native OS menu, so it must draw from the active theme palette just like the editor panes.

This change affects chrome and settings state only. Markdown derived-state caches, syntax highlighting memoization, and cached text handles are not involved.

## Goals / Non-Goals

**Goals:**

- Make the Preferences panel the editable surface for every already-supported panel preference.
- Use button-like controls for boolean values so users can tell the row is actionable.
- Put Language before Theme in the panel flow.
- Make the in-window menu bar and dropdown contrast correctly on dark and light themes.

**Non-Goals:**

- Add new preference fields or change the TOML schema.
- Change native OS menus.
- Change Markdown rendering, parsing, typing-path caches, or file tree virtualization.

## Decisions

1. Reuse existing app actions and persistence helpers for preferences.

   The app already has methods that toggle focus mode, typewriter mode, code line numbers, Preview adaptive width, sidebar visibility, and sidebar tab, and those paths update app state plus persist to `config.toml`. The panel should call those paths or small shared helpers rather than creating a second persistence flow. Alternative considered: mutate `AppPreferences` directly from the panel; rejected because it risks diverging status text, menu closing, and render invalidation behavior.

2. Keep boolean controls visually compact and theme-colored.

   Boolean settings should render as small segmented or pill-style buttons using the active theme's `active_bg`, `active_text`, `surface_bg`, `muted`, and `border` colors. This keeps the current dense Preferences panel but turns passive text into controls. Alternative considered: checkboxes; rejected for now because existing Preview adaptive width already established a button-like row pattern.

3. Treat sidebar tab as an option group under the Sidebar row.

   Sidebar visibility is boolean; active tab is a small mutually exclusive choice between Files and Outline. Keeping both in the same area matches the existing persisted data model and avoids adding a new section.

4. Theme the in-window menu from `theme_colors()`.

   Menu bar backgrounds, top-level menu labels, dropdown surfaces, borders, hover/active states, separators, and disabled/muted text should use the active theme palette. This keeps dark themes readable without adding hard-coded dark-theme branches. Alternative considered: infer `is_dark` and use two static menu palettes; rejected because custom themes can be light or dark with arbitrary colors.

## Risks / Trade-offs

- [Risk] GPUI button styling can accidentally increase row height or make the Preferences panel feel crowded. -> Mitigation: use fixed-height, compact controls and preserve existing section spacing.
- [Risk] Directly invoking existing toggle handlers from the panel can close menus or status text unexpectedly. -> Mitigation: use shared helper methods for state changes where necessary and only close `active_menu`, not the Preferences panel.
- [Risk] Some custom theme palettes may have low contrast in menu controls. -> Mitigation: rely on the same active/background/text palette already used by the rest of the app, with active states using `active_bg` and `active_text`.

## Migration Plan

No data migration is required. Existing preferences continue to load and save through the current TOML schema. Rollback is a UI-only revert because persisted values and field names remain unchanged.

## Open Questions

None.
