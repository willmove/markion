## Why

Users report that Markion's resident memory with several open tabs exceeds Obsidian's in the same scenario, which is the opposite of what a native Rust application should achieve. A code audit identified several unbounded or never-evicted retention sites, but their relative weight is unknown and depends entirely on document content: image-heavy documents would be dominated by GPUI's global image asset table, while long plain-text documents would be dominated by per-tab derived caches and retained shaped text. Optimizing before measuring risks spending effort on the wrong layer, so this change delivers the measurement instrument first.

## What Changes

- Add an in-process memory accounting facility that reports, on demand, the retained size of every known allocation site: per-tab document text and derived caches, per-tab editor layout state, undo/redo history, and the process-global diagram, math, and highlight caches.
- Add a developer-facing way to obtain that report from a running application (a diagnostic action that writes a structured report to the existing tracing log), so a user experiencing high memory can produce evidence without a debugger or a special build.
- Add a headless reproduction harness that builds a configurable set of open tabs from fixture documents and emits the same report, so a given document profile (plain text, images, diagrams, math, code) can be attributed to a specific retention site deterministically.
- Record the audit's retention-site inventory as a maintained artifact so subsequent optimization changes have a shared baseline to compare against.

Non-goals: this change does not evict, cap, downsample, or otherwise reduce any cache. It only measures. Every optimization identified by the audit is deferred to follow-up changes that this change's numbers will prioritize.

## Capabilities

### New Capabilities
- `memory-diagnostics`: on-demand accounting of Markion's retained memory by allocation site, the diagnostic report surface that exposes it, and the headless harness that reproduces a tab profile for attribution.

### Modified Capabilities
- `engineering-quality`: the deterministic-evidence requirement currently covers incremental parsing work counters only; it needs to state that memory accounting is likewise gated on deterministic, machine-independent assertions (relative attribution and monotonic release) rather than absolute byte thresholds, which vary by platform and allocator.

## Impact

- New module for the accounting facility, plus report-producing methods on the tab collection and on each global cache (`DiagramCache`, `MathCache`, the highlight cache) in `src/app/`.
- `MarkdownDocument` in `src/lib.rs` gains a read-only accounting accessor over its derived caches; the cached-per-version invariant is untouched because accounting never forces a cache to populate — an empty cache must report zero rather than deriving one.
- Reuses the existing `tracing` logging setup in `src/storage/logging.rs` for report output; no new dependency.
- The diagnostic action is developer-facing and must not add user-visible chrome or new translated strings beyond what an existing i18n-backed status message can carry.
