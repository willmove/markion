## Context

The prior change `update-default-shortcuts-and-markdown-reference` already rewired source/split/New Tab/Read/`F1`. This follow-up reassigns three more defaults, adds Open Folder to the registry, and polishes Markdown Reference scrolling. All shortcut work continues through the shared `menu_shortcuts` + `effective_binding` path.

## Goals / Non-Goals

**Goals:**

- Ship the requested factory defaults with macOS `Cmd` via `secondary`.
- Free `Ctrl+E` for Visual Edit by moving inline code to `Ctrl+Shift+\``.
- Bind Open Folder like other File actions (`open-folder` id).
- Make Markdown Reference overflow discoverable with a right-side scrollbar.

**Non-Goals:**

- Replacing the reference overlay with a document tab.
- Changing cycle-mode, Edit/source, Split Preview, or Markdown Reference `F1`.
- Migrating stored overrides (overrides keep winning).

## Decisions

### 1. Inline code uses a literal backtick keystroke

GPUI stores the OEM-3 key as `` ` `` on Windows, not the named alias `backquote`. Registry binding SHALL be `` secondary-shift-` `` so `Keystroke::parse` and runtime events match. Labels remain `Ctrl+Shift+\`` / `Cmd+Shift+\``. Markion's `format_keystroke_label` already renders a single-character key as `` ` ``.

### 2. Visual Edit takes `secondary-e`; Read takes `secondary-r`

No other factory binding uses these chords after inline code moves. Cycle-mode and Edit/source stay unchanged.

### 3. Open Folder joins the registry next to Open

Add `OPEN_FOLDER` (`open-folder`, `secondary-shift-o`). Install in `bind_app_keys`, pass the descriptor to the in-window File row, insert into `ALL` after `OPEN_DOCUMENT`, and add a Files catalog row after Open.

### 4. Markdown Reference uses the shared pane scrollbar

GPUI's `scrollbar_width` alone does not draw Markion's visible chrome thumb. Wire `markdown_reference_scroll` with `track_scroll` and reuse `pane_scrollbar_view` / `PaneScrollTarget::MarkdownReference` (same overlay pattern as Preferences), reserving `PANE_SCROLLBAR_RESERVED_WIDTH` on the body. Reset the handle when the overlay opens or closes.

## Risks / Trade-offs

- [Users who memorized Ctrl+E for inline code / Ctrl+Alt+4 for Visual Edit / Ctrl+Shift+R for Read] → Overrides still work; docs and catalog update. Acceptable for intentional default refresh.
- [Ctrl+R may collide with browser-style reload muscle memory] → In-app only; no web surface. Accept.

## Migration Plan

No schema migration. New installs and override-free configs pick up defaults on next launch. Document the new chords in `docs/keyboard-shortcuts.md` and `docs/faq.md`.
