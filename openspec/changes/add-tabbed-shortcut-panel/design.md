## Context

Help -> Keyboard Shortcuts currently calls `window.prompt` with a localized plain-text document produced by `shortcut_reference`. Native prompts do not render Markdown and cannot host tabs, category navigation, key-cap styling, or theme-aware layout. The application already has an in-app modal pattern in `preferences_panel_view`, so the shortcut reference can use the same overlay approach without introducing a new UI framework or dependency.

The completed but unarchived `clarify-keyboard-shortcuts` change established explicit Windows/Linux and macOS key strings and changed the sidebar toggle to `secondary-shift-b`. This change supersedes that change's text-table presentation while retaining the binding and platform-specific data.

Data flow after the change:

```text
localized shortcut catalog
          |
          v
ShowShortcuts action -> panel state (open, platform, category)
          |                         |
          +-------------------------+
                    |
                    v
          GPUI shortcut panel view
```

The catalog and panel state are static UI concerns. They do not read or invalidate document text, Markdown-derived state, syntax highlighting, preview lists, or per-version caches.

## Goals / Non-Goals

**Goals:**

- Replace the native text prompt with a theme-aware in-app modal.
- Let users switch between Windows/Linux and macOS shortcuts with a two-option platform tab control.
- Let users navigate the existing shortcut categories without scanning one long table.
- Render each action and its shortcut combinations as an aligned list row with visually distinct key labels.
- Keep all section names, actions, controls, and status feedback localized.
- Keep displayed shortcuts and actual bindings testably consistent.

**Non-Goals:**

- User-editable keybindings or persisted panel selection.
- Shortcut conflict detection.
- Search or filtering inside the shortcut panel.
- Changes to commands, except retaining the already-selected sidebar toggle binding.
- Changes to Markdown parsing, rendering, or editing caches.

## Decisions

1. Render a custom GPUI modal instead of extending `window.prompt`.

   The native prompt is intentionally a plain system dialog and cannot express the proposed interaction. The panel will follow the existing Preferences overlay structure: full-window scrim, occluding centered surface, theme palette, title bar, close control, and bounded scrolling. Alternative considered: continue generating ASCII text; rejected because alignment depends on font metrics and localized label widths and still cannot provide platform or category navigation.

2. Use one platform tab control and a category sidebar, not two levels of tabs.

   The top control contains only `Windows/Linux` and `macOS`. The left sidebar contains File, Tabs, Editing, View, Search, Tables, and Export. This keeps platform selection persistent while moving through categories and avoids seven category tabs wrapping or competing with the platform tabs. The first category is selected whenever the panel opens.

3. Default the platform from the build target and do not persist panel state.

   macOS builds open on the macOS tab; Windows and Linux builds open on Windows/Linux. Reopening resets to that platform and the first category. This makes the first view immediately relevant and avoids adding a preference for temporary help UI state. Users can still inspect the other platform in one click.

4. Replace formatted text generation with a structured localized catalog.

   The i18n layer will expose sections containing a localized category label and localized action rows. Each row carries Windows/Linux and macOS shortcut combinations as structured values, allowing the GPUI view to render separate key labels instead of parsing a display string. Existing key arrays and localized action labels are the migration source. Combined actions may expose multiple shortcut combinations in one row, but platform-internal names such as `Secondary` never reach the UI.

5. Keep panel interaction state in `MarkionApp`.

   Add open/closed state plus selected `ShortcutPlatform` and category. `ShowShortcuts` initializes these values, closes the active menu, updates status, and requests a render. Panel callbacks switch platform/category or dismiss the panel. Escape and the explicit close button dismiss it; action routing must avoid changing editor content while the modal is open.

6. Reuse the active theme palette and existing modal dimensions responsively.

   The panel uses the active `ThemePalette` for scrim, surface, text, borders, active tabs, hover states, and key labels. Its body has a bounded height and scrollable shortcut list so all categories remain usable on smaller windows. Key labels have stable padding and do not control the row width.

7. Carry the sidebar shortcut contract into this change.

   `secondary-shift-b` remains the implementation binding, displayed as Ctrl+Shift+B on Windows/Linux and Cmd+Shift+B on macOS. Ctrl+B / Cmd+B remains Bold. This makes the new change self-contained so the obsolete text-table requirement from `clarify-keyboard-shortcuts` does not need to be archived.

## Risks / Trade-offs

- [Risk] Display data can drift from `KeyBinding` declarations. -> Mitigation: retain one structured catalog, add tests for important command/platform mappings, and specifically assert the sidebar and Bold shortcuts.
- [Risk] A modal overlay can leak clicks or key actions to the editor. -> Mitigation: occlude the panel, place it last in the root overlay stack, and test dismissal and selection state independently from editing state.
- [Risk] Long translated labels can compress shortcut keys. -> Mitigation: give the action column flexible width, keep the key area non-shrinking, and allow the body to scroll rather than resizing rows unpredictably.
- [Risk] The prior active change can later reintroduce a conflicting table requirement. -> Mitigation: treat this change as its replacement and reconcile/remove the superseded change before archive.
- [Trade-off] No in-panel search is provided. -> The seven-category navigation keeps the current catalog manageable; search can be proposed separately if the command set grows materially.

## Migration Plan

1. Convert the current localized labels and platform key arrays into the structured catalog without changing bindings.
2. Add panel state and rendering alongside the existing prompt path.
3. Switch `ShowShortcuts` to open the panel and remove the text-table formatter and prompt call.
4. Verify all supported languages and light/dark themes, then remove obsolete prompt-specific tests.
5. Validate this change and reconcile the superseded `clarify-keyboard-shortcuts` change before archiving either shortcut-help specification.

Rollback is limited to restoring the native prompt call and text formatter; the sidebar binding can remain independently.

## Open Questions

None. Shortcut search and configurable keybindings remain explicit future work.
