## Context

P0 established same-directory atomic writes, content-confirmed disk identities, external-change conflict actions, recovery-v2 snapshots, recovery-before-autosave, and saved-tab session restoration. The P1 work must build on those paths instead of adding a second persistence layer. The remaining recovery UI reads every file but asks for one all-or-nothing decision and deletes successfully loaded snapshots immediately; a crash before the restored dirty tab writes its next snapshot can therefore lose the only recovery copy. Atomic replacement also creates a fresh temporary inode/file and currently leaves destination permission preservation to platform defaults.

Visual Edit already derives exact `VisualBlock` source ranges and stable ephemeral `VisualBlockId`s per document version. Those are suitable validation tokens for block commands, but Markdown text remains canonical: UI state never owns a rendered tree and no block action may mutate cached blocks directly.

## Goals / Non-Goals

**Goals:**

- Provide selective, inspectable, durable management for the complete recovery inventory.
- Preserve destination permissions during atomic replacement where supported.
- Provide a discoverable slash palette, complete common block transformations, and both button/keyboard and drag reorder paths.
- Make every block mutation one UTF-8-safe canonical source edit with one undo entry and ordinary dirty/autosave/recovery/cache behavior.
- Reject stale, overlapping, nested, or otherwise ambiguous targets without mutating source.

**Non-Goals:**

- Persisting UI-only palette/menu/drag state, introducing a rich-text/AST canonical model, or guessing transformations for unsupported syntax.
- Collaborative file locks, remote recovery synchronization, workspace file moves, or changing the existing recent-file/session storage format.
- Dragging blocks in source Edit/Split modes; their source selection remains the complete fallback.

## Decisions

### 1. Recovery inventory is ephemeral UI over durable files

Startup scans recovery paths into lightweight rows containing path, timestamp, original path, parse state, and current disk relationship. Restore loads the chosen snapshot on demand; unreadable rows remain visible and durable until explicitly discarded. Restore All processes every readable row and retains failures.

A restored tab keeps `last_recovery_file` pointing at the original snapshot. The file is removed only after a durable normal save, explicit tab/recovery discard, or a successfully written successor snapshot. If the original path already belongs to a clean session-restored tab, that tab is replaced in place and activated, preserving path uniqueness. This is preferred over opening duplicate path-backed tabs or deleting the snapshot eagerly.

### 2. Atomic replacement copies destination permissions to the temporary file

Before replacement, the atomic helper reads the existing destination permissions and applies them to the fully written temporary file, then performs the same atomic replace. Failure to copy permissions aborts before replacement and retains the old destination. New files continue to inherit normal directory defaults. This is a narrow extension of the existing helper rather than another save path.

### 3. Block operations use immutable validated targets

UI events carry `document_version`, `VisualBlockId`, and the observed source range. At execution the model resolves the current visual block and requires the same version/id/range plus a supported non-overlapping shape. Presentation-only selection, hover, menu navigation, and drag movement do not change the document version or derived-cache identity.

Data flow:

`canonical Markdown -> cached Arc<Vec<VisualBlock>> -> BlockTarget(version/id/range) -> validate -> one source replacement -> normal invalidation/undo/autosave`

For ordinary paragraph, heading, top-level list/task item, quote leaf, code block, rule, and whitespace blocks, a narrow source transformer removes only the proven structural prefix/payload and serializes the requested target. Unsupported HTML/front matter, ambiguous quote groups, and overlapping/nested ranges keep source editing as the fallback.

### 4. Slash commands are a transient query over the current empty block

In Visual Edit, a collapsed caret whose current line consists only of optional indentation plus `/query` opens the palette. The slash text remains canonical until a command is confirmed. Up/Down change only the selected palette row, Escape closes only the palette, and Enter/mouse confirmation replaces the slash query with one canonical block template. The command set is fixed and localized: Text, H1-H6, Bulleted List, Numbered List, Task List, Quote, Code Block, Divider, and Table.

Keeping the query in Markdown avoids a hidden input buffer and preserves platform text/IME behavior. A version/range mismatch closes the palette without mutation.

### 5. Reordering operates on proven source units

The reorder helper expands a supported visual block to a complete newline-safe source unit and includes separator whitespace deterministically. It rejects overlapping source ranges, quote-group leaves, nested list items, stale identities, and a drop into the dragged unit. Move Up/Down and drag before/after call the same helper, so keyboard-accessible and pointer paths have identical serialization and undo semantics. Nested/ambiguous structures remain reorderable manually in source mode.

### 6. Row chrome is contextual and cache-neutral

The focused supported Visual Edit row exposes a drag grip and compact block menu. The menu provides Turn Into, Duplicate, Delete, Move Up, and Move Down. Slash and block menus are ephemeral `MarkionApp` state keyed by version/id; they are cleared on document mutation, tab/mode change, undo/redo, or stale validation. No row-local state is written to Markdown or persisted sessions.

## Risks / Trade-offs

- **[Markdown boundaries can overlap for nested/quoted structures]** → Require non-overlapping exact ranges and disable reorder/transform chrome when proof fails; source mode remains complete.
- **[Moving separator whitespace can change visual spacing]** → Use one shared pure unit builder with CRLF/LF and leading/trailing blank-line tests; button and drag paths share it.
- **[Recovery list may contain corrupt or very large files]** → Inventory metadata first, load content on restore, retain failures, and bound visible rows through a scrollable manager.
- **[Permission copying differs by platform]** → Use `std::fs::Permissions`; add portable read-only coverage and Unix mode-bit coverage where available, while retaining atomic failure behavior.
- **[Slash interception can conflict with literal slash prose]** → Activate only when the entire current line is a slash query and close without source changes on Escape; continued prose makes the predicate false.
- **[Virtualized row drag targets can disappear]** → Use stable IDs/version validation and drop zones only on rendered rows; stale/off-screen targets become no-ops.

## Migration Plan

No persisted Markdown or recovery format migration is required. Recovery-v1/v2 reading remains compatible. Existing recovery files appear in the new manager. Rollback leaves those snapshots readable by the prior loader; P1 UI state is ephemeral.

## Open Questions

None. Conservative target rejection is the default whenever exact source ownership is uncertain.
