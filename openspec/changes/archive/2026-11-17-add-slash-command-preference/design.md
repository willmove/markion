## Context

See proposal.md for motivation. Slash popup state is derived in `sync_slash_command_state` during render and editor actions. Preferences use an app-level field mirrored to `AppPreferences` and a tolerant serde-facing TOML shape. General already contains boolean controls for editing behavior.

## Goals / Non-Goals

**Goals:** Use the existing preference lifecycle and gate popup state before any slash query or cached block lookup.

**Non-Goals:** Change parser/query semantics, completion layout, canonical text edits, or existing insertion editors after they have already opened.

## Decisions

- Add `slash_commands_enabled: bool`, default true, through model, TOML conversion, app startup/current preferences, and reset. Reuse `deserialize_bool_or_true` so old and hand-edited configuration stays compatible. A document-specific flag would not match the requested system preference.
- Add a General boolean row with a message in `src/i18n.rs` for all supported languages. Reuse the existing themed row and persistence path rather than adding a separate dialog.
- Gate synchronization at its entry, excluding image tabs before accessing document editing state. The toggle clears popup/dismissed-query state and synchronizes under the new value so disabling is immediate and re-enabling can recognize the current query. Guard command execution too, covering stale queued pointer actions.
- Data flow: `config.toml -> AppPreferences -> MarkionApp -> General row / slash state gate`; a toggle updates the field, clears transient completion state, synchronizes, saves current preferences, and notifies the view. Reset restores the default field before the next render. No document mutation, history operation, parsing, or derived-cache invalidation occurs; enabled recognition continues reusing the existing versioned block cache.

## Risks / Trade-offs

- [State can reappear on render] -> Gate the shared synchronization path rather than hiding only the rendered popup.
- [Queued pointer event can reference a dismissed popup] -> Reject slash execution while disabled.
- [Shared workspace already contains uncommitted editing changes] -> Keep patches scoped and run existing slash/emoji/block regressions as well as preference tests.

## Migration Plan

No migration is required: missing fields default to enabled. Older builds ignore the new key. Reset returns to enabled through the existing default preferences flow.
