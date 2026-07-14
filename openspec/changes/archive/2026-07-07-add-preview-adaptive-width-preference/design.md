## Context

Markion now has explicit Edit, Split Preview, and Read modes. Read mode currently uses the same preview pane rendering path as split preview, so the rendered content expands to the available pane width. On wide displays this creates very long lines, which is poor for reading Markdown prose.

The Preferences panel already carries several persisted display toggles, and preferences are loaded into app state at startup, changed immediately from the panel, persisted to `config.toml`, included in reset behavior, and surfaced through localized UI text.

Data flow:

1. Preferences load into `AppPreferences` and `MarkionApp` at startup.
2. The Preferences panel toggles `preview_adaptive_width`.
3. Toggling updates app state, persists the preferences file, and rerenders immediately.
4. The preview pane layout checks `view_mode` and `preview_adaptive_width`.
5. In Read mode only, disabled adaptive width centers rendered preview content with a max width of 860px; enabled adaptive width keeps full-width behavior.

## Goals / Non-Goals

**Goals:**

- Make Read mode use a readable default line width, capped at 860px and centered.
- Add a persisted "Preview adaptive width" preference, default off.
- Let the preference restore the current full-width Read mode behavior when enabled.
- Expose the setting in the Preferences panel, reset behavior, and preferences summary.
- Localize all new UI chrome in English and Simplified Chinese.

**Non-Goals:**

- No per-document or per-mode width customization beyond the boolean toggle.
- No slider/input for custom max width in this change.
- No effect on Edit mode or Split Preview mode layout.
- No effect on Markdown parsing, preview-block computation, export output, or derived-state caches.

## Decisions

1. Use a fixed 860px max width for default Read mode.

   Rationale: 860px is a common prose reading constraint and matches the user's requested example. It provides a clear default without adding a more complex configuration surface.

   Alternative considered: expose a numeric width preference. That adds UI and persistence complexity before the project has a broader typography/preferences model.

2. Scope the cap to Read mode only.

   Rationale: Split Preview is an editing aid, where using available pane width is useful and already familiar. Read mode is the mode where prose readability matters most.

   Alternative considered: apply the cap to all preview panes. That would reduce available preview area in Split mode and surprise users who expect the preview pane to match its split width.

3. Store the preference as a boolean in the existing TOML preferences file.

   Rationale: this matches focus mode, typewriter mode, code line numbers, and sidebar visibility. Missing values should fall back to `false` so older config files keep the new readable default without migration work.

   Alternative considered: store a string enum for future extensibility. A boolean maps directly to the requested behavior and avoids overdesign.

## Risks / Trade-offs

- [Risk] Users on very narrow windows could see awkward centering/padding -> Mitigation: max-width should cap only when enough width exists; otherwise the preview uses available width.
- [Risk] Existing users may expect Read mode to remain full-width -> Mitigation: provide the opt-in "Preview adaptive width" preference.
- [Risk] Preference files without the new field could fail if parsing is strict -> Mitigation: default the new field and add round-trip/missing-field tests.
- [Risk] Centering a nested preview container could interfere with scrollbars -> Mitigation: keep the scroll container full-width and apply the max width only to the preview content inside it.
