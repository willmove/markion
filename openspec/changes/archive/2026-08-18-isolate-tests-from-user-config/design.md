## Context

See `proposal.md` for motivation. `MarkionApp::new` already skips loading and saving `session.toml` under `cfg!(test)`, but it still loads `default_preferences_path()` and `persist_preferences` writes that path unconditionally. Neighboring typography tests redirect `preferences_path` to a tempfile; `source_layout_snapshot_maps_wrapped_utf8_content_bidirectionally` does not, then calls `set_editor_font_size(28)`, which overwrites the developer's `config.toml`. Preference mutation is not on the keystroke/derived-cache path; this change only affects test construction and the persist guard.

## Goals / Non-Goals

**Goals:**

- Tests that forget to redirect `preferences_path` cannot read or write the real `config.toml`.
- Tests that do redirect to a tempfile keep working (they already cover persist round-trip).
- Test construction becomes deterministic: documented defaults, not the developer's live theme/language/font.

**Non-Goals:**

- Do not no-op every `persist_preferences` call in tests (that would break tempfile round-trip tests).
- Do not isolate `themes/`, recovery, or logs in this change.
- Do not migrate other machines' `config.toml` files.

## Decisions

### 1. Mirror session isolation, with a path-aware persist guard

`persist_session` returns immediately when `cfg!(test)`. Preferences cannot copy that blindly: several tests assign `app.preferences_path` to a tempfile and then call `set_editor_font_size` / `persist_preferences` to assert TOML output.

Chosen approach:

```
MarkionApp::new under cfg!(test):
  preferences_path = default_preferences_path()   // same field as production
  preferences      = AppPreferences::default()    // do not load the real file

persist_preferences under cfg!(test):
  if self.preferences_path == default_preferences_path() { return; }
  // otherwise write (tempfile tests)
```

**Alternatives considered:**

| Option | Why not |
|--------|---------|
| No-op all `persist_preferences` in tests | Breaks existing tempfile persist tests unless they call `save_app_preferences` directly. |
| Give every `MarkionApp::new` a unique `TempDir` | Extra struct lifetime / cleanup; tests that already override the path would ignore it. |
| Only fix the 28px test | Leaves the next `set_*` / `toggle_*` that forgets a tempfile free to clobber theme, language, or shortcuts. |

The path-aware guard is the belt; loading defaults is the suspenders. Either one alone still leaks (load-real then persist-skip keeps tests flaky on developer settings; persist-skip without default-load still *reads* the real file).

### 2. Redirect the known 28px test to a tempfile anyway

`typography_changes_preserve_document_caches_and_list_positions` and `non_default_editor_font_reflows_wrapped_text_and_caret_geometry` already use tempfile. Make `source_layout_snapshot_maps_wrapped_utf8_content_bidirectionally` match, so that test is consistent even if someone later removes the persist guard.

### 3. Regression test asserts the real file is unchanged

A GPUI test constructs `MarkionApp` without overriding `preferences_path`, snapshots the real `config.toml` bytes (or its absence), calls `set_editor_font_size(28)`, and asserts the real file is bit-identical (still absent, or same bytes). That locks the leak closed without depending on a developer-specific starting value.

### 4. Local 14px recovery is operator-side, not a shipped migration

The leaked `editor_font_size = 28` on this developer machine is restored to 14 in `%APPDATA%\Markion\config.toml`. Production code MUST NOT rewrite other users' persisted font sizes. After the guard lands, re-running the 28px test MUST leave that 14 in place.

## Risks / Trade-offs

- **[Tests that silently depended on the developer's live `config.toml`]** → They will now see documented defaults (English, Paper, 14px source). That is the intended determinism; any assertion that assumed `zh-hans` or a custom theme from the developer file should be updated to set the value explicitly.
- **[Guard compares paths and Windows path canonicalization differs]** → Compare using the same `default_preferences_path()` helper both at construction and at persist; tests that override the path use a tempfile under a different directory, so equality cannot match by accident.
- **[Someone writes through `save_app_preferences(&default_preferences_path(), …)` in a test]** → Out of scope for the `MarkionApp` guard; the regression test covers the app persist path that actually leaked. Do not add a global hook inside `save_app_preferences` (production and tempfile tests share it).

## Migration Plan

1. Restore this machine's leaked `editor_font_size` to 14 (already done in `%APPDATA%\Markion\config.toml`).
2. Land the load + persist guards and the regression test.
3. Re-run the former leaking test; confirm the real `config.toml` still has `editor_font_size = 14`.
4. No rollback schema: production persist behavior is unchanged.

## Open Questions

(none)
