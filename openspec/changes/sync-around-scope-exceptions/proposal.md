## Why

Today one file outside the approved sync scope stops Sync Now completely. Examples: a new PDF attachment, a macOS `.DS_Store`, a Windows `Thumbs.db`/`desktop.ini`, or anything under a hidden folder. The pre-sync check returns "… is outside the approved synchronization scope", and the engine preflight rejects the plan, so none of the user's notes are saved to history or uploaded.

The Backup and Sync surface marks such a file with a red "attention" label but offers no action on it. The settings form can only edit folder roots; the allowed file kinds (Markdown, text, image) are hard-coded. The user's only ways out are deleting the file or hand-editing Git ignore files outside the app. The `git-workspace` spec already promises that an exception can be resolved by "explicit inclusion, policy adjustment, or exclusion", but none of these exist in the app.

## What Changes

- **Hold back instead of block.** Changes outside the approved scope are left out of the automatic commit and listed as "not included in sync". The rest of the approved notes are saved, committed and synchronized as usual.
  - Held-back **new files** never block anything, except an incoming update to the same path, which is still refused as today.
  - Held-back **tracked changes** (modifications outside tracked roots) do not block committing or uploading other notes. They only pause applying incoming updates, because integration requires a clean tracked state.
- **Moves stay whole.** A tracked deletion whose content exactly matches a held-back new file is held back with it. A moved file can never propagate as a deletion alone.
- **Operating-system artifacts are ignored by default.** Well-known OS-generated files (`.DS_Store`, `._*`, `Thumbs.db`, `desktop.ini`, Office/LibreOffice lock files, and similar) are never proposed as new notes and never shown as exceptions. Already-tracked copies keep following the tracked-path policy.
- **Per-file decisions where the exception is shown.** Each held-back new file offers:
  - **Include once**: commits exactly this content on the next sync; the approval is invalidated if the file changes first.
  - **Always sync .ext files**: adds the extension to the notes policy.
  - **Don't sync on this device**: adds a local-only Git exclusion that does not create a repository change or affect other devices.

  Settings show the extra file types and the device-local exclusions, and allow removing them.
- **Truthful status.** The ordinary Backup and Sync state is computed from approved content. A separate "N files are not included in sync" line with a Review entry is always shown while held-back items exist; it is never folded into a plain "synchronized" message.
- Non-goals:
  - changing the attachment-reference check and its acknowledge flow;
  - committing symlinks or nested repositories (never offered);
  - provider-side ignore management;
  - similarity-based rename detection (only exact-content moves are paired);
  - changing the local-version composer.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `git-workspace`: the personal notes policy holds exceptions back without blocking approved content. It ignores OS artifacts by default, pairs exact-content moves, and offers per-file include-once, always-allow-type and device-local exclusion decisions.
- `git-sync`: status projection reports held-back items as a separate not-included count instead of overriding the primary state. A new requirement covers Sync Now proceeding around held-back items and integration pausing only for held-back tracked changes or path collisions.

## Impact

- `crates/git-sync`:
  - `policy.rs`: scope partition, OS-artifact list, `allowed_extensions` and `included_once` policy fields (serde-defaulted, schema version unchanged), device-local exclusion helpers;
  - `engine.rs`: preflight partitions; integration tolerates held-back untracked paths; new error for held-back tracked changes; held-back count in the synchronized outcome;
  - `model.rs`: presentation fact and field for held-back items;
  - `repository.rs`: batched `hash-object --stdin-paths` for move pairing.
- `src/app/git_sync.rs`: precheck partitions instead of failing; held-back buffers stay unsaved.
- `src/app/git_panel.rs`: held-back rows with reasons and the three actions, summary line, settings fields for file types and device exclusions.
- `src/i18n/git.rs`: new strings in all seven languages, covered by the existing "Backup, sync, and recovery chrome SHALL be fully localized" requirement (no `ui-i18n` delta).
- Persisted data: two optional policy fields. Older builds ignore them, but drop them if they rewrite the policy file.
- Invariants: no typing-path or rendering work; all Git/file work stays in background executors; `markion-git-sync` stays gpui-free.
- Ordering: independent of `reduce-git-refresh-overhead`. If that change lands first, use its batched path helpers for the new checks.
