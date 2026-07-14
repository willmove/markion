# Implementation Plan: TOML config, tracing logs, configurable auto-save (Typune integration Phase 3)

## Overview

Adopt the Typune `filesystem` crate's designs (TOML schema, logger blueprint, configurable auto-save) inside Markion's own `storage/` layer. The crate itself is not copied (rfd/GTK would break the Linux release build; Typune branding; alien schema — see proposal).

## Tasks

- [x] 1. Dependencies (root `Cargo.toml`)
  - [x] 1.1 `[workspace.dependencies]` gains `toml`, `tracing-subscriber` (env-filter), `tracing-appender`; root package adds `serde`, `toml`, `tracing`, `tracing-subscriber`, `tracing-appender`.

- [x] 2. TOML preferences (`src/storage/preferences.rs`, `src/model.rs`, `src/paths.rs`)
  - [x] 2.1 `AppPreferences` gains `auto_save: AutoSavePreferences { enabled: true, delay_secs: 5 }` (plain struct, model stays serde-free).
  - [x] 2.2 Serde DTO with full `#[serde(default)]` coverage; `render_app_preferences` emits TOML; `parse_app_preferences` parses TOML; legacy parser kept as `parse_legacy_app_preferences`.
  - [x] 2.3 `default_preferences_path()` → `config.toml`; `legacy_preferences_path()` → `preferences.conf`; `load_app_preferences` migrates legacy → TOML once (legacy file left in place, ignored afterwards).
  - [x] 2.4 Tests: TOML round-trip incl. custom_theme/None and sidebar tab; partial file defaults; `[auto_save]` parsing; migration writes `config.toml`; legacy parser tests retargeted.

- [x] 3. Configurable auto-save (`src/main.rs`)
  - [x] 3.1 App keeps the loaded `auto_save` prefs; `current_preferences()` round-trips them; `schedule_autosave` skips when disabled and uses `delay_secs`.

- [x] 4. Logging (`src/storage/logging.rs` new, `src/paths.rs`, `src/main.rs`)
  - [x] 4.1 `default_log_dir()` per platform (Markion-branded); `init_logging()` — daily rotation keep 7, plain-text file layer, compact console layer, `RUST_LOG` override (default info), failures non-fatal.
  - [x] 4.2 Initialize at top of `main()`; startup info event with version + log dir.
  - [x] 4.3 Events: preference migration (info), preference load failure (warn), auto-save failure (warn), export engine fallback with reason (`src/export.rs`), highlight warm-up timing (debug).

- [x] 5. Verification
  - [x] 5.1 `cargo test --workspace` fully green.
  - [x] 5.2 Manual: run a headless-safe check that `init_logging` writes a log file to a temp dir (env-forced), and that a legacy `preferences.conf` migrates to `config.toml` end-to-end.
  - [x] 5.3 Update `docs/typune-integration-plan.md` Phase 3 status with the scope correction.
