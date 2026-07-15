## Context

Markion installs application keybindings in `src/app/bootstrap.rs`, while the custom in-window dropdowns in `src/app/root_view.rs` invoke the same actions through click listeners. Native GPUI menus can associate menu entries with actions, but the custom dropdown rows currently receive only a localized label, so they have no shortcut text to render. Shortcut reference data in `src/i18n.rs` already distinguishes Windows/Linux from macOS, but it is organized by Help-panel category rather than by menu action.

This change affects static application chrome only. It must preserve localization, the configured Format-menu heading depth, native menu behavior, and all document-version caching invariants.

## Goals / Non-Goals

**Goals:**

- Render the current platform's user-facing shortcut beside every bound action in the six in-window application menus.
- Keep shortcut labels tied to the binding metadata used when GPUI keybindings are installed.
- Keep localized action labels left-aligned and shortcut labels consistently right-aligned without clipping.
- Preserve conditional H6 visibility and the absence of shortcut text for unbound actions.

**Non-Goals:**

- Add, remove, or remap shortcuts.
- Add customizable keymaps or persistence.
- Add shortcut annotations to file-tree or preview context menus.
- Change native menu dispatch, document state, Markdown parsing, or derived-state caching.

## Decisions

1. Represent each menu-visible binding with shared structured metadata.

   The binding metadata will carry the GPUI key string plus explicit Windows/Linux and macOS display combinations. Keybinding installation will use the GPUI string, and the in-window menu will use the platform display value from the same descriptor. Actions with more than one active binding may expose all applicable combinations in a compact joined label, so the UI does not claim a shortcut that is not installed.

   This is preferred over hard-coding shortcut text directly in `active_menu_dropdown`, which would create a third independently maintained key map. Parsing GPUI key strings at render time was also considered, but explicit display values are safer for platform conventions such as `secondary`, `Alt` versus `Option`, and fixed `Ctrl+Tab` bindings.

2. Reuse the current platform selection convention.

   Shortcut display will use the same build-target distinction as `ShortcutPlatform::current()`: Cmd/Option on macOS and Ctrl/Alt on Windows/Linux. Key names remain concise platform terms rather than localized prose, matching the existing shortcut panel.

3. Extend the menu-row primitive with an optional shortcut column.

   `menu_action_button` will accept optional shortcut text and render a two-column flex row: the localized action label on the left and muted shortcut text on the right. Rows without bindings will render only the action label. Dropdown widths will be increased per menu where necessary, while retaining the existing theme palette, hover behavior, click listeners, separators, and language-specific left offsets.

4. Keep conditional menu construction authoritative.

   The existing `action_item!` call sites will supply the appropriate shared shortcut descriptor for bound actions and no descriptor for unbound actions. H1–H5 will always receive their existing bindings, while H6 will receive its shortcut only in the branch that already exposes H6. No hidden or context-only binding will create a menu row.

5. Verify metadata and rendering contracts separately.

   Unit tests will cover platform label selection and binding aliases from the structured metadata. Menu construction tests will assert that representative bound and unbound actions are wired correctly, all six dropdowns use the shortcut-aware row, and conditional heading rows retain their existing depth behavior.

## Risks / Trade-offs

- [Risk] Longer localized labels plus shortcut text could clip or make menus excessively wide. → Mitigation: use a fixed right-aligned shortcut column, add an intentional gap, and adjust each dropdown width based on its longest expected row.
- [Risk] The Help shortcut catalog can still drift from the shared menu/binding metadata because its category model is separate. → Mitigation: scope the shared descriptor to binding installation and menu display now, retain existing catalog tests, and avoid duplicating new literal shortcut strings in the menu renderer.
- [Risk] Multiple bindings on one action can create a long shortcut label. → Mitigation: use compact separators and test the longest row when selecting dropdown widths.
- [Risk] Visual changes could accidentally affect click targets. → Mitigation: keep the listener and full-width row on the outer element; the two text columns remain non-interactive children.

