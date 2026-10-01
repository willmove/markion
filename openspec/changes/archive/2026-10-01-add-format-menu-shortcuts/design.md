## Context

See `proposal.md` for the motivation. The five target formatting actions and their Markdown transforms already exist and are exposed in both the native and in-window Format menus. They are absent from the shared `menu_shortcuts` registry, `bind_app_keys`, the in-window menu's shortcut-bearing rows, and the categorized shortcut catalog, so they have neither bindings nor discoverable shortcut labels.

The current shortcut architecture resolves a stable action id to a default GPUI keystroke and curated Windows/Linux and macOS labels. That descriptor drives effective binding resolution, persisted overrides, conflict checks, runtime rebinding, and menu/reference display. The new shortcuts must extend this path rather than introduce a second keymap source.

## Goals / Non-Goals

**Goals:**

- Add the five bindings as ordinary customizable menu actions with stable ids and platform labels.
- Keep keyboard invocation, native-menu key equivalents, in-window labels, and Preferences shortcut rows synchronized through existing sources of truth.
- Reuse the existing formatting action handlers without changing document transformation behavior.

**Non-Goals:**

- Add a math/equation-block action from the reference application's screenshot; Markion has no corresponding Format-menu command, and `Ctrl/Cmd+Shift+M` remains the existing Format Table binding.
- Redesign shortcut customization, introduce chords, or change existing default bindings.
- Change parsing, rendering, document-versioning, or caching behavior.

## Decisions

### 1. Extend the shared shortcut registry with five stable actions

Add descriptors for `unordered-list`, `ordered-list`, `task-list`, `block-quote`, and `code-fence`, and include them in the registry's `ALL` collection. Their GPUI defaults are `secondary-shift-]`, `secondary-shift-[`, `secondary-shift-x`, `secondary-shift-q`, and `secondary-shift-k`; curated labels render `Ctrl` on Windows/Linux and `Cmd` on macOS.

This makes the actions valid persisted-override ids and automatically brings them into registry-wide validation and conflict detection. Keeping the descriptors alongside the existing formatting shortcuts avoids a parallel table that could drift. The alternative of binding five literal keys only in bootstrap was rejected because it would bypass customization, effective labels, and conflict checks.

### 2. Represent bracket keys as literal GPUI key components

Use literal `[` and `]` as the final key component in the GPUI binding strings. GPUI accepts single-character key components, and Markion's keystroke parser/formatter preserves the bracket symbols; the explicit curated labels guarantee the intended menu and reference presentation.

Named forms such as `bracketleft` and `bracketright` were not chosen because platform key events are normalized to the character key used by GPUI matching, while the literal form directly expresses the screenshot mapping. Tests will parse and simulate both bracket bindings so layout or serialization mistakes are caught before manual verification.

### 3. Bind the descriptors to the existing actions in the complete keymap rebuild

Add one `KeyBinding` per descriptor in `bind_app_keys`, targeting the existing `UnorderedList`, `OrderedList`, `TaskList`, `BlockQuote`, and `CodeFence` actions. No new action or editing handler is introduced. The data flow remains:

`effective registry binding → GPUI action dispatch → existing formatting handler → existing document edit/version update`

The native menu already references those actions, so GPUI can derive their key equivalents from the new bindings. Document edits continue through the current mutation path; derived Markdown state remains cached per version and no new parse, render, or cache path is added.

### 4. Reuse existing localized Format labels in the shortcut catalog

Convert the five in-window Format rows to the shortcut-bearing `action_item!` form and pass their registry descriptors. Add five Editing-category shortcut rows keyed by the same stable ids, reusing the existing localized `Msg::ItemBullets`, `ItemNumbers`, `ItemTask`, `ItemQuote`, and `ItemCodeFence` strings instead of introducing duplicate translation text.

The alternative of expanding the seven-language static shortcut-label arrays was rejected because the menu catalog already contains correct translations and duplicating them would create localization drift.

### 5. Pin every integration seam with focused tests

Extend registry uniqueness/completeness tests, keymap-to-registry count checks, Format-menu bound/unbound metadata checks, platform-label assertions, and shortcut-catalog id coverage. Add GPUI dispatch tests for all five defaults and at least the two bracket bindings, plus an override/reset assertion for one new action, so the implementation proves both the default path and integration with customization.

## Risks / Trade-offs

- [Bracket shortcuts can behave differently on non-US keyboard layouts] → Use GPUI's literal character-key representation, cover parsing and dispatch in automated tests, manually verify on Windows and macOS where available, and retain user overrides as the escape hatch for layouts where the defaults are awkward.
- [A descriptor is added to the registry but omitted from a menu, catalog, or binding table] → Extend existing source-contract and registry-count tests to name all five actions and fail on drift.
- [A new default collides with another current binding] → Registry uniqueness tests compare canonical effective keystrokes; the selected combinations are currently unused, and future reassignment uses the existing conflict checker.

## Migration Plan

No data migration or configuration schema change is required. Existing installations receive the five defaults when no matching override is present, and users may immediately reassign them through Preferences. Rollback removes the five registry/binding/catalog/menu-label entries; existing documents and unrelated shortcut overrides remain unchanged.
