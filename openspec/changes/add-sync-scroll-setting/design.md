# Design — Sync scroll preference

## Goal

When the user enables a persisted "Sync scroll" preference, scrolling either the source editor or the rendered preview pane in Split Preview mode moves the other pane to the same proportional position. Disabled by default; current independent scrolling is unchanged.

## Why proportional, not block-mapped

A block-to-source-line map would give perfect alignment but requires a new derived artifact (mapping each `PreviewBlock` to its source byte/line range) and extra bookkeeping against the per-version cache. That is a larger change and risks the "do not recompute derived state on every keystroke" invariant.

Proportional sync (match scroll *fraction* of scrollable range) is cache-safe: it reads only each pane's current offset and scrollable bounds, which the existing scrollbar code already computes (`scroll_handle.offset()`, `scroll_handle.max_offset()`, `list_state.scroll_px_offset_for_scrollbar()`, `list_state.max_offset_for_scrollbar()`). It is good enough for reading/navigating and is the standard approach in editors without a semantic map. This design deliberately scopes to proportional.

## Pane scroll state today

- **Editor pane**: `EditorTab::editor_scroll: ScrollHandle` (GPUI `overflow_y_scroll` + `track_scroll`). Scrollable content height comes from the measured wrapped-text height; offset is `scroll_handle.offset().y` (negative), max scroll is `scroll_handle.max_offset().height`.
- **Preview pane**: `EditorTab::preview_list: ListState` (GPUI virtualized `list`). Scroll offset is `list_state.scroll_px_offset_for_scrollbar().y` (negative); max scroll is `list_state.max_offset_for_scrollbar().height`.
- Both are per-tab, restored on tab switch (existing invariant). Sync must not break per-tab isolation.

The two scroll mechanisms are different types, so sync translates through a **fraction** in `[0,1]` rather than copying a pixel offset.

## Data flow

```
user scrolls pane A (wheel / scrollbar drag / keyboard)
  -> pane A's scroll handle / list state updates its offset (existing behavior)
  -> a sync hook observes the change and computes fraction = clamp(offset / max_scroll, 0, 1)
  -> if Sync scroll is ON and view mode is Split:
       set pane B offset = -clamp(fraction * max_scroll_B, 0, max_scroll_B)
       guard against feedback loops (re-entrant sets)
```

### Hook point: where to observe scroll changes

GPUI does not emit a "scroll changed" event for `ScrollHandle` or `ListState`. The reliable observation points are:

1. **During render** (`render` of the workspace): both scroll states are read to draw the scrollbar thumbs already. The same render pass can, *after* reading the current offsets, detect which pane drove the most recent change and apply the proportional offset to the other pane. To know "which pane changed", we cache each tab's last-applied editor/preview fractions and compare; the one whose stored fraction differs from the freshly-read fraction is the driver.

2. **At the input handlers**: `on_scroll_wheel` / the scrollbar drag handlers (`pane_scrollbar_view`, `preview_list_scrollbar_view`) and `scroll_editor_to_offset`/`scroll_editor_typewriter_to_offset` (cursor-following scroll). Tagging a "scroll source" on the app state at these points is more precise but spreads the coupling across many call sites.

**Decision: render-time reconciliation with a per-tab "last fractions" cache, plus an explicit re-entrancy guard flag.** This keeps all sync logic in one place, avoids hunting every scroll-mutating call site, and is naturally idempotent (a reconciled frame converges). The guard flag (`syncing_scroll: bool`) prevents the set-on-B from being read back next frame as a B-driven change and ping-ponging.

Reconciliation runs only when: view mode is `Split` AND `sync_scroll` preference is ON AND the document/preview has nonzero scrollable range in both panes. Otherwise it is a no-op and the panes scroll independently (current behavior).

### Per-tab state added

On `EditorTab`:
- `sync_scroll_editor_fraction: Option<f32>` — last fraction applied to the editor (None until first reconciled).
- `sync_scroll_preview_fraction: Option<f32>` — last fraction applied to the preview.

On `MarkionApp` (transient, not persisted):
- `syncing_scroll: bool` — re-entrancy guard for the current render's reconciliation.

### Fraction computation

```
editor_offset = -editor_scroll.offset().y clamped to [0, editor_max]
editor_max    = editor_scroll.max_offset().height.max(0)
editor_frac   = editor_max > 1 ? editor_offset / editor_max : 0   (clamped 0..1)

preview_offset = -preview_list.scroll_px_offset_for_scrollbar().y clamped to [0, preview_max]
preview_max    = preview_list.max_offset_for_scrollbar().height.max(0)
preview_frac   = preview_max > 1 ? preview_offset / preview_max : 0   (clamped 0..1)
```

If either `max <= 1` (content fits, nothing to scroll), sync is a no-op for that direction (the other pane can still scroll on its own).

### Applying to the other pane

- Set editor: `editor_scroll.set_offset(point(0, -(preview_frac * editor_max)))` — but only if `|editor_frac - preview_frac|` exceeds a small epsilon, to avoid fighting the user's own fine-grained scroll and to let the driving pane lead.
- Set preview: `preview_list.set_offset_from_scrollbar(point(0, -(editor_frac * preview_max)))` (the list's scrollbar-API setter, consistent with how the preview scrollbar drag drives it).

Wait — `ListState` scroll is driven during drag via `set_offset_from_scrollbar`. For wheel/keyboard scroll the list updates internally; reading via `scroll_px_offset_for_scrollbar` reflects it. Setting the list from sync reuses `set_offset_from_scrollbar`, which is the public-ish API the scrollbar already uses. (Confirmed at `preview_list_scrollbar_view`.)

## Recursion / loop safety

- The `syncing_scroll` guard: set true at the start of reconciliation, false at the end. Re-entrant reads during the same frame see it set and skip.
- The epsilon threshold (e.g. `0.001` of fraction, or ~1px equivalent) stops the converged state from re-applying every frame.
- Because we only ever *write the non-driving pane* (detected by which stored fraction changed), and then store that pane's fraction to match, the next frame sees both stored fractions equal to the driver's → no further writes. Converges in one frame.

## Preference persistence

Mirrors `preview_adaptive_width` exactly:
- `AppPreferences::sync_scroll: bool` (default `false`), `src/model.rs`.
- `PreferencesFile::sync_scroll: bool` with `#[serde(deserialize_with = "deserialize_bool_or_false")]`, `src/storage/preferences.rs`; included in both `From<&AppPreferences>` and `From<PreferencesFile>`.
- Legacy `parse_legacy_app_preferences`: handle `sync_scroll` → `parse_preference_bool`.
- `MarkionApp::sync_scroll` field; set in the workspace constructor from loaded prefs; in `current_preferences`; in `reset_preferences`.

## Preferences panel & i18n

- Add a `preference_boolean_row` for Sync scroll in the "Other" section of `preferences_panel_view`, bound to `app.sync_scroll` and a `toggle_sync_scroll(cx)` action (mirrors `toggle_preview_adaptive_width`).
- `Msg` variants: `PrefPanelSyncScroll` (label), `StatusSyncScrollOn`, `StatusSyncScrollOff`. Add `en()`/`zh()` arms and the exhaustive test lists.
- No keyboard shortcut is added (non-goal; matches other preferences like adaptive width which have no shortcut).

## Edge cases

- **Edit / Read mode**: sync is a no-op (only one pane visible). The preference still persists and the toggle still works; it just has no visible effect outside Split.
- **Content fits in one pane**: that direction's fraction is 0/undefined (max<=1) → we skip writing the other pane from it, but the other pane can still drive its own scroll (no clamp applied to it).
- **Tab switch**: stored fractions reset with the tab (they're per-tab fields, initialized None). First Split frame after switch reconciles from whatever the restored scroll positions imply. No jump because we set stored fractions *to* the read fractions on the first observed frame before writing.
- **Typewriter mode**: typewriter sets editor scroll directly (`scroll_editor_typewriter_to_offset`). That counts as an editor-driven change and will pull the preview along when sync is on — acceptable/desired.
- **Cursor-follow on find/goto**: `scroll_editor_to_offset` editor-driven → preview follows. Desired.
- **Preview parse debounce / stale blocks**: max_offset may change when blocks land. Reconciliation runs each frame so it self-corrects; no reset forced. The `ListState` splice path preserves scroll, and we only read its offset — we never `reset` it.

## Test plan

- Preference round-trip + default-false + reset (unit, `preferences.rs`/`model.rs` style tests already present).
- A pure helper `sync_fraction(offset, max) -> f32` (clamped) and a `should_apply_sync(editor_frac, preview_frac, epsilon) -> bool` decision helper, both unit-tested.
- The existing `read_mode_preview_is_constrained`-style helper tests show the pattern; add `sync_scroll_is_active(view_mode, sync_scroll) -> bool` (true only in Split) and test it across the three modes.
- No GPUI render integration test (the suite is unit-level on helpers, consistent with the rest of the codebase).
