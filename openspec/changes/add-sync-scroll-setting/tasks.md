## 1. Preference Model and Persistence

- [x] 1.1 Add a `sync_scroll: bool` field (default `false`) to `AppPreferences` in `src/model.rs`.
- [x] 1.2 Add `sync_scroll` to `PreferencesFile` in `src/storage/preferences.rs` with `#[serde(deserialize_with = "deserialize_bool_or_false")]`, and wire it through both `From<&AppPreferences>` and `From<PreferencesFile>`.
- [x] 1.3 Handle `sync_scroll` in `parse_legacy_app_preferences` via `parse_preference_bool`.
- [x] 1.4 Add/extend preference round-trip, default-false, invalid-value-falls-back, and legacy-migration tests for the new field.

## 2. App State, Reset, and Snapshot

- [x] 2.1 Add a `sync_scroll: bool` field to `MarkionApp` in `src/main.rs`, initialized from loaded preferences.
- [x] 2.2 Include `sync_scroll` in `current_preferences()`.
- [x] 2.3 Set `sync_scroll` from the reset preferences in `reset_preferences`.
- [x] 2.4 Add a `toggle_sync_scroll(cx)` action that flips the field, sets localized status, persists, and notifies (mirror `toggle_preview_adaptive_width`).

## 3. Scroll Coupling (Split Preview only)

- [x] 3.1 Add pure helpers `sync_fraction(offset: f32, max: f32) -> f32` (clamped 0..1, 0 when max <= 1) and `sync_scroll_is_active(view_mode: ViewMode, sync_scroll: bool) -> bool` (true only for Split + enabled).
- [x] 3.2 Add per-tab fields `sync_scroll_editor_fraction: Option<f32>` and `sync_scroll_preview_fraction: Option<f32>` to `EditorTab`; reset them on `reset_preview_list` and new-tab creation.
- [x] 3.3 Add a transient `syncing_scroll: bool` re-entrancy guard to `MarkionApp`.
- [x] 3.4 Implement a render-time reconciliation step (run only when `sync_scroll_is_active`) that: reads editor fraction from `editor_scroll` and preview fraction from `preview_list` (via the scrollbar-API getters), determines the driving pane by which stored fraction changed, writes the proportional offset to the other pane (`editor_scroll.set_offset` / `preview_list.set_offset_from_scrollbar`), updates both stored fractions, and guards against re-entrancy and a small epsilon to avoid feedback loops.
- [x] 3.5 Ensure the reconciliation is a no-op when either pane's scrollable range is <= 1px, and never calls `reset`/`splice` on the preview list or triggers a reparse.

## 4. Preferences Panel

- [x] 4.1 Add a Sync scroll `preference_boolean_row` to the "Other" section of `preferences_panel_view`, bound to `app.sync_scroll` and `toggle_sync_scroll`.
- [x] 4.2 Confirm the panel reflects the persisted state on open and updates immediately on toggle.

## 5. Localization

- [x] 5.1 Add `Msg` variants `PrefPanelSyncScroll`, `StatusSyncScrollOn`, `StatusSyncScrollOff` in `src/i18n.rs`.
- [x] 5.2 Add English and Simplified Chinese translations for the new variants in `en()`/`zh()`.
- [x] 5.3 Add the new variants to the exhaustiveness-guard test lists so a missing translation fails the build.

## 6. Verification

- [x] 6.1 Add focused unit tests for `sync_fraction` (clamping/zero-max), `sync_scroll_is_active` across the three view modes, and preference round-trip/default/reset/migration.
- [x] 6.2 Run `cargo test`.
- [x] 6.3 Run `openspec validate add-sync-scroll-setting`.
