## Context

`MarkdownDocument` currently clears `cached_preview_blocks` and `cached_visual_blocks` on every mutation. The next Visual Edit render synchronously performs a full pulldown-cmark parse for preview/outline and another source-ranged derivation for all visual blocks. `ListState` then compares entire value objects, so a small edit near the top makes shifted suffix ranges appear changed even when their rendered content and row height are identical.

The absorbed `crates/markdown/src/incremental.rs` cannot be used as the Visual Edit model: its AST has no complete source spans, independent regions are cloned into an assembled document, and every assembly renumbers `NodeId`s. Those properties conflict with exact source mutation and stable visual row identity.

The intended flow is:

```text
canonical source replacement + previous region cache
  -> UTF-8-safe SourceEdit
  -> conservative region boundary analysis
  -> reuse unchanged region derivations / parse dirty regions
  -> assemble exact PreviewBlock + outline ranges
  -> derive/reconcile VisualBlock with stable document-local ids
  -> version-cached Arc<Vec<_>> + identity-aware ListState splice
  -> existing ephemeral projection, layout, navigation and IME state
```

## Goals / Non-Goals

**Goals:**

- Bound ordinary localized Visual Edit derivation to the affected top-level region plus small boundary context.
- Preserve byte-exact source ranges and full-parse semantic equivalence after every accepted incremental edit.
- Preserve stable identities for unchanged visual blocks even when a preceding edit shifts their source ranges.
- Retain the current document version, cache sharing, undo snapshot, dirty/autosave, virtualization, and stale-result rejection invariants.
- Resolve and remove the unused spanless Typune incremental parser inventory.

**Non-Goals:**

- Rope/piece-table storage, mutable rich-text trees, background parsing redesign, direct complex-block widgets, or parser changes outside the current CommonMark/GFM feature set.
- Incremental stats/highlighting beyond preserving their existing versioned/memoized behavior.
- Eliminating full parse fallback for globally scoped or ambiguous Markdown.

## Decisions

### 1. Capture source edits at `MarkdownDocument` mutation boundaries

Introduce a pure `SourceEdit` containing the old UTF-8 byte range, inserted byte length, and old/new document versions. `insert`, `replace_range`, and editing helpers feed the same internal replacement path. Whole-document `set_text`, open/recovery replacement, and undo snapshot restoration mark the next derivation as full.

The document retains the previous preview/visual cache together with a bounded pending edit chain instead of deleting it immediately. A chain is incremental only while edits are ordered against consecutive versions and remain below conservative count/size limits; otherwise it collapses to a full-parse marker.

Inferring edits by diffing old/new whole strings was rejected because mutation call sites already know exact ranges and diffing adds O(document) work on the typing path.

### 2. Use a Markion-owned source-ranged region cache

Add a GUI-independent root module that splits Markdown into conservative top-level regions at blank-line boundaries while keeping fenced blocks, indented continuations, lists, blockquotes, tables, HTML blocks, and front matter intact. Each cached region stores its exact text, source start, preview blocks with local ranges, local outline entries, and a stable region generation.

For a localized edit, re-split only the old affected region plus its immediate neighbors, reuse text-identical prefix/suffix regions, parse dirty regions with the existing pulldown-cmark configuration, and shift local ranges during assembly. Link reference definitions, footnote definitions/references, front-matter boundary edits, ambiguous HTML, unclosed fences, or failed parity guards force the current full derivation.

Adopting `typune_markdown::IncrementalParser` was rejected because converting its spanless AST back to exact authored ranges would require reparsing and its renumbered nodes cannot supply stable identities. The existing module will be removed after the Markion path has differential coverage.

### 3. Make full parse equivalence the correctness oracle

Incremental output must equal `derive_preview_and_outline` for block variants, rendered content, ordering, and every source range. Pure tests apply deterministic and randomized UTF-8 edit sequences to both paths after every edit. Debug/test builds expose derivation counters and can force a full comparison; runtime release behavior uses the conservative boundary rules and falls back before publishing uncertain output.

An optimistic parser that publishes locally plausible output was rejected because a temporarily wrong source range could direct a visual edit at unrelated Markdown.

### 4. Add stable visual block identity separate from source range

Add an opaque `VisualBlockId` to `VisualBlock`. Initial derivation allocates ids monotonically within the process. Reconciliation maps old source ranges through the pending edit chain and reuses an id only when the old and new block kind, source slice, and source-relative editable/reveal structure match. Dirty, split, merged, or ambiguous blocks receive new ids.

Identity is not serialized and is not a document version. Source offsets remain authoritative for mutations; ids exist only to associate ephemeral row state and list height caches across versions.

Content hashes alone were rejected because equal repeated paragraphs need occurrence-aware matching and hashes cannot prove source lineage.

### 5. Reconcile visual list rows by identity

`sync_visual_list` computes its common prefix/suffix using `VisualBlockId`, then splices the changed middle. Fresh block values remain available to row builders, so shifted ranges update without invalidating unchanged row heights. Navigation snapshots remain keyed by document version and block id/index; stale version geometry is discarded exactly as in the interaction-core change.

Preview/Read continues using its existing debounced full-document path in this change. The new region cache feeds synchronous Visual Edit derivation first; expanding it to asynchronous preview is a later decision.

## Risks / Trade-offs

- **[Region boundary misses a cross-block dependency]** -> Treat global definitions, ambiguous HTML/front matter, open fences, and uncertain continuation boundaries as full-parse fallbacks; lock parity with edit-sequence differential tests.
- **[Range shifting corrupts nested inline/math metadata]** -> Implement one exhaustive offset visitor per source-ranged domain type and test every variant with UTF-8 insert/delete before and after the block.
- **[Stable ids attach old row state to different repeated content]** -> Reuse only through edit-range lineage plus structural equality, never by global text search alone.
- **[Cache retention increases memory]** -> Keep only the immediately previous derivation and a bounded edit chain; undo snapshots continue to clone no derived caches.
- **[Small documents get extra bookkeeping without benefit]** -> Use a size/region threshold that may choose the existing full path, and benchmark both paths before enabling incremental work below the threshold.
- **[Removing Typune inventory loses potentially useful code]** -> Preserve the source-ranged strategy and tests in Markion first; git history retains the old implementation.

## Migration Plan

1. Add pure edit/offset and region-cache types behind the existing `MarkdownDocument` API.
2. Differential-test region assembly before enabling it in `visual_blocks_shared`.
3. Add `VisualBlockId`, reconciliation, and identity-aware list splicing with cache/scroll regressions.
4. Enable conservative incremental derivation and fallback counters.
5. Remove `crates/markdown/src/incremental.rs`, its exports, and direct tests after workspace parity passes.
6. Roll back by routing all derivation through the preserved full-parse function; no persisted data migration is required.

## Open Questions

- None for implementation. Threshold constants may be tuned from the checked-in large-document benchmark without changing the behavioral contract.
