## Context

Launch currently hard-codes a centered 1180×760 window in `src/app/bootstrap.rs` (`Bounds::centered` + `WindowBounds::Windowed`). `MarkionApp` then always starts `sidebar_width` at `DEFAULT_SIDEBAR_WIDTH` (230) and `editor_split_ratio` at 0.5 (`src/app/application.rs`). Users can drag both dividers (`src/app/workspace.rs`) and resize/move the native window, but those values live only in memory.

Preferences already persist sidebar *visibility* and *tab* in `config.toml`. Session continuity already lives in `session.toml` (`SessionState` in `src/model.rs`, load/save in `src/storage/session.rs`). Window construction happens *before* `MarkionApp::new` loads that session, so any remembered window rectangle must be read earlier than today’s session restore.

This change does not touch Markdown documents, per-version derived caches, highlight memoization, or editor text handles. Layout I/O is chrome-only.

## Goals / Non-Goals

**Goals:**

- Restore the last windowed size and position (and maximized vs windowed) on the first frame.
- Restore the last sidebar width and Split Preview split ratio.
- Keep writes cheap and infrequent during live resize/drag.
- Survive missing files, partial TOML, and a display set that no longer contains the saved origin.

**Non-Goals:**

- Preferences-panel editors or Reset Preferences coverage for layout.
- Fullscreen, view mode, caret, selection, or scroll persistence.
- Multi-window or per-workspace layout profiles.
- Changing `sidebar_visible` / `sidebar_tab` storage.

## Decisions

### D1: Store layout on `session.toml`, not `config.toml`

Add an optional `[layout]` table to the existing session file:

```toml
[layout]
x = 120.0
y = 80.0
width = 1420.0
height = 900.0
maximized = false
sidebar_width = 280.0
editor_split_ratio = 0.42
```

All fields optional; absent or invalid values fall back to today’s defaults (centered 1180×760, sidebar 230, split 0.5). Logical pixels, matching GPUI `Pixels`.

Rationale: window origin is machine- and display-local “where I left it,” the same class as last-open files. Putting it in `config.toml` would mix it with theme/language and make Reset Preferences move the window — surprising. A third file (`ui-state.toml`) would add another atomic-write path for no gain.

Alternative rejected: Preferences keys — users do not set these in the panel, and reset semantics are wrong.

### D2: Load layout before `open_window`; apply pane geometry in `MarkionApp::new`

`window_bounds` is consumed only at window creation. Bootstrap therefore loads `session.toml` (same `default_session_path()` as today), maps `[layout]` through a GPUI-free `resolve_windowed_bounds` helper plus a thin display-aware clamp, and passes `WindowBounds::Windowed` or `WindowBounds::Maximized`.

`sidebar_width` and `editor_split_ratio` stay `MarkionApp` fields, initialized from the same loaded `SessionState` (tests keep injecting defaults and must not touch the developer session file).

CLI `StartupOpenIntent` still skips *document/workspace* restore; layout restore always runs. A CLI file open should not reset the user’s window.

### D3: Debounced persist on bounds and divider changes; flush on close

GPUI already exposes `Context::observe_window_bounds` (fires on resize; platforms typically also report moves that change bounds). Markion will:

1. Keep a `Subscription` on the root view / app entity.
2. Copy `window.window_bounds()` into `SessionState.layout` (store the *restore* rectangle from `WindowBounds::get_bounds()`, plus `maximized` when the variant is `Maximized`).
3. Arm a short debounce (about 300ms) before `save_session_state`.
4. On sidebar or split drag, update the live field as today and reuse the same debounce; double-click reset persists after the debounce or on the next flush.
5. Flush immediately when the window is allowed to close (`install_window_close_guard` already owns that path).

Do not persist `WindowBounds::Fullscreen`. If the user is fullscreen, keep the last windowed/maximized snapshot.

Writes go through the existing atomic `save_session_state` so layout and file-session fields stay one file. Updating layout MUST NOT increment document versions or rebuild derived caches.

### D4: Clamp and re-center instead of blindly restoring

Pure helper (unit-testable, no GPUI):

- `width`/`height` missing, non-finite, or below 640×480 → default size 1180×760.
- `sidebar_width` clamped to the existing 150–480 range.
- `editor_split_ratio` clamped to the existing 0.15–0.85 range.
- `maximized` missing → false.

Display-aware step (needs `App` displays): if the saved rectangle does not intersect any current display by at least a title-bar-sized strip, center the (clamped) size on the primary display and open windowed. This covers a disconnected second monitor without trapping the window off-screen.

### D5: Session write shape stays additive

`SessionFile` gains `#[serde(default)] layout: Option<LayoutFile>` with `skip_serializing_if = "Option::is_none"`. Older Markion builds ignore the table; a downgrade that re-saves will drop `[layout]` (same as every prior session-field addition). No migration of `config.toml` or `preferences.conf`.

## Data flow and caching

```
session.toml [layout]
    → bootstrap (before open_window): WindowBounds
    → MarkionApp::new: sidebar_width, editor_split_ratio
    → live: observe_window_bounds / divider drag
    → debounce → SessionState.layout → atomic save
```

No document text, version, `Arc` derived caches, highlight memo, or text-handle reuse is on this path.

## Risks / Trade-offs

- [Saved origin sits on a disconnected monitor] → Intersect-or-recenter rule in D4; never restore a rectangle with no visible intersection.
- [Bounds observer fires every frame during drag] → 300ms debounce; close path flushes the last value.
- [Linux/Wayland reports inner vs outer bounds inconsistently] → Persist `window.window_bounds()` (the same type used at open) rather than widget `bounds()`; accept small chrome deltas across platforms.
- [Unarchived `persist-session-and-recent-files` also owns `session.toml`] → Additive `[layout]` table only; no rename of existing keys.
- [Reset Preferences users expect the window to shrink] → Explicitly out of scope; divider double-click still resets width/split locally and will persist that reset.

## Migration Plan

Additive `[layout]` on an existing file. First launch after upgrade writes the table once the user moves or resizes anything (or on close). Rollback is revert; leftover `[layout]` is ignored by older builds.

## Open Questions

- Whether some Linux WMs fire `observe_window_bounds` on move as well as resize. If move is silent, flush-on-close still captures the last position; implementation can add a close-time read as the guarantee.
