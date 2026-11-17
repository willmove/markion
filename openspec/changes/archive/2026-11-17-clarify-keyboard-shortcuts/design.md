## Context

Markion binds shortcuts in `src/app/bootstrap.rs` using GPUI key strings such as `secondary-s`, where `secondary` maps to Ctrl on Windows/Linux and Cmd on macOS. The Help -> Keyboard Shortcuts dialog comes from localized static text in `src/i18n.rs`, so it currently exposes those internal key names directly.

## Goals / Non-Goals

**Goals:**

- Make the shortcut reference easier to scan by arranging shortcuts as action/platform-key rows.
- Show Windows/Linux and macOS shortcuts separately so `secondary` no longer leaks into user-facing Help text.
- Replace the sidebar toggle shortcut with `secondary-shift-b`, matching the requested Ctrl+Shift+B / Cmd+Shift+B shape.

**Non-Goals:**

- Add runtime keybinding customization.
- Change editor formatting shortcuts such as Bold (`secondary-b`).
- Touch Markdown parsing, preview derived-state caches, or typing-path rendering.

## Decisions

1. Keep GPUI binding strings as the implementation source of truth and translate them manually in the localized Help reference.

   Rationale: bindings are currently static and there is no keymap data model to render dynamically. A small static reference update is lower risk than introducing a new abstraction only for this dialog. Alternative considered: generate the Help table from structured shortcut metadata; useful later if configurable shortcuts are added, but unnecessary for this scoped change.

2. Use `secondary-shift-b` for sidebar toggle.

   Rationale: it gives the user-requested Ctrl+Shift+B on Windows/Linux, maps naturally to Cmd+Shift+B on macOS, and avoids the existing Bold shortcut (`secondary-b`). Alternative considered: keep `secondary-alt-b` and only clarify it in the table; rejected because the request explicitly asks for a more convenient binding.

3. Update every localized shortcut reference table rather than only English/Chinese.

   Rationale: the i18n contract requires the keyboard shortcut reference to be localized through `shortcut_reference`, and stale platform text in secondary languages would make the Help dialog internally inconsistent.

## Risks / Trade-offs

- [Risk] Static Help text can drift from bindings. -> Mitigation: update tests to assert the new table/platform terms and the sidebar binding, and keep the implementation edit in the same change.
- [Risk] Ctrl+Shift+B may already mean build in some tools. -> Mitigation: Markion currently has no build command, and the shortcut is scoped to this app's View/sidebar behavior.
