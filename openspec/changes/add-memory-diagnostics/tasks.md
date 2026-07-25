## 1. Accounting foundation

- [x] 1.1 Add a memory-accounting module in `src/app/` defining the report types: a named site, its estimated bytes, the counts the estimate derives from, and a marker for externally owned sites that cannot be enumerated.
- [x] 1.2 Define the trait each retention site implements to report its own retained size, plus the aggregation that groups sites into per-tab and process-global totals and carries an explicit unaccounted remainder.
- [x] 1.3 Add unit tests for the report types alone: an empty report totals zero, an externally owned site contributes no bytes to the total but still appears in the report, and a site marked shared is excluded from the total.

## 2. Per-document accounting

- [x] 2.1 Add a read-only accounting accessor on `MarkdownDocument` in `src/lib.rs` that reports text bytes and each derived cache separately, reading the existing `RefCell` storage directly without calling the deriving accessors.
- [x] 2.2 Include the source-mapped cache in the accounting: its retained source handle and its per-region block and heading storage, which are additive on top of the top-level preview-block cache.
- [x] 2.3 Add document tests proving accounting is observational: a freshly opened document reports zero for every derived cache, the caches are still unpopulated afterwards, and the document version is unchanged.
- [x] 2.4 Add a document test that populates every derived cache and asserts each corresponding site reports a non-zero figure, so a future cache field added without accounting is caught.

## 3. Per-tab accounting

- [x] 3.1 Add accounting to `EditorTab` in `src/app/state.rs` covering its document, undo and redo history, and the per-tab layout caches (display text, line offsets, measured height).
- [x] 3.2 Account for retained shaped editor lines as its own named site, reporting the retained line count alongside the byte estimate derived from the known per-line structural cost.
- [x] 3.3 Account for the tab's derived-block handles as shared references to the document's caches rather than as independent allocations, so a rendered document is not counted twice.
- [x] 3.4 Add tests: a tab that has only ever rendered in Visual Edit reports zero retained shaped lines, and a tab holding the same blocks as its document contributes them to the total exactly once.

## 4. Global cache accounting

- [x] 4.1 Add accounting to `DiagramCache` in `src/app/diagram.rs` reporting entry count, pending count, and total raster bytes derived from each ready entry's pixel dimensions.
- [x] 4.2 Add accounting to `MathCache` in `src/app/math_render.rs` reporting entry count and completed bytes, reusing the byte total it already tracks.
- [x] 4.3 Add accounting to the highlight cache in `src/app/application.rs` reporting entry count and the bytes held by its keys and its span values.
- [x] 4.4 Add the GPUI image asset table as a named, externally owned site: report the distinct image references reachable from open documents and their decoded dimensions where known, with no fabricated byte figure.
- [x] 4.5 Add tests that each global cache reports zero when empty and grows by a predictable entry count as entries are inserted.

## 5. Application report surface

- [x] 5.1 Add the app-level report assembly that walks all tabs and all global caches and produces the complete report.
- [x] 5.2 Add a developer-facing action that formats the report and writes it to the diagnostic log through the existing `tracing` setup, keeping the report body out of `src/i18n.rs`.
- [x] 5.3 Wire the action's confirmation to an existing localized status message so the surface adds no new translated strings.
- [x] 5.4 Add a GPUI test that invoking the action produces a report containing every expected site name and leaves document versions, selection, and scroll state unchanged.

## 6. Headless attribution harness

- [x] 6.1 Add fixture documents in `examples/` covering the profiles to be attributed: plain text of several thousand lines, embedded images, diagrams, math, and code blocks.
- [x] 6.2 Add the harness entry point in the root crate's test surface that opens a configurable number of tabs of a chosen profile and emits the same report.
- [x] 6.3 Add a harness test asserting that opening an additional tab of the same document increases the per-tab total and leaves the global-cache total unchanged.
- [x] 6.4 Add a harness test asserting that closing a tab returns the per-tab total to its value before that tab was opened.
- [x] 6.5 Add a harness test asserting that two consecutive reports with no intervening activity are identical.

## 7. Baseline and documentation

- [x] 7.1 Record the retention-site inventory and the audit's reasoning as a document under `docs/`, so follow-up optimization changes have a shared baseline to quote before and after numbers against.
- [x] 7.2 Run the harness across every fixture profile and record the resulting attribution table in that document as informational diagnostics, explicitly noting the platform it was measured on.
- [x] 7.3 Note in that document which sites the attribution leaves unexplained relative to the process's resident size, since a large remainder points at the externally owned image asset table.
- [x] 7.4 Run `cargo fmt --check` and `cargo test --workspace`, then `openspec validate add-memory-diagnostics` before archiving.
