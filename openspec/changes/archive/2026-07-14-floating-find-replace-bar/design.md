## Context

Find / Replace is currently rendered by `search_panel_view` as a normal child between the tab bar and the main content row. That makes the bar full-width, consumes vertical layout space, and pushes the editor/preview panes downward whenever search opens. The same view and its helper buttons also use hard-coded light colors instead of the active `ThemePalette`.

The existing search state and behavior are already sufficient: `search_visible`, `replace_visible`, `search_query`, `replace_text`, `search_focus`, match refresh, next/previous navigation, and replace actions all live in `MarkionApp`. This change should preserve that state model and move only the chrome presentation and dismissal behavior.

Data flow:

```text
Find / Replace actions
        |
        v
MarkionApp search state
        |
        v
refresh_search_matches()
        |
        v
floating search overlay render
```

Closing flow:

```text
Close button / dismiss path
        |
        v
search_visible = false
replace_visible = false
search_focus = None
refresh_search_matches()
        |
        v
match highlights cleared; query text retained
```

No Markdown derived-state cache, syntax highlighting cache, or cached text handle should be affected.

## Goals / Non-Goals

**Goals:**

- Render Find / Replace as a compact, upper-right floating overlay above the editor/preview workspace.
- Ensure opening the overlay does not change the height or position of the tab bar, editor pane, preview pane, or status bar.
- Provide an explicit close control that hides the overlay, removes active match highlighting, and clears search focus.
- Preserve the current query and replacement buffers across close/open.
- Use the active `ThemePalette` for the overlay surface, borders, text, muted summary, active field border, buttons, and hover states.
- Keep existing shortcuts and search/replace behavior intact.

**Non-Goals:**

- Do not change the search algorithm, regex semantics, replacement behavior, or match selection rules.
- Do not add a persisted preference for overlay position, size, or visibility.
- Do not introduce a new theming schema or external dependency.
- Do not change document content, preview parsing, or derived-state caching.

## Decisions

### Use an absolute-positioned root overlay

Move the search UI out of the flex column where it currently occupies a row. Render it as an `.absolute()` child of the root `.relative()` container, positioned near the upper-right edge of the workspace.

Rationale: GPUI already uses root-level absolute children for menus and panels, and this approach avoids resizing the main content row. It keeps the implementation local to `src/main.rs`.

Alternative considered: keep the panel in layout but reduce its width and align it right. That would still consume height and shift the workspace, so it does not meet the core requirement.

### Keep the existing search state

Reuse `search_visible`, `replace_visible`, `search_focus`, `search_query`, `replace_text`, match state, and existing action handlers. Add a small dismissal path rather than introducing a separate overlay model.

Rationale: The requested change is presentation and dismissal, not search semantics. Keeping the existing state reduces regression risk for keyboard shortcuts, match counts, regex handling, and replace actions.

Alternative considered: introduce a dedicated `SearchPanelState` struct. That could be cleaner later, but it is unnecessary for this scoped change and would broaden the refactor.

### Close by hiding the overlay and refreshing matches

The close control should set `search_visible = false`, `replace_visible = false`, and `search_focus = None`, then call `refresh_search_matches()`. The current query and replacement text should remain in memory so reopening search resumes the prior input.

Rationale: `refresh_search_matches()` already clears matches when `search_visible` is false, which removes highlights without special editor rendering logic.

Alternative considered: clear `search_query` on close. That would clear highlights but makes quick reopen less useful and differs from common editor behavior.

### Theme all search chrome through `ThemePalette`

Update `search_panel_view`, `search_field_view`, and the search toolbar button helper to accept or resolve `ThemePalette`. Replace hard-coded light colors with `palette.panel_bg`, `palette.surface_bg`, `palette.text`, `palette.muted`, `palette.border`, `palette.active_bg`, and `palette.active_text`.

Rationale: Find / Replace is application chrome and should follow built-in and custom themes, especially dark palettes.

Alternative considered: add dedicated search colors to `ThemeColors`. That would require a theme schema change and migration surface for a small piece of chrome; existing palette roles are enough.

### Keep visible text localized and prefer a symbol-only close control

Continue using existing `Msg::SearchFind`, `Msg::SearchReplace`, `Msg::SearchPrev`, `Msg::SearchNext`, `Msg::SearchAll`, and search option labels. Use a symbol-only close control (`×`) unless implementation adds visible text, in which case the new text must go through `src/i18n.rs`.

Rationale: The existing i18n layer already covers the visible search labels. A symbol-only close button avoids adding translation work for a universal control.

## Risks / Trade-offs

- [Risk] The overlay could cover content near the top-right of the editor or preview. → Mitigation: keep it compact, right aligned with margin, and only visible while search is active; preserve scrolling and editing under it.
- [Risk] Small windows may not have enough width for Replace mode. → Mitigation: give the overlay responsive constraints, allow wrapping or a reduced width, and ensure text/buttons do not overflow.
- [Risk] Close behavior might leave stale highlights if matches are not refreshed. → Mitigation: route all close paths through one helper that hides search and calls `refresh_search_matches()`.
- [Risk] Hard-coded colors could remain in helper functions. → Mitigation: change the search-specific field/button helpers to require `ThemePalette` and use palette roles throughout.
- [Risk] Escape handling can conflict with file-tree filter dismissal. → Mitigation: if Escape is wired to close search, preserve the existing priority where name prompts cancel first, then search overlay closes, then file-tree filtering clears.

## Migration Plan

No data migration is required. The change is UI-only and can be rolled back by rendering `search_panel_view` in the original flex position and removing the close/dismiss helper.

## Open Questions

- None. The overlay should use a fixed upper-right placement with responsive width constraints rather than a configurable position.
