## Why

Markion already auto-saves after inactivity and keeps crash-recovery snapshots, but the only durable control is `[auto_save] enabled` in `config.toml`, and that single switch disables **both** silent write-back to the original file and recovery. Users who want to stop silent overwrites of named files still need crash recovery, and they need to set the inactivity interval without editing a TOML file.

## What Changes

- Split silent destination write-back from crash recovery via a new `[auto_save] silent_save` preference (default `true`). When `enabled = true` and `silent_save = false`, inactivity still writes/updates a recovery snapshot for dirty tabs (named or untitled) but **does not** write the original file path; the tab stays dirty until a manual save succeeds.
- Keep `[auto_save] enabled` as a file-only master switch (timer + recovery). The Preferences panel does **not** expose it.
- Expose in Preferences → General: a toggle for silent save-to-file (`silent_save`) and a numeric control for the inactivity interval (`delay_secs`, default 5, minimum 1). Changes apply immediately and persist through the existing preferences path.
- Status feedback for recovery-only autosaves of named files uses the recovery-saved messaging path, not “auto-saved to path”.
- Localize every new Preferences and status string in all supported UI languages.
- Update end-user docs (`docs/faq.md` and any config schema mention) so `silent_save` and the panel controls are documented.

**Non-goals:** multi-version backups / `.bak` files; exposing `enabled` in the panel; changing manual Save / Save As; changing recovery-manager restore UX beyond what recovery-only autosave already implies.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `workspace`: Auto-save/recovery requirement splits silent write-back (`silent_save`) from recovery snapshots; panel configures `silent_save` and `delay_secs`, while `enabled` remains file-only.
- `chrome-platform`: `[auto_save]` schema gains `silent_save`; `silent_save` and `delay_secs` become Preferences-panel–configurable (no longer “file only” for those two fields).
- `theme-preferences`: Preferences General surface gains silent-save toggle and delay interval controls.
- `ui-i18n`: New Preferences labels / status feedback for silent save and delay are localized in every supported language.

## Impact

- Specs: `workspace`, `chrome-platform`, `theme-preferences`, `ui-i18n`.
- Code: `AutoSavePreferences` / TOML serialize-parse (`src/model.rs`, `src/storage/preferences.rs`); autosave background stage gate in `src/app/application.rs` (`run_autosave`); Preferences General UI + setters/persistence; `src/i18n.rs`; tests for preference round-trip and `silent_save = false` recovery-only behavior.
- Docs: `docs/faq.md` auto-save section.
- Compatibility: configs omitting `silent_save` default to `true` (current behavior). No **BREAKING** removal of `enabled` or `delay_secs`.
- Invariants: derived Markdown caches, syntax memoization, and text-handle reuse stay untouched; autosave still runs off the UI thread and still writes recovery before any destination save when silent save is on.
