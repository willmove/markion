## Context

See `proposal.md` for motivation. The native window is created in `src/app/bootstrap.rs` with a static `TitlebarOptions.title` of `"Markion"`; nothing later calls `Window::set_window_title`. The OS therefore draws the window icon (the Markion logo) plus the brand text, and never the open file.

The bottom status bar’s left region is a single formatted string from `status_bar_feedback` in `src/app/status_bar.rs`: `Markion - {title}{optional " *"} | {save_state} | {status}`. Image tabs skip save-state and use `Markion - {title} | {status}`. Tab titles already come from `title_from_path` (`Untitled.md` when unsaved). Dirty state is the document flag `is_dirty()`, not a Markdown-derived cache.

The in-window menu bar sits *below* the native title bar; this change does not redraw chrome or hide the system title bar.

## Goals / Non-Goals

**Goals:**

- Compose a testable native window-title string from the active tab’s existing title and dirty flag, and apply it through GPUI’s `Window::set_window_title`.
- Keep the filename immediately after the brand text so the OS title bar reads as logo, then “Markion”, then the file name.
- Remove brand, filename, and dirty-marker duplication from the status-bar feedback region without changing persistent context (counts, caret, Git branch) or transient operation messages.
- Leave Markdown per-version caches, syntax memoization, and text-handle reuse untouched.

**Non-Goals:**

- Client-side decorations, custom-drawn title bars, or moving window controls.
- Showing the full filesystem path, workspace name, or view mode in the title.
- Changing document-tab labels, dirty dots on tabs, or localized untitled-document names.
- Translating the brand token `"Markion"` or interpolating filenames through i18n.

## Decisions

### 1. Update the native window title instead of drawing a second in-window title

Keep the existing system title bar (logo + caption + platform window controls). Format the caption as `Markion - {tab_title}` and, for a dirty *document* tab, append ` *` (space then asterisk), matching today’s status-bar identity fragment. Image tabs use the image file name and never a dirty suffix.

The OS remains responsible for layout (left-aligned on Windows, typically centered on macOS) and for clipping a long caption. The logical title still contains the full tab title.

Alternative considered: hide the system title bar and render a custom GPUI title row with the SVG logo, brand label, and filename. Rejected because it would reimplement drag regions, hit-testing, and window controls across Windows/macOS/Linux for no extra information.

Alternative considered: `filename - Markion` (VS Code style). Rejected because the request is to show the file name *after* the Markion logo and text.

### 2. Project the title from existing tab identity, not from document text

Add a pure helper, e.g. `window_title(title: &str, is_dirty: bool) -> String`, next to the status-bar helpers. `title` is `EditorTab::title()` (already `title_from_path`). `is_dirty` is true only for document tabs whose `is_dirty()` flag is set.

```text
active tab path -> title_from_path ----+
document dirty flag (documents only) --+-> window_title -> Window::set_window_title
```

No Markdown parse, stats, outline, or highlight cache is read. Caret motion and preview debounce do not change the title.

Alternative considered: using the first Markdown heading as the window title. Rejected because the user asked for the currently open *file name*, which tabs already display.

### 3. Apply the title from one sync point, only when it changes

Call `window.set_window_title` from the root view’s render path (or a single `sync_window_title` invoked there), comparing against the last applied string stored on `MarkionApp`. That covers tab switches, Save / Save As, rename, dirty toggles, and image-tab activation without hunting every mutation site.

Do not call `set_window_title` when the string is unchanged, so typing that does not flip dirty state does not hit the platform title API every frame.

Alternative considered: set the title only from file-open/save/tab handlers. Rejected because dirty transitions and in-app renames are easy to miss.

### 4. Strip identity from status-bar feedback; keep save-state and messages

Change `status_bar_feedback` so document tabs render `{save_state} | {status}` and image tabs render `{status}` only. The right-hand persistent context row is unchanged. Localized `TitleModified` / `TitleSaved` tokens stay in the status bar; they are save-state, not identity.

Alternative considered: also moving “Modified” / “Saved” into the title bar. Rejected as redundant with the `*` suffix and as crowding the caption.

### 5. Docs follow the chrome contract

Update `README.md` / `README.zh-CN.md` so chrome identity is described on the title bar, and align `docs/faq.md` so the documented `*` suffix refers to the native title bar rather than the status row.

## Risks / Trade-offs

- [Risk] Very long file names fill the native caption and crowd window controls on narrow windows. → Accept OS truncation; do not add a second path tooltip on the title bar (tabs already have hover tooltips with the full path).
- [Risk] Calling `set_window_title` every render could flicker or spam the platform. → Compare with the last applied title and skip no-ops.
- [Risk] macOS / Linux caption placement differs from Windows (centered vs after the icon). → Specify the *string* (`Markion - {file}`), not pixel placement; platform chrome owns layout.
- [Risk] Tests today assert `note.md *` inside status feedback and would fail after the move. → Move those assertions onto the window-title helper and add a status-feedback test that identity tokens are absent.

## Migration Plan

No persisted data or preference migration. Ship as a chrome-only change: helper + `set_window_title` sync + status-bar string + tests + README/FAQ. Rollback is reverting those files; documents and config are untouched.

## Open Questions

None — filename-after-brand, dirty `*` on document tabs only, and status-bar identity removal are decided.
