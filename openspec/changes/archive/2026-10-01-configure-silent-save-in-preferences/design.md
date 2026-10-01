## Context

Autosave already runs in two stages in `run_autosave` (`src/app/application.rs`): write a recovery snapshot first, then—if the tab has a path—silently save to that destination and retire the recovery file. Scheduling is gated only by `[auto_save] enabled` and `delay_secs` on `AutoSavePreferences`; both are persisted under `[auto_save]` in `config.toml` but are **not** in the Preferences panel (General / Shortcuts / Export). Specs currently treat “disable auto-save” as disabling both write-back and recovery.

Users want to keep crash recovery while stopping silent overwrite of named files, and to adjust the interval from Preferences.

## Goals / Non-Goals

**Goals:**

- Add `silent_save: bool` (default `true`) to `[auto_save]` / `AutoSavePreferences`.
- When `enabled && !silent_save`, inactivity still writes recovery for dirty tabs; named paths are not written; dirty stays set; status uses recovery messaging.
- Preferences → General exposes: toggle for silent save-to-file, numeric control for `delay_secs` (default 5, clamp ≥ 1).
- Leave `enabled` file-only (master kill for timer + recovery).
- Persist through existing preferences load/save/reset/summary paths; localize new strings; document in FAQ.

**Non-Goals:**

- Exposing `enabled` in the panel.
- Versioned backups, sibling `.bak` files, or multi-snapshot history per file.
- Changing Recovery Manager restore UX, manual Save semantics, or external-change conflict handling beyond “no silent write when `silent_save` is off”.
- Touching derived Markdown caches / syntax memo / text-handle invariants.

## Decisions

### 1. Additive `silent_save` field (not rename of `enabled`)

Keep `enabled` as the schedule/recovery master switch. Add `silent_save` that only gates the destination `save_text_snapshot` step after recovery succeeds.

```
enabled=false  → no timer / no recovery / no write-back (unchanged)
enabled=true, silent_save=true  → recovery → write path → clear dirty (unchanged)
enabled=true, silent_save=false → recovery only → keep dirty
```

**Alternatives considered:** Reinterpret `enabled` as “write-back only” and always keep recovery — rejected: breaks existing `enabled = false` users who want nothing automatic. Separate `recovery_enabled` + `save_to_file` both in panel — rejected: user chose not to expose the recovery master in UI.

### 2. Gate in `run_autosave`, pass flag on `AutosaveRequest`

Capture `silent_save` on the UI thread into `AutosaveRequest` (same pattern as path/text/generation). After a successful recovery write:

- if `path` is `None` → `RecoveryOnly` (untitled; unchanged)
- if `path` is `Some` and `!silent_save` → `RecoveryOnly` (named; **new**), keep recovery file
- if `path` is `Some` and `silent_save` → existing destination save / delete recovery / `Saved` or `SaveFailed`

`apply_autosave_outcome` already treats `RecoveryOnly` as “record `last_recovery_file`, do not clear dirty” — reuse that path so named recovery-only tabs stay dirty and keep a durable snapshot (aligns with `reliable-file-persistence`).

**Alternatives considered:** Skip scheduling when `!silent_save` for named tabs — rejected: would drop recovery. Branch only in `apply_*` after writing the file — rejected: must not write the file at all.

### 3. Preferences UI on General, mirror existing control idioms

- Boolean: same toggle/button affordance as Sync scroll / Show hidden files (`preference_option_button` or equivalent).
- `delay_secs`: same stepper pattern as typography sizes / PDF margin (decrement/increment, disable at bound). Bound: minimum 1 (match `Duration::from_secs(...max(1))`); reasonable upper bound (e.g. 300) to avoid accidental huge delays—document in tasks if chosen.
- Label copy: “Auto-save to file” (or localized equivalent), not the raw key `silent_save`, so the boolean polarity stays clear (`true` = do silent write-back).
- On change: update in-memory `auto_save_preferences`, `persist_preferences()`, bump/reschedule as needed (toggling `silent_save` off should not require restart; next `schedule_autosave` / in-flight apply already reads captured request fields).

`enabled` remains absent from the panel and from preference-summary “editable” emphasis if summary lists panel settings; FAQ still documents the file-only master switch.

### 4. Defaults and migration

- Missing `silent_save` in TOML → `true`.
- Reset preferences → `enabled=true`, `silent_save=true`, `delay_secs=5`.
- Legacy `preferences.conf` migration: no new key; defaults apply after migrate (same as today for auto_save).

### 5. Docs

Update `docs/faq.md` auto-save section: explain `silent_save`, panel controls, and that `enabled = false` still disables everything including recovery.

## Risks / Trade-offs

- **[Named files stay dirty forever while typing if silent_save is off]** → Intended; title-bar `*` and close/quit dirty guards remain the reminder. Mitigation: clear Preferences label + FAQ.
- **[Recovery dir growth for long-lived dirty named tabs]** → Still one atomically replaced snapshot per tab (`recovery_id`), same as untitled; successful manual save retires it.
- **[Users confuse “Auto-save to file” with disabling all protection]** → Do not expose `enabled` in panel; FAQ states recovery continues when silent save is off.
- **[Status bar says “auto-saved path” incorrectly]** → Route named `RecoveryOnly` through recovery status strings only.
- **[Older `MarkdownDocument::autosave` helper still forks save-vs-recovery]** → Prefer routing production through `run_autosave`; if the document helper remains for tests/legacy, update or stop using it for silent_save semantics so tests don’t diverge.

## Migration Plan

1. Ship additive TOML field + runtime gate; default preserves behavior.
2. Add Preferences controls and i18n.
3. Update FAQ.
4. Rollback: remove UI and ignore `silent_save` (or treat missing as true); no data migration required.

## Open Questions

None — panel scope (`silent_save` + `delay_secs` only) and field name (`silent_save`) are decided.
