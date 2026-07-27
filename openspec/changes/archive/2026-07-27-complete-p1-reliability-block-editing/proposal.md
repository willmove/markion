## Why

P0 made ordinary saves and external conflicts safe, but the remaining P1 reliability promise is not complete: startup recovery is still an all-or-nothing prompt, restored snapshots are retired before another durable recovery exists, unreadable snapshots cannot be managed individually, and atomic replacement does not preserve existing destination permissions. Visual Edit also lacks the block-level authoring interactions expected from a writing-first editor: discoverable slash commands, explicit block transformations, and source-safe block reordering.

## What Changes

- Replace the all-or-nothing recovery prompt with a localized recovery manager that inventories every snapshot, identifies its original path and disk relationship, and supports Restore, Discard, Restore All, and Discard All without deleting unreadable or unselected recovery data.
- Keep a restored recovery snapshot durable until a successful document save, explicit discard, or atomically written successor recovery supersedes it; reuse the matching session-restored tab instead of creating a duplicate path-backed tab.
- Preserve existing destination permissions across atomic document and settings replacement where the platform exposes them.
- Add a keyboard- and pointer-operable Visual Edit slash-command palette for paragraph, headings, bulleted/numbered/task lists, quote, fenced code, divider, and table blocks.
- Add exact, current-version block transformations plus duplicate, delete, move up/down, and drag/drop reorder controls. Reordering is exposed only when non-overlapping source boundaries are proven; ambiguous nested/quoted structures remain editable through canonical source.
- Route every new label, recovery state, command name, and status through all supported localization catalogs.

Non-goals: a parallel rich-text document model, arbitrary AST rewriting, cross-machine recovery sync, directory/workspace moves, collaborative locking, or guessed block mutations for overlapping/ambiguous source ranges.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `reliable-file-persistence`: Complete multi-snapshot recovery management and preserve destination permissions during atomic replacement.
- `markdown-editing`: Add the full source-backed slash-command, block transformation, and safe reorder experience to Visual Edit.
- `ui-i18n`: Require complete localization of P1 recovery and block-authoring chrome.

## Impact

- Persistence: `src/storage/atomic.rs`, `src/storage/recovery.rs`, recovery lifecycle and startup/session integration in `src/app/application.rs`.
- Markdown model: a GPUI-free exact block command/reorder module and `MarkdownDocument` mutation entry points, preserving per-version cache invalidation and cheap undo snapshots.
- GPUI application: new ephemeral recovery-manager, slash-palette, block-menu, and drag target state; Visual Edit row chrome and keyboard routing.
- Tests/docs: pure source mutation and filesystem tests, rendered GPUI interaction tests, localization completeness, and the maintained Visual Edit support matrix.
