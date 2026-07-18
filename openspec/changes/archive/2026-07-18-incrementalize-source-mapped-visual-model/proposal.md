## Why

Visual Edit now has WYSIWYG-oriented interaction fidelity, but every source mutation still discards all source-mapped visual blocks and synchronously derives them again from a full-document preview parse. That makes typing cost grow with document size and gives visual rows no stable identity across localized edits, which limits both responsiveness and the next phase of direct block editing.

## What Changes

- Record every canonical source replacement as a UTF-8-safe edit descriptor and carry forward the previous source-mapped derivation instead of unconditionally discarding it.
- Add a conservative top-level Markdown region cache that reparses only edit-affected regions when their boundaries are independently parseable and falls back to the existing full parse for global or ambiguous constructs.
- Add stable, document-local visual block identities that survive unchanged prefix/suffix movement across versions while changed, split, merged, or reparsed blocks receive new identities.
- Reconcile the virtualized Visual Edit list by stable block identity so localized edits invalidate only affected row heights and preserve scroll/navigation ownership for unchanged rows.
- Keep preview blocks, outline, stats, highlighting, shared text handles, dirty/autosave, undo/redo, and exact `MarkdownDocument.text` behavior consistent with the current versioned-cache contract.
- Add differential and randomized tests that compare incremental output, source ranges, visual projections, and structural edits against a full parse after every edit, plus large-document performance guards.
- Resolve the retained Typune incremental-parser gate: Markion will not adopt its spanless/re-numbered AST for Visual Edit; any reusable region-boundary logic will be ported into Markion's source-ranged model, and the unused inventory module will be removed once parity coverage is in place.
- **Non-goals:** rope storage, background preview debounce redesign, a mutable rich-text tree, direct table/image/code widgets, multi-cursor editing, or changing persisted Markdown.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Replace the full-reparse-per-edit contract for source-mapped Visual Edit derivation with conservative incremental region reuse, stable block identity, exact full-parse fallback, and version/cache invariants.
- `crate-architecture`: Resolve the absorbed `crates/markdown/src/incremental.rs` inventory gate after choosing Markion's source-ranged incremental model instead of the spanless Typune AST path.

## Impact

- Affects `MarkdownDocument` mutation/cache ownership in `src/lib.rs`, source-ranged block types in `src/model.rs`, region derivation in `src/visual.rs` and parsing helpers, and Visual Edit list reconciliation in `src/app/state.rs` / `src/app/preview.rs`.
- Adds pure incremental-region and stable-identity data structures without adding runtime dependencies or GPUI coupling to workspace members.
- Preserves document-version invalidation for actual text changes while reusing unaffected parsed regions and visual identities inside the new version.
- Removes the unused Typune incremental module and its exports/tests only after equivalent source-ranged behavior is covered in the root Markion model.
