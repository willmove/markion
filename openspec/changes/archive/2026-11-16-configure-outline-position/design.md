# Design

## Context

See proposal.md for motivation. Files and Outline share one sidebar, scroll handles, and a session-persisted width. The workspace row currently places that sidebar and its divider before the document column, and resizing assumes the pointer x-coordinate is the width. Preferences use a dependency-free model plus a tolerant TOML adapter and explicit startup, save, and reset mappings.

## Goals / Non-Goals

**Goals:** Keep one sidebar and existing cached document state while allowing Outline to dock on either side, including correct divider behavior and preferences compatibility.

**Non-Goals:** Introduce a second sidebar, duplicate outline rendering, or recalculate derived document data on preference changes.

## Decisions

1. Add a typed `OutlinePosition` model enum defaulting to Left. Map it to the TOML string `outline_position`; tolerate non-string values locally so one bad setting cannot discard other preferences. Older configs need no migration. A boolean would obscure the preference's meaning.
2. Derive whether the sidebar is on the right from the active Files/Outline tab and the position setting. Reuse the existing sidebar before or after the document column, keeping its divider at the inner edge. Moving Files too would exceed the requested outline-only scope; separate panels would change visibility and keyboard semantics.
3. Compute resized width relative to the workspace row's left/right bounds, preserving the existing 150–480 limits, debounce, reset, and shared session width. This avoids assuming the workspace starts at screen x=0.
4. Add the Left/Right segmented row beside the existing General sidebar control. Its setter only updates app preference state, saves, and notifies. Translate its label and choices in every supported interface language.

Data flow:

```text
config.toml -> PreferencesFile -> AppPreferences -> MarkionApp
General position choice -> setter -> persist_preferences -> config.toml
sidebar tab + outline position -> workspace child order + divider edge
divider pointer + workspace bounds -> shared width -> session layout
```

Document text/version, selection, undo, derived `Arc` caches, and per-tab outline folding remain untouched. Moving the same outline view reuses existing scroll/folding state rather than creating a second cache.

## Risks / Trade-offs

- [Files/Outline switching moves the shared sidebar when Right is selected] → Preserve existing two-tab interaction and test both positions explicitly.
- [Right-side resizing inverts the pointer math] → Exercise actual layout bounds and drag events in GPUI regression tests.
- [Placement interacts with document modes and tab chrome] → Verify bounds across all four document modes and hidden-sidebar state.

## Migration Plan

No explicit migration is required: omitted or invalid position loads as Left. Existing builds ignore the additional TOML key. Reverting the feature restores the prior layout without altering document or session data.
