## 1. Preference lifecycle

- [x] 1.1 Add the default-on `slash_commands_enabled` field to the model, tolerant TOML persistence, app initialization/current preferences, and reset; verify true/false round trips and missing/invalid compatibility.

## 2. UI and completion behavior

- [x] 2.1 Add the localized General toggle, immediately dismiss/synchronize slash state when toggled, gate recognition and queued command execution while disabled, and preserve document/history/cache state.

## 3. Verification

- [x] 3.1 Add GPUI regressions for literal ASCII/Chinese slash input while disabled, default-on recognition, live toggle, stale commands, persistence/reset, and unrelated command availability; verify the General control is actionable.
- [x] 3.2 Run formatting, focused regressions and appropriate root tests/build checks, strict OpenSpec validation, and record results and any verification limitations.

## Verification record (2026-11-16, Windows)

- `cargo test --lib --bin markion -- --quiet`: library 756 passed / 1 ignored; application 825 passed / 2 ignored; no failures. Includes all existing slash, emoji, block-menu, preference and localization regressions, the new TOML compatibility test, and four new GPUI preference tests. Log: `target/tmp/slash-preference-qa/root-tests.log`.
- GPUI verification covers default-on `/` and `、` completion, literal input and Enter while disabled, immediate dismissal/re-enable, stale command rejection, document/cache/history preservation, other command surfaces, image-tab safety, and clicking the General toggle through persistence and confirmed preferences reset.
- `cargo check --bin markion` passed. Log: `target/tmp/slash-preference-qa/check.log`.
- `cargo fmt --all -- --check`, `git diff --check`, and `openspec validate add-slash-command-preference --strict` passed.
- UI interaction was verified with GPUI's test window; no separate native desktop session or macOS/Linux runtime check was performed. The three existing ignored root tests remain ignored.
