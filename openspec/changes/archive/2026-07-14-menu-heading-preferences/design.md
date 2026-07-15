## Context

`MarkdownDocument::apply_markdown_format` already handles `MarkdownFormat::Heading(level)` for levels 1–6 (`level.clamp(1, 6)`). UI wiring stops at three dedicated actions (`Heading1`–`Heading3`) duplicated across the in-window dropdown (`active_menu_dropdown`) and `install_menus`. Preferences recently gained interactive controls (`make-preferences-controls-interactive`); this change adds one more segmented preference using the same `preference_option_button` pattern as Language and sidebar tab.

## Goals / Non-Goals

**Goals**

- One persisted preference, two allowed values: max heading level **3** (default) or **6**.
- Menus, shortcuts, and Preferences panel stay consistent after toggling.
- Minimal duplication when building heading menu items.

**Non-goals**

- Per-heading visibility checkboxes, setext syntax, or config-file-only depth without a panel control.
- Re-parsing or cache invalidation changes.

## Decisions

### 1. Store `heading_menu_max_level: u8` with allowed values `{5, 6}`

**Why:** Matches existing numeric heading levels and keeps TOML simple (`heading_menu_max_level = 6`). Invalid/missing values default to `5` so H4 and H5 are always available in the Format menu.

**Alternatives considered:** Boolean `extended_headings` — rejected as less self-documenting in `config.toml`.

### 2. Add `Heading4`, `Heading5`, `Heading6` GPUI actions

**Why:** Follows the existing `Heading1`–`Heading3` pattern and keeps keybinding registration straightforward.

**Implementation sketch:** Thin handlers call a shared helper `apply_heading_level(&mut self, level: u8, cx)` to avoid six copies of `apply_markdown_format` boilerplate.

### 3. Centralize menu item generation

Add helpers:

- `heading_menu_items_in_window(app, palette, cx) -> Vec<Div>` for the GPUI dropdown
- `heading_menu_items_native(language, max_level) -> Vec<MenuItem>` for `install_menus`

Both iterate `1..=max_level` and map level → label (`Msg::ItemH1`…`ItemH6`) + action.

**`install_menus` signature:** extend to `install_menus(language: Language, heading_menu_max_level: u8, cx: &mut App)` and update all call sites (startup, language change, heading-depth change, preferences reset).

### 4. Keybindings

Register `secondary-4/5/6` → `Heading4/5/6` alongside existing `secondary-1/2/3`. When max level is 3, the extra actions remain bound but are unreachable from menus; this matches how other actions exist without menu entries. Shortcut reference lists H4–H6 only when max level is 6 (or always list them with a note — prefer **conditional listing** when depth is 6 only, to avoid documenting dead shortcuts).

### 5. Preferences panel UX

Add a row under **Other** (after existing boolean rows):

- Label: localized "Heading menu"
- Segmented control: **H1–H5** | **H1–H6**
- On change: update state, persist, `install_menus`, `cx.notify()`

Reuse `preference_option_button` from the interactive-preferences work.

## Data flow

```
Preferences panel / config.toml
        ↓ load / toggle
MarkionApp.heading_menu_max_level
        ↓
┌───────────────────────┬─────────────────────────┐
│ active_menu_dropdown  │ install_menus (native)  │
│ (in-window Format)    │                         │
└───────────────────────┴─────────────────────────┘
        ↓ user picks Hn
HeadingN action → apply_heading_level(n) → MarkdownDocument (unchanged cache path)
```

## Risks / Trade-offs

- **Native menu reinstall** on depth change — same cost as language switch; acceptable.
- **Wider Format dropdown** when depth is 6 — may need slightly taller dropdown; existing dynamic width logic should absorb three extra rows.

## Migration

Additive TOML field; absent key → `3`. No legacy file migration required.
