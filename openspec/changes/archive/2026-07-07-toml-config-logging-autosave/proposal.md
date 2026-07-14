## Why

Phase 3 of `docs/typune-integration-plan.md`: adopt Typune's `filesystem` crate strengths — TOML configuration, tracing-based rotating file logs, configurable auto-save. Markion today persists preferences in a hand-written `key=value` text format, has **no logging at all**, and hard-codes the 5-second auto-save timer.

**Scope correction (same pattern as Phases 1–2):** the `filesystem` crate is **not** copied wholesale. Three reasons, discovered by inspection:

1. **It would break the Linux release build.** The crate depends on `rfd` (GTK3 file dialogs on Linux, plus `cocoa`/`objc` on macOS and `winapi` on Windows). The release workflow's apt list has no `libgtk-3-dev`, and Markion needs none of it — GPUI's native `prompt_for_paths` already covers dialogs.
2. **It is Typune-branded throughout** — log directories (`MarkdownEditor`/`markdown-editor`), a `.typune_running` crash sentinel, `Typune` config dirs — all of which would need rewriting anyway.
3. **Its `Config` schema is not Markion's.** Markion's preferences (theme/custom theme/language/focus/typewriter/line numbers/sidebar) are authoritative; forcing them into Typune's struct buys nothing. Its tokio-based `AutoSaver` duplicates a mechanism Markion already has working on GPUI timers.

What Phase 3 actually absorbs is the crate's **designs**: the TOML schema layout (top-level fields plus an `[auto_save]` table with `enabled`/`delay_secs`), the logger blueprint (daily rotation, keep 7 files, `RUST_LOG` override, console + file layers), and the configurable-auto-save semantics — implemented in Markion's existing `storage/` layer, Markion-branded, with Markion's defaults (5s, not Typune's 8s).

## What Changes

- **Preferences become TOML** (`src/storage/preferences.rs`). Canonical file moves from `preferences.conf` (hand-written `k=v`) to `config.toml` (serde + `toml`), all fields defaulted so partial files parse. On first load, if `config.toml` is missing but a legacy `preferences.conf` exists next to it, the legacy file is parsed, converted, and written out as `config.toml` (the old file is left in place, ignored thereafter). The legacy parser is kept (renamed `parse_legacy_app_preferences`) as the migration reader.
- **Auto-save becomes configurable via the config file** (`src/model.rs`, `src/main.rs`). `AppPreferences` gains an `auto_save` section (`enabled`, `delay_secs`; defaults `true`/5 — Markion semantics win per plan §Phase 3). `schedule_autosave` respects both. No Preferences-panel UI for it (config-file only), matching the plan's "落点" intent.
- **Tracing file logging** (`src/storage/logging.rs`, new; `src/paths.rs`). Daily-rotated plain-text logs (keep 7), Markion-branded platform dirs (Linux `~/.cache/markion/logs`, macOS `~/Library/Logs/Markion`, Windows `%LOCALAPPDATA%\Markion\Logs`), `RUST_LOG` env override (default `info`), compact console layer for dev runs. Initialized at the top of `main()`. Deviations from Typune's logger: plain-text file format instead of JSON (user-serviceable logs for a desktop app), and no crash sentinel (Markion's recovery subsystem already covers crash handling).
- **First log events** where they earn their keep: startup (version, log dir), preference load/migration, auto-save failures, export-engine fallbacks (`src/export.rs` now reports *why* pandoc was skipped), highlight grammar warm-up timing.
- **Root `Cargo.toml`**: add `serde`/`toml`/`tracing`/`tracing-subscriber`/`tracing-appender` (workspace-managed; `toml`, `tracing-subscriber`, `tracing-appender` join `[workspace.dependencies]`).
- **Non-goals:** no `crates/filesystem` copy; no rfd; no config-file watcher/hot-reload; no keybindings/font sections yet (the TOML shape leaves room); no crash-report UI.

## Capabilities

### Modified Capabilities
- `chrome-platform`: the persisted preferences file becomes `config.toml` (TOML) with automatic one-time migration from the legacy format; adds a diagnostic file-logging requirement (previously an explicit non-goal).
- `workspace`: auto-save is no longer a fixed 5-second timer with no preference — the interval and an enable/disable flag come from the config file (defaults preserve current behavior).

## Impact

- **Edited:** `src/storage/preferences.rs` (TOML + migration), `src/storage/mod.rs`, `src/model.rs` (auto_save section), `src/paths.rs` (config.toml path, legacy path, log dir), `src/main.rs` (logging init, configurable autosave, preference field), `src/lib.rs` (re-exports, tests), `src/export.rs` + `src/highlight.rs` (log events), root `Cargo.toml`. **New:** `src/storage/logging.rs`.
- **User-visible:** existing users' preferences carry over silently (migration); logs start appearing under the platform log dir; auto-save behavior unchanged by default.
- **Typing-path invariants:** untouched — logging happens on I/O and lifecycle events, not per keystroke.
- **Tests:** legacy-format tests retargeted to the renamed parser; new tests for TOML round-trip, partial-file defaults, `[auto_save]` parsing, and the one-time migration.
