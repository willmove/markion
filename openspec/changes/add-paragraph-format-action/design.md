## Context

See `proposal.md` for motivation and `specs/markdown-editing/spec.md` for the behavior contract. Markion currently models H1–H6 as GPUI actions backed by shared `MenuShortcut` descriptors, binds them during the complete keymap rebuild, shows them in native and in-window Format menus, and dispatches them through `apply_markdown_format` to `MarkdownDocument::apply_heading`. The document helper already identifies selected line starts, detects ATX markers, adjusts caret/selection offsets as markers change, and applies mutations through the canonical versioned source path. Visual Edit separately has a block-level `Text` transform, but that path requires a versioned block target and is not suitable for a global menu/keyboard action in every editable view mode.

## Goals / Non-Goals

**Goals:**

- Make Paragraph a first-class peer of the heading actions across action dispatch, menus, shortcut customization, localization, and the shortcut reference.
- Reuse the existing line-selection and source-mutation machinery so caret/selection offsets, undo, dirty state, tabs, and cache invalidation behave like other Markdown formatting actions.
- Keep a true no-op free of document-version and cache churn.

**Non-Goals:**

- Reuse the Visual Edit block-target command as the application action, because it cannot represent an arbitrary source selection and intentionally rejects stale visual targets.
- Generalize Paragraph into a universal block-normalization command for lists, quotes, fences, Setext headings, or other Markdown containers.
- Add a special cache or parsed-state update path for this action.

## Decisions

### 1. Add an explicit Paragraph format variant and GPUI action

Introduce a dedicated `MarkdownFormat::Paragraph` variant and a `Paragraph` GPUI action with its own handler. The document-format variant will call a focused helper that visits the same selected line starts as heading formatting and removes only valid ATX heading markers.

Using `Heading(0)` was considered, but the current heading variant clamps levels to 1–6 and semantically represents adding or switching a heading. An explicit variant keeps level validation intact and makes exhaustive matches and tests describe the user-visible command accurately.

### 2. Share marker-removal offset logic with heading formatting

The Paragraph helper will reuse the existing ATX marker-length detection and selection-offset adjustment rules used by `apply_heading`. It will remove a marker only when the line is recognized as an ATX heading, accumulate byte deltas across selected lines, and apply transformed text only if at least one marker was removed. This preserves UTF-8 boundaries and anchors a caret or selection to the same content after prefixes shrink.

A separate parse of preview or Visual Edit blocks was rejected because source-line syntax is sufficient, parsed blocks may be debounced, and consulting them would couple formatting to derived-cache readiness.

### 3. Route the action through the shared shortcut registry

Add a stable `paragraph` descriptor with GPUI binding `secondary-0` and curated labels `Ctrl+0` / `Cmd+0`, include it in the registry's complete collection, and bind it in the same keymap rebuild as H1–H6. This automatically integrates persisted overrides, conflict checks, live rebinding, menu labels, and shortcut-preferences lookup.

The native and in-window Format menus will place Paragraph immediately before H1 in the heading section. The localized Editing shortcut catalog will include Paragraph in the same row/group as the heading commands so the `0`–`6` family is discoverable together.

### 4. Preserve the canonical mutation and cache lifecycle

The data flow is:

`effective shortcut or Format menu → Paragraph action → apply_markdown_format → MarkdownDocument source transform → existing undo/dirty/after-change path → per-version derived caches refresh lazily`

When an ATX marker is removed, the normal document mutation increments the version and invalidates derived state exactly once. When no marker is removed, the document helper does not apply transformed text; the app reports the existing no-formatting-change status and leaves the version, undo stack, cached preview/outline/stats state, syntax-highlight memoization, and cached text handle untouched.

### 5. Localize the action through the existing message catalog

Add distinct Paragraph menu/reference and success-status messages for every supported language, and include them in localization completeness tests. The no-op case will reuse the existing generic no-formatting-change status instead of introducing a second failure message.

## Risks / Trade-offs

- [Lines that begin with hashes but are not valid ATX headings could be stripped accidentally] → Reuse the same ATX recognition helper that heading switching already trusts, and cover no-separator hashes plus indented/code-like lines in document-level tests.
- [Multi-line selections can shift endpoints incorrectly after several prefixes are removed] → Reuse the existing per-line delta accounting and add tests with mixed heading levels, ordinary lines, UTF-8 content, forward ranges, and a caret inside a heading.
- [The action may be registered but omitted from one discovery surface] → Extend registry-count, native-menu, in-window-menu, shortcut-catalog, platform-label, and override/reset tests around the stable `paragraph` id.
- [`Ctrl+0` could conflict with a future zoom-reset command] → The shared conflict checker will reject duplicate effective bindings; Markion currently has no zoom-reset action, and users retain the normal shortcut override path.

## Migration Plan

No data migration is required. Existing installations receive the new default unless a future or manually edited preferences file already defines the `paragraph` id, in which case normal shortcut sanitization applies. Rollback removes the action, registry descriptor, menu/catalog entries, and localization messages; document files and unrelated shortcut overrides remain valid.
