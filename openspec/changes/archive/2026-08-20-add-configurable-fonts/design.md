# Design — add-configurable-fonts

## Context

Today one hardcoded family (`.SystemUIFont`) flows from the root view (`root_view.rs`) into every surface — chrome, source editor, rendered body, inline code — while six call sites in `preview.rs` hardcode "JetBrains Mono" for code-shaped surfaces. Typography preferences (`editor_font_size`, `rendered_font_size`, `paragraph_spacing`) already have a presentation-only invalidation pipeline (`refresh_typography_measurements`) that clears measurements without touching document versions or derived caches. gpui resolves `.SystemUIFont` per platform (Segoe UI on Windows, SF Pro on macOS, IBM Plex Sans on Linux) and panics in test-support builds when a non-system family fails to resolve. See proposal.md for motivation.

## Goals / Non-Goals

Goals:

- Three independent font slots (source / rendered / code) with predictable resolution and no change to chrome fonts.
- Font changes ride the existing presentation-only invalidation path.
- Fix the silent proportional degradation of code text when JetBrains Mono is absent.

Non-Goals (beyond the proposal's): searchable font-picker UI (v1 uses validated free-form input), line-height derivation from font metrics.

## Decisions

### D1 — Resolution precedence: explicit preference over theme over default

`effective(slot) = preference[slot] ?? active_theme.fonts[slot] ?? DEFAULT[slot]`, per slot.

Alternative considered: theme-over-preference (a strict reading of "theme override"). Rejected because an explicit panel choice would then be silently defeated by any theme carrying `[fonts]`, forcing the panel to grow "overridden by theme" warning states. With preference-first, out-of-box users (preference unset) still see theme fonts in full, and an explicit user choice always wins. The panel expresses the unset state as a first-class **"Follow theme"** choice rather than an empty value, which keeps a single point of configuration visible at any time.

### D2 — Slot naming aligned with existing preference roots

`config.toml` keys: `editor_font_family`, `rendered_font_family`, `code_font_family` — matching `editor_font_size` / `rendered_font_size` so each plane keeps one word-root across the file. Theme `[fonts]` keys reuse the same roots: `editor`, `rendered`, `code`. (The exploration conversation sketched `reading_font_family`; it was renamed to `rendered_font_family` for file-level consistency — panel labels still say "Reading font" per existing i18n wording.)

### D3 — Defaults and the code fallback chain

Defaults equal today's rendering exactly: source and rendered slots default to `.SystemUIFont` (the gpui magic name; resolved per platform); the code slot defaults to "JetBrains Mono" **with** `FontFallbacks` of monospace families (Cascadia Code, Consolas, SFMono-Regular, then the platform per-glyph fallback), closing the degradation gap. Source/rendered slots carry no explicit fallback list — platform per-glyph fallback already covers CJK and missing glyphs. A user-set code family keeps the same monospace tail appended. A preference value of `".SystemUIFont"` is legal and passes through to the platform resolver.

### D4 — Application points: per-plane containers, chrome untouched

The root view keeps `.SystemUIFont` so chrome inherits it as today. The reading slot is applied on the rendered-surface containers (preview list, Visual Edit list); the source slot on the source editor container; the code slot at the six existing code-view sites. The caret/ascent measurement in `preview.rs` (currently resolving `.SystemUIFont` directly) resolves the rendered slot instead, so caret geometry tracks the actual body font.

### D5 — Invalidation and cache identity

A change to any resolved slot goes through `refresh_typography_measurements(true, true)`. The source measured-height cache key gains the resolved source family (today it holds version/wrap-width/font-size; a family swap without a size change would otherwise hit a stale cache). Applying a theme re-resolves all slots and runs the same invalidation when any effective family actually changed. Family changes are presentation-only: no document-version bump, no derived-cache rebuild, scroll anchors preserved — same contract as size changes.

### D6 — Panel control: tri-state row with an installed-font selection list

Each slot renders as a row: localized label, a **Follow theme** state (preference unset, showing the effective family that would apply), and when overridden the explicit family name rendered in its own family (live preview). Activating the control opens a scrollable selection list built from `TextSystem::all_font_names()` (refreshed whenever the panel opens): a follow-theme entry first, then every installed family sorted, each rendered in its own family, with the slot's current choice marked active. Selecting an entry applies and persists immediately and closes the list; only one slot's list is open at a time. An earlier draft used a free-form capture input with advisory validation; it was replaced after review — users should pick from the fonts the machine actually offers rather than type (and mistype) family names. Free-form values still load and persist verbatim from `config.toml` (the storage contract is unchanged), and the row shows an advisory warning when such a stored value is not installed.

### D7 — Line heights stay proportional constants

`DocumentTypographyMetrics` keeps its size-proportional constants. Fonts with unusual vertical metrics may render slightly tighter or looser than the tuned defaults; accepted as a documented limitation rather than deriving line heights from font metrics in this change.

### D8 — Test strategy avoids real shaping

Precedence, TOML/config round-trips, and cache-key sensitivity are tested as pure logic. No unit test performs text shaping with non-system families: gpui's test-support builds panic when a non-system font fails to resolve, and CI machines do not install JetBrains Mono.

## Risks / Trade-offs

- [Chosen family missing or enumerated differently on another machine] → code slot has a monospace fallback chain; other slots fall back to the platform system font via gpui's existing resolution; panel shows an advisory warning for unknown names.
- [Tall-font line-height mismatch] → documented limitation (D7); follow-up could derive line heights from font ascent/descent.
- [Stale layout after a family-only change] → family joins measured-height cache keys; explicit regression task.
- [Theme authors surprised that explicit user preferences win] → panel's Follow-theme state surfaces the theme's contribution; delta spec documents precedence normatively.
- [i18n breadth] → new Msg entries must land in every supported language, enforced by the existing i18n tests.

## Migration Plan

Purely additive: absent config keys mean `None` (follow theme/default) and absent `[fonts]` means theme defaults — behavior is byte-identical to today until a user opts in. Rollback = remove the keys; defaults reassert.
