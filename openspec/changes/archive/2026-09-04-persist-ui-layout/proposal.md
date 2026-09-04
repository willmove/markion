## Why

Markion already remembers theme, sidebar visibility, and last-opened files, but every launch still opens a fixed 1180×760 centered window with a 230px sidebar and a 50/50 editor split. Users who resize the window, move it to another monitor, or widen the Files panel lose that layout on restart. Remembering chrome geometry restores the desktop habit of “leave it where I put it.”

## What Changes

- Persist the last windowed size and screen position, plus whether the window was maximized, and restore them on the next launch.
- Persist the live sidebar width and the Split Preview editor/preview divider ratio, and restore them with the same launch path.
- Store this chrome geometry in `session.toml` (a `[layout]` table), not `config.toml`, so it stays machine-local and is not wiped by Reset Preferences.
- Apply saved window bounds at `open_window` time; clamp or re-center when the saved rectangle is off every current display or below the existing size floors.
- Debounce writes while the user is dragging the window or a divider so resize/move does not rewrite disk on every pointer event.

**Non-goals:** no Preferences-panel controls for these values; no remembering fullscreen, view mode, cursor, selection, or scroll; no per-monitor DPI profiles beyond clamping to the current display set; no multi-window sessions; no change to sidebar visibility/tab preferences already in `config.toml`.

Invariants preserved: per-document derived-state caches, syntax-highlight memoization, cached editor text handles, bounded file-tree rendering, and GPUI-free workspace members. Layout I/O MUST NOT recompute Markdown derived state.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chrome-platform`: The desktop window SHALL restore its last windowed bounds (and maximized state) and the last sidebar width / Split Preview split ratio across launches, with safe fallbacks when the saved geometry is missing or no longer visible.
- `workspace`: The existing `session.toml` snapshot SHALL also carry an optional `[layout]` table for chrome geometry, loaded independently of CLI open-intent overrides that skip document/workspace restore.

## Impact

- Persistence: extend `SessionState` / `src/storage/session.rs` with optional layout fields; keep `config.toml` and Reset Preferences unchanged for these values.
- Bootstrap: load layout before `cx.open_window` so the first frame uses the saved window rectangle instead of the hardcoded centered default.
- App chrome: apply sidebar width and split ratio at construction; persist them on divider settle and persist window bounds via GPUI `observe_window_bounds` (debounced) plus clean close.
- Tests: parse/round-trip, clamp/fallback helpers, and “missing `[layout]` keeps today’s defaults.”
- No new crates, network, or user-facing copy unless a restore failure needs a status string (then through `src/i18n.rs`).
