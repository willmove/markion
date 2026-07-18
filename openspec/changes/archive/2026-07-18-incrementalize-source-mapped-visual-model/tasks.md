## 1. Source Edit and Range Infrastructure

- [x] 1.1 Add UTF-8-safe `SourceEdit` / edit-chain types and route `insert`, `replace_range`, and whole-document replacement through one mutation boundary
- [x] 1.2 Add exhaustive offset-shift visitors for every source-ranged preview, outline, visual-run, reveal-group, prefix, math, and structural-edit field
- [x] 1.3 Add edit-chain tests for insertion, deletion, replacement, UTF-8 boundaries, chained versions, overflow limits, whole-document replacement, undo/redo, and cache-free document clones

## 2. Conservative Source-Ranged Region Cache

- [x] 2.1 Add a Markion-owned region splitter/cache that preserves exact source text and keeps fences, lists, quotes, tables, HTML, front matter, and continuation lines within conservative boundaries
- [x] 2.2 Parse dirty regions with the existing CommonMark/GFM configuration, assemble shifted preview blocks and outline entries, and reuse unchanged region derivations
- [x] 2.3 Add explicit full-parse fallbacks for reference/footnote dependencies, front-matter boundary edits, ambiguous HTML, unclosed fences, invalid edit chains, and parity failures
- [x] 2.4 Wire the region cache into synchronous Visual Edit derivation while preserving existing debounced Split/Read preview behavior and per-version `Arc` cache hits

## 3. Stable Visual Block Identity

- [x] 3.1 Add opaque document-local `VisualBlockId` allocation and include identity in every normal, whitespace, and source-island visual block
- [x] 3.2 Reconcile old blocks through the source edit chain and reuse identities only for occurrence-safe, structurally equal unchanged descendants
- [x] 3.3 Issue new identities for dirty/split/merged/ambiguous blocks and reset identity state on open, recovery, whole-document replacement, and cache-free undo clones
- [x] 3.4 Add identity tests for prefix/suffix edits, byte-range shifts, repeated equal blocks, block splits/merges, nested lists, conservative source islands, and multi-tab isolation

## 4. Virtualized List and Interaction Integration

- [x] 4.1 Reconcile the Visual Edit `ListState` by stable block identity so only affected middle rows are spliced while current block values expose fresh source ranges
- [x] 4.2 Re-key or invalidate navigation snapshots, pending movement, caret affinity, marked geometry, math/diagram presentation, and other row-local state by current version plus stable block identity
- [x] 4.3 Add rendered GPUI regressions for scroll anchoring, caret ownership, wrapped navigation, marker reveal, IME, direct typing, and unchanged-row reuse after edits near the start of a large document

## 5. Differential Correctness and Performance

- [x] 5.1 Add deterministic differential tests covering every preview/visual block variant and UTF-8 edits before, inside, and after each source-mapped construct
- [x] 5.2 Add randomized edit-sequence tests comparing incremental blocks, outline, projections, formatting mutations, and structural edits with full derivation after every edit
- [x] 5.3 Add test-only derivation counters and large-document benchmarks proving localized edits reparse bounded regions while fallback cases remain exact
- [x] 5.4 Verify document versions, dirty/autosave/recovery, semantic undo, highlighting, stats, shared text handles, preview debounce, and cache-free snapshots retain their existing contracts

## 6. Inventory Resolution and Verification

- [x] 6.1 Remove `crates/markdown/src/incremental.rs`, its module/export declarations, and direct incremental-parser tests after the source-ranged Markion path reaches parity
- [x] 6.2 Run `cargo fmt --all -- --check`, focused incremental/Visual Edit tests, `cargo test`, and `cargo test --workspace`
- [x] 6.3 Run `openspec validate incrementalize-source-mapped-visual-model` and resolve all validation errors
