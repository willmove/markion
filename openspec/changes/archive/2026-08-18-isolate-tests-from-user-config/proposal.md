## Why

GPUI tests construct `MarkionApp` against the developer's real `config.toml` (`%APPDATA%\Markion\config.toml` on Windows) and `set_editor_font_size` persists immediately. One layout test writes `editor_font_size = 28`, so every `cargo test` clobbers the source font size the next time the desktop app launches. Session files already skip this trap; preferences do not.

## What Changes

- Isolate `MarkionApp` test construction from the developer machine's real preferences file: tests MUST load default preferences rather than `default_preferences_path()`, matching the existing `session.toml` guard.
- Make `persist_preferences` refuse to write the real default preferences path under `cfg!(test)`, while still allowing tests that redirect `preferences_path` to a tempfile to round-trip `config.toml`.
- Add a regression test that a preference-mutating GPUI test cannot change the real `config.toml` (the current 28px write is the fixture).
- Point the known leaking test (`source_layout_snapshot_maps_wrapped_utf8_content_bidirectionally`) at a tempfile the same way neighboring typography tests already do.
- Restore this developer's leaked `editor_font_size = 28` back to the documented 14px default in the local `config.toml`. That is a one-time local recovery, not a shipped migration of other users' files.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `engineering-quality`: add a requirement that the workspace test suite MUST NOT read or write the developer machine's real preferences file (`config.toml` under `default_config_dir()`), with the same isolation already applied to `session.toml`.

## Impact

- `src/app/application.rs` — `MarkionApp::new` test-path preference loading; `persist_preferences` guard parallel to `persist_session`.
- `src/app/tests.rs` — leaking 28px test plus a regression that the real `config.toml` is unchanged after preference mutation.
- No production preference schema, defaults (`DEFAULT_EDITOR_FONT_SIZE` stays 14), or Preferences UI change.
- Does not touch Markdown derived-state caches, syntax-highlight memoization, or the cached text handle.

## Non-goals

- Not migrating existing users' persisted `editor_font_size` (or any other `config.toml` field) to 14px.
- Not changing the 10–32px clamp, reading font size, or paragraph spacing.
- Not isolating `themes/`, recovery files, or logs; those stay out of this change unless a test is already writing them.
- Not rewriting every GPUI test to use an explicit tempfile; the default test construction becomes safe, and only tests that need persistence keep overriding the path.
