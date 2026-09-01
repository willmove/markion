## Context

View-mode actions already exist (`SetEditMode`, `SetSplitPreviewMode`, `SetReadMode`, plus unchanged `SetVisualEditMode` and `ToggleViewMode`) and are bound through the shared `menu_shortcuts` registry. Factory defaults today are `Ctrl+Alt+1/2/3` (source / split / Read). File → New Tab (`NewTab`) is wired in both menu surfaces but has no registry descriptor, so it cannot show a shortcut or accept `[shortcuts]` overrides. `F1` is the factory default for `show-shortcuts`, which opens Preferences on the Shortcuts tab; Help no longer lists Keyboard Shortcuts.

This change retargets those factory defaults, binds New Tab, and adds a Help overlay for Markdown syntax. It must stay inside the existing shortcut registry, i18n catalog, and overlay patterns. Derived Markdown caches, highlight memoization, and cached text handles are not on the typing path of this work.

The complete-but-unarchived `customize-shortcuts-in-preferences` change still describes `F1` → `ShowShortcuts`. This change supersedes that factory default; archive order must keep the later F1 assignment.

## Goals / Non-Goals

**Goals:**

- Ship the requested factory defaults: source `Ctrl+/`, Split Preview `Ctrl+P`, New Tab `Ctrl+Shift+N`, Read `Ctrl+Shift+R`, Markdown Reference `F1` (macOS uses `Cmd` wherever the user wrote `Ctrl`).
- Keep keymap, in-window labels, native-menu equivalents, Preferences shortcut rows, and docs synchronized through the existing registry.
- Add a real Markdown syntax reference (not a stub menu item) as a Help overlay.
- Preserve stored `[shortcuts]` overrides and leave Visual Edit, view-mode cycling, and Open in New Tab defaults alone.

**Non-Goals:**

- Opening Markdown Reference as a document tab, welcome-document replacement, or remote help page.
- Rendering the cheat sheet through the live preview / Visual Edit pipeline.
- A command palette, Print action, or new window chord occupying `Ctrl+P` / `Ctrl+Shift+N`.
- Changing Visual Edit (`Ctrl+Alt+4`) or cycle-mode (`Ctrl+Shift+V`) bindings.
- Persistence-schema changes or a general unbound-shortcut redesign beyond what `show-shortcuts` needs after losing `F1`.

## Decisions

### 1. Change factory defaults in the shared registry, do not add a second keymap

Update `SET_EDIT_MODE`, `SET_SPLIT_PREVIEW_MODE`, and `SET_READ_MODE` in `menu_shortcuts`, and add `NEW_TAB` plus `SHOW_MARKDOWN_REFERENCE`. `bind_app_keys` continues to install `effective_binding` for every bound registry entry. In-window File / View / Help rows that currently omit a descriptor (New Tab, and the new Help item) take the same `action_item!` / `file_action_item!` shortcut argument used by sibling items.

Rejected: hard-coding the new keys only in `bootstrap.rs`. That would desync menu labels, conflict detection, and Preferences.

GPUI strings and curated labels:

| Action id | GPUI default | Windows/Linux | macOS |
|---|---|---|---|
| `set-edit-mode` | `secondary-/` | `Ctrl+/` | `Cmd+/` |
| `set-split-preview-mode` | `secondary-p` | `Ctrl+P` | `Cmd+P` |
| `new-tab` | `secondary-shift-n` | `Ctrl+Shift+N` | `Cmd+Shift+N` |
| `set-read-mode` | `secondary-shift-r` | `Ctrl+Shift+R` | `Cmd+Shift+R` |
| `show-markdown-reference` | `f1` | `F1` | `F1` |

Use the literal `/` key component (`secondary-/`). GPUI `Keystroke::parse` accepts a one-character final component, and platform key events normalize to `"/"`, not the named token `slash`. Curated labels still show `/`. Verify parse + dispatch in tests.

Unchanged: `set-visual-edit-mode` (`secondary-alt-4`), `toggle-view-mode` (`secondary-shift-v`), `open-in-new-tab` (`secondary-t`).

### 2. Free `F1` by making `show-shortcuts` factory-unbound

`ShowShortcuts` remains the handler that opens Preferences → Shortcuts. It MUST NOT keep factory `f1`. Keep the action in the customizable registry so a user can assign a shortcut later.

`MenuShortcut` currently always has a binding. Add an optional factory binding (or skip keymap install when the descriptor is unbound). `bind_app_keys` installs a `ShowShortcuts` key only when a valid override exists. The Preferences row shows an empty / “not set” chip until assigned; per-action reset returns it to unbound. Conflict detection must not treat an unbound default as occupying `F1`.

Rejected: inventing a replacement factory key (e.g. `F10`) the user did not request. Rejected: leaving `F1` on both actions.

### 3. Markdown Reference is an About-style overlay, not a tab

Add `ShowMarkdownReference` / `CloseMarkdownReference` and a boolean overlay flag, following `about_dialog_open`. Help → Markdown Reference and `F1` open it; Escape and an explicit close control dismiss it; invoking the in-window item closes the Help dropdown. The overlay occludes the workspace, uses the active theme palette, and does not mutate tabs, documents, dirty state, or derived Markdown caches.

Rejected: opening a new untitled tab with sample Markdown. That would interact with dirty guards, “open in current tab”, session restore, and the typing-path caches. Rejected: shelling out to the GitHub README — the request is in-app reference content.

### 4. Cheat-sheet body is structured localized content, not live preview

Provide `markdown_reference(language) -> Vec<MarkdownReferenceSection>` next to the existing shortcut catalog. Each section has a title, a monospaced syntax example, and a short caption covering constructs Markion actually parses: headings; emphasis / strong / strikethrough / highlight / super / sub / inline code; links and images; quotes and thematic breaks; unordered / ordered / nested / task lists; tables; fenced code; inline and display math; footnotes and reference links.

Overlay chrome (menu label, title, close, status) goes through `Msg`. Section bodies live in the dedicated function so `Msg` does not grow by dozens of example strings, matching how `shortcut_reference` already works. Every supported language returns the same section set with non-empty text.

Rejected: driving the overlay from `DEFAULT_WELCOME_MARKDOWN` (English-only, not a cheat sheet). Rejected: rendering examples through the preview pipeline (would create a hidden document version and cache traffic for Help).

### 5. Help menu placement and width

Insert Markdown Reference as the first item in the help/reference group, after Check for Updates and its separator, before Report an Issue, on both the in-window dropdown and the native Help menu. Show the effective `F1` label beside the in-window item. Re-check `AppMenu::Help` dropdown width against the longest new localized label; widen only if it would clip. Do not change per-language menu-title offsets unless a clip is demonstrated.

### 6. Docs and tests stay on the same defaults

Update `docs/keyboard-shortcuts.md` and `docs/faq.md` (and any README mention of `Ctrl+Alt+1/2/3` or `F1` → shortcuts). Extend registry uniqueness, keymap-count, menu source-scan, catalog-id, platform-label, and GPUI dispatch tests for the five bindings plus overlay open/close. One override/reset test on `set-edit-mode` or `new-tab` proves customization still works after the default move.

## Risks / Trade-offs

- [`Ctrl+P` / `Cmd+P` is Print on many desktops; `Cmd+Shift+N` is often New Window] → Accept the requested mapping; Markion has neither Print nor New Window. Users can rebind in Preferences. Document the defaults in the shortcut list.
- [`Ctrl+/` may be awkward on non-US layouts] → Bind the GPUI character `/`, cover parse/dispatch in tests, keep overrides as the escape hatch.
- [Existing `show-shortcuts = "f1"` overrides would collide with Markdown Reference] → Overrides win; Markdown Reference then has no effective key until the conflict is cleared. Typical installs store no override for the old default, so they receive `F1` → reference.
- [Unbound `show-shortcuts` may need a small Preferences empty-state] → Prefer a blank chip over inventing a new factory key. If the capture UI assumes a non-empty default, add that empty state in the same change.
- [Help overlay content can drift from parser coverage] → Spec the construct groups to match current Markdown support; tests assert every language has every group.
- [Unarchived `customize-shortcuts-in-preferences` still says F1 opens Shortcuts] → This change’s chrome-platform delta is the source of truth for F1 after it is archived.

## Migration Plan

No `config.toml` schema change. On upgrade, installations without matching overrides get the new defaults immediately. Rollback reverts registry bindings, Help item, and overlay; unrelated overrides remain.

`show-shortcuts` factory unbinding is not a stored migration: an absent override already means “use default,” and the new default is unbound.

## Open Questions

None. Platform `Ctrl` → `Cmd` follows existing `secondary` convention. Markdown Reference is in-app overlay content, not an external URL.
