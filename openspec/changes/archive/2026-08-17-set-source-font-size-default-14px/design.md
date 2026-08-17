# Design: set-source-font-size-default-14px

## Context

See `proposal.md` — Why. The source-editor font size default lives in a single constant, `DEFAULT_EDITOR_FONT_SIZE = 15` (`src/model.rs:82`), with bounds `MIN_EDITOR_FONT_SIZE = 10` / `MAX_EDITOR_FONT_SIZE = 32`. The rendered-view default is already 14px (`DEFAULT_RENDERED_FONT_SIZE`), so this change aligns the two defaults.

## Goals / Non-Goals

**Goals:**

- New default of 14px for the source editor, flowing through every existing consumer of the constant without new plumbing.
- Keep source line metrics proportional to the font size so caret, selection, scroll, and typewriter geometry stay consistent.

**Non-Goals:**

- No persistence-format or normalization changes (field name, 10–32px clamp, lenient parsing stay as-is).
- No migration of `config.toml` files that already persist an explicit `editor_font_size`.

## Decisions

### 1. Change the single constant; let existing derivation do the rest

Data flow for the default (all existing code):

```
DEFAULT_EDITOR_FONT_SIZE (src/model.rs)
  → AppPreferences::default()                    (src/model.rs:225)
  → storage fallback for missing/invalid fields  (src/storage/preferences.rs:315)
  → app state editor_font_size                   (src/app/application.rs:22,75)
  → DocumentTypographyMetrics::new               (src/app/mod.rs:1023–1030)
  → tab.line_height                              (src/app/application.rs:27,774; src/app/appearance.rs:263)
  → shaping / painting / hit-testing             (src/app/editor_element.rs)
```

Only the constant changes. `DocumentTypographyMetrics::new` already normalizes inputs and derives `editor_line_height = font_size * (EDITOR_LINE_HEIGHT / DEFAULT_EDITOR_FONT_SIZE)`, so both metrics and the per-tab `line_height` update automatically. The `px(EDITOR_LINE_HEIGHT)` value in `EditorTabState` construction (`src/app/state.rs:871`) is a placeholder that is overwritten from typography metrics when the tab is created (`src/app/application.rs:27`) and re-measured during layout, so it needs no change.

Alternative considered: special-casing the default in the storage layer only. Rejected — it would diverge from `AppPreferences::default()` and the Preferences-reset path, which both consume the constant.

### 2. Keep proportional line-height scaling (1.6×); adjust `EDITOR_LINE_HEIGHT` to 22.4

`editor_line_height = font_size * (EDITOR_LINE_HEIGHT / DEFAULT_EDITOR_FONT_SIZE)` uses both constants, so changing only the default font size would silently change the ratio for every size (24/14 ≈ 1.71) and keep an absolute 24px line height at the default. To preserve the historical 1.6× ratio, `EDITOR_LINE_HEIGHT` changes from 24 to 22.4 (= 14 × 1.6), giving a 22.4px line height at the new default and identical line heights at every non-default size (e.g. 32px font stays 51.2px). Alternative: leave `EDITOR_LINE_HEIGHT = 24` and accept the looser ≈1.71 ratio. Rejected — it changes line spacing at every size, a bigger visual shift than the requested default change, and would break the existing 1.6-encoded test expectations (38.4 at font 24, 51.2 at font 32).

### 3. Caching / versioning impact: none

Typography is presentation-only by contract (see the `document-typography` spec requirement "Typography changes preserve document and cache invariants"). This change alters only the initial value that flows through the existing pipeline; it does not touch document versions, per-version derived caches, memoized syntax highlighting, or the cached text handle. The preferences panel displays the live value, so no i18n strings change.

## Risks / Trade-offs

- [Existing users who never changed the font size silently move from 15px to 14px] → Accepted and stated in the proposal; anyone who prefers 15px can set it once and it persists.
- [A test hardcodes the old default values] → `document_typography_metrics_preserve_defaults_and_scale_boundaries` (`src/app/tests.rs:6143`) asserts `editor_font_size == 15.` and `editor_line_height == 24.`; both are updated to `14.` and `22.4`.
- [Spec drift] → Both `document-typography` and `theme-preferences` specs state the 15px default; deltas in this change update both so archive-time sync keeps spec and code aligned.

## Migration Plan

None required. Deploy is a normal build; rollback is reverting the constant (and test) — no persisted state references the default.
