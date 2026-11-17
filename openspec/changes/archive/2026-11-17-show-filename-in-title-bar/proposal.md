## Why

The native window title bar currently shows only the brand name “Markion”, while the active file name lives in the bottom status bar (`Markion - {title} * | …`). Users looking at the top of the window — the conventional place for document identity in desktop editors — cannot see which file is open, and the status bar wastes its left region repeating that identity instead of leaving room for save state, operation feedback, and persistent document context.

## What Changes

- Show the active tab’s file name in the native window title bar, immediately after the existing Markion logo and “Markion” text (for example `Markion - notes.md`).
- Keep the unsaved-changes `*` suffix next to that file name in the title bar, matching the documented dirty-marker behavior.
- Stop repeating the brand name, file name, and dirty marker in the status-bar feedback string. The status bar continues to show save-state tokens, transient operation feedback, and the existing persistent context (counts, caret, Git branch).
- Update bilingual README claims that currently describe document identity as a status-bar concern, plus the FAQ sentence about the title-bar dirty marker so it matches the new surface.

**Non-goals:** custom-drawn / client-side title bars, relocating window controls, changing tab-strip titles, showing the full filesystem path in the title bar, translating the brand name or file name, or adding configurable title-bar items.

This change does not touch per-document-version Markdown caches, syntax-highlight memoization, text-handle reuse, or any `crates/*` member.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `chrome-platform`: Native window title presents active-tab identity after the Markion brand; status-bar feedback no longer duplicates that identity.
- `project-documentation`: README (and related overview claims) describe document identity as a title-bar concern rather than a status-bar one.

## Impact

- Window bootstrap currently sets a static `TitlebarOptions.title` of `"Markion"` in `src/app/bootstrap.rs` and never calls `Window::set_window_title`.
- Status-bar composition lives in `src/app/status_bar.rs` (`status_bar_feedback`) and is rendered from `src/app/root_view.rs`; image tabs use a similar inline `Markion - {title}` prefix.
- Tests in `src/app/tests.rs` assert that status feedback contains `note.md *`; those contracts move to the window-title helper.
- Docs: `README.md`, `README.zh-CN.md`, and `docs/faq.md`.
- No new dependencies, no workspace-member GPUI usage, and no persistence or preference schema changes.
