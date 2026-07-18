## Context

The completed Visual Edit changes established a source-backed mixed projection: `MarkdownDocument.text` is canonical, `VisualBlock` metadata is cached per document version, and visible rows derive ephemeral rendered/source projections from the active source selection. This already hides supported Markdown markers, reveals one exact local group when focused, routes platform input through the shared `EntityInputHandler`, and retains conservative source islands.

Four interaction gaps remain:

- Hidden marker ranges collapse several source offsets onto one display boundary, but the selection model stores only a byte offset and cannot preserve which visual side of that boundary the caret belongs to.
- Up/Down and Home/End use logical source lines, so they skip painted wrapped lines and do not retain a preferred horizontal position across blocks.
- `marked_range` participates in source mutation but not in the Visual Edit projection or paint path, so active IME composition is not visibly marked and range geometry is reduced to the last caret rectangle.
- Every platform replacement captures a separate undo entry, including intermediate IME updates and contiguous typing.

The desired data flow is:

```text
MarkdownDocument.text + version-cached VisualBlock metadata
  + source selection / marked range / caret affinity
  -> ephemeral VisualProjection with boundary candidates
  -> shaped visual layout + registered navigation geometry
  -> pointer / keyboard / IME intent
  -> existing source mutation path + semantic undo group
  -> MarkdownDocument.text + next version only when text changed
```

## Goals / Non-Goals

**Goals:**

- Make Visual Edit the closest safely achievable WYSIWYG surface while retaining exact authored Markdown as the only mutable document model.
- Preserve the intended inside/outside side of collapsed hidden-syntax boundaries for pointer placement, arrows, marker reveal, and subsequent typing.
- Make vertical and line-boundary movement follow painted wrapped lines and retain a preferred horizontal coordinate across visual blocks.
- Show and position IME composition from the active projected layout and make one composition one undoable action.
- Coalesce compatible contiguous typing without merging structural or otherwise atomic commands.
- Keep all new interaction state outside document-version-derived caches.

**Non-Goals:**

- Incremental parsing, rope adoption, or stable visual block identities across document versions.
- Direct visual editors for table cells, code blocks, images, HTML, front matter, math blocks, or diagrams.
- A mutable rich-text tree, canonical Markdown reserialization, multi-cursor editing, or plugin-defined block types.
- Changing the persisted Markdown format or Read mode mutability.

## Decisions

### 1. Treat WYSIWYG as a presentation and interaction policy, not a second document model

Visual Edit will prefer rendered editing whenever a source mutation and source/display mapping are exact. It will reveal the smallest complete syntax group needed for the active operation and use a source island only when exact behavior cannot be proven. `MarkdownDocument.text`, source byte ranges, and the existing dirty/autosave/recovery path remain authoritative.

An attributed inline tree as a second mutable model was rejected because it would require canonical reserialization, could rewrite authored delimiter choices and whitespace, and would introduce synchronization and undo problems across modes.

### 2. Represent projection boundaries with source candidates and caret affinity

Extend the GPUI-independent projection model so a collapsed display boundary can expose both its upstream and downstream canonical source candidates. Add per-tab ephemeral caret affinity that records which candidate owns the caret when multiple source positions share the same display coordinate.

Pointer hit testing chooses a candidate from the clicked glyph side; Left/Right preserves directional intent while traversing a revealed delimiter; text input uses the selected canonical source offset. Moving to an unambiguous position resets affinity. Source selections remain normalized byte ranges; affinity disambiguates only the active collapsed endpoint and never changes persisted text.

The current nearest-boundary formulas duplicated between `VisualProjection` and `VisualEditableText` will be consolidated behind one mapping API. Keeping the duplicated formulas was rejected because future marked-range and navigation behavior could drift between pure-model tests and GPUI hit testing.

### 3. Register ephemeral visual navigation geometry per rendered row

Each painted `VisualEditableText` records a compact navigation snapshot keyed by document version, visual block index, and the current projection inputs. It contains wrapped-line geometry plus UTF-8-safe display/source mapping sufficient to locate the closest caret for a pixel point. This snapshot is interaction/layout state on `EditorTab`, not `MarkdownDocument` derived state.

In Visual Edit, Up/Down uses the active painted line and a per-tab `preferred_x`. Movement within a block selects the closest caret on the adjacent wrapped line; movement past the first or last line asks the neighboring visual block for the closest caret. If virtualization has not mounted that target, navigation records a one-shot pending target, reveals the row, and completes after layout. Home/End targets the painted line bounds in rendered rows and retains source-line behavior inside explicit source islands. Shift variants reuse the same target computation and extend the canonical source selection.

Using logical Markdown lines in Visual Edit was rejected because it makes visual wrapping non-navigable. Keeping every row mounted was rejected because the existing list virtualization and surface-level input bridge make that unnecessary.

### 4. Make the marked range a first-class projection input

`build_visual_projection` will accept the active marked range in addition to selection and cursor state. Any exact syntax group touched by composition is revealed so the marked source has an identity-mapped display range. `VisualEditableText` paints the marked range with the platform composition underline while retaining ordinary selection and inline styles.

The active projected layout supplies `bounds_for_range` for the requested marked/caret range. The existing Visual Edit surface rectangle remains a temporary fallback before the owning row is laid out. The single surface-level input handler remains unchanged; per-row input handler registration is still prohibited.

### 5. Add semantic undo groups without replacing the compact diff history

Introduce capture metadata that distinguishes contiguous typing, IME composition, and atomic commands. Compatible typing coalesces only when it is temporally adjacent, starts from the previous collapsed caret, and has no intervening selection replacement or command boundary. Paste, formatting, Enter/Backspace structural transitions, table commands, mode/tab changes, and explicit undo/redo terminate a typing group.

The first update of an IME composition captures its pre-composition snapshot. Intermediate marked-text replacements update the same pending group; commit or unmark closes it. One Undo restores the state before composition, and Redo reapplies the committed result. Existing full-plus-diff stack compaction remains in place after group boundaries are chosen.

Operation-based undo was rejected because the current source snapshot/diff design already preserves exact source and selection state with bounded memory; only grouping semantics are missing.

### 6. Test pure interaction rules and the rendered platform boundary

Pure tests cover ambiguous boundary candidates, affinity transitions, UTF-8/grapheme safety, wrapped-line target selection, preferred-x retention, marked-range projection, and history grouping. GPUI tests exercise pointer placement, arrow navigation across hidden markers and blocks, CJK/emoji composition, candidate geometry, one-step undo/redo, and cache/version stability.

## Risks / Trade-offs

- [Risk] Affinity and source offset disagree after text mutation or projection fallback. -> Clear or revalidate affinity whenever the document version changes, and fall back to the exact canonical source offset when no matching boundary exists.
- [Risk] Marker reveal changes wrapping during vertical navigation. -> Compute the target from the newly painted projection and use the existing one-shot reveal mechanism rather than retaining stale line geometry.
- [Risk] An off-screen target needs an additional frame before movement completes. -> Store one bounded pending navigation request keyed by document version and cancel it on any newer input or mutation.
- [Risk] Undo coalescing joins edits users expect to undo separately. -> Require caret continuity, a bounded time window, compatible capture kind, and no intervening command, selection, mode, or tab boundary.
- [Risk] IME implementations signal commit/unmark differently across platforms. -> Drive the state machine from marked-range transitions plus ordinary replacement fallback, and cover Windows-oriented UTF-16 cases together with platform-neutral GPUI tests.
- [Risk] Recording layout snapshots during paint causes notification loops. -> Update ephemeral geometry without mutating the document or unconditionally notifying; notify only when a pending navigation can now complete.

## Migration Plan

1. Add affinity and consolidated mapping types with pure tests while preserving current default mapping behavior.
2. Pass marked ranges through projection and paint composition feedback, then add composition-level undo capture.
3. Register visual navigation geometry and switch Visual Edit movement actions to the layout-aware path behind mode checks.
4. Add compatible typing coalescing and explicit command boundaries.
5. Add rendered GPUI regressions and run the root/workspace suites plus OpenSpec validation.

No persisted migration is required. Rollback removes the new per-tab interaction fields and restores source-line navigation and per-replacement undo capture; Markdown files and existing history entries remain valid.

## Open Questions

No blocking questions. The exact coalescing duration remains an implementation constant and should be chosen alongside interaction tests rather than exposed as a preference in this change.
