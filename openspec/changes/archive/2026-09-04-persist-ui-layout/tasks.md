## 1. Session layout model

- [x] 1.1 Add a `SessionLayout` (or equivalent) value type on `SessionState` for optional window origin/size, maximized flag, sidebar width, and editor split ratio, with built-in defaults matching today’s 1180×760 / 230 / 0.5 values
- [x] 1.2 Extend `src/storage/session.rs` with an optional `[layout]` table (all fields optional, unknown keys ignored) and keep the existing atomic write path
- [x] 1.3 Add a GPUI-free clamp/normalize helper for size floors (640×480), sidebar 150–480, and split 0.15–0.85
- [x] 1.4 Add unit tests for missing `[layout]`, partial/invalid fields, round-trip, and clamp behavior without touching the developer `session.toml`

## 2. Restore on launch

- [x] 2.1 Load `session.toml` in bootstrap before `cx.open_window` and map `[layout]` to `WindowBounds::Windowed` or `WindowBounds::Maximized`
- [x] 2.2 Re-center on the primary display when the saved rectangle does not intersect any current display; fall back to the centered 1180×760 default when size is missing or invalid
- [x] 2.3 Initialize `MarkionApp.sidebar_width` and `editor_split_ratio` from the loaded session in `MarkionApp::new` (tests still use in-memory defaults and must not read/write the real session file)
- [x] 2.4 Keep CLI file/folder open-intent skipping document/workspace restore while still applying `[layout]`

## 3. Persist while running

- [x] 3.1 Subscribe to `observe_window_bounds`, copy restore bounds plus maximized state into `SessionState.layout`, and debounce disk writes (~300ms); do not persist fullscreen
- [x] 3.2 Persist sidebar width and split ratio through the same debounce after divider drags and after double-click resets
- [x] 3.3 Flush the latest layout immediately on the existing clean-close path in `install_window_close_guard`
- [x] 3.4 Confirm Reset Preferences still does not clear `[layout]`; confirm layout I/O does not bump document versions or rebuild derived caches

## 4. Verification

- [x] 4.1 Run the root-package tests covering session parse/round-trip and the new clamp/fallback helpers; fix regressions
- [x] 4.2 Manually confirm: resize and move the window, widen the sidebar, drag the split, quit, relaunch; then unplug/off-screen case or invalid TOML still starts
- [x] 4.3 Run `openspec validate persist-ui-layout` and leave tasks checked only when the corresponding work is done
