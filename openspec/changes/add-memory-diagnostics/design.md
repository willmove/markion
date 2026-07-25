## Context

An audit of multi-tab memory retention found three distinct layers, each with a different growth law and a different fix, and no way to tell from a running application which one dominates.

```
                     Markion resident memory
  ┌──────────────────────────────────────────────────────────────┐
  │ A  Fixed baseline (independent of tab count)                 │
  │      GPUI/platform renderer, syntect two-face grammar set,   │
  │      DIAGRAM_FONT_DB (load_system_fonts), embedded math fonts│
  ├──────────────────────────────────────────────────────────────┤
  │ B  Process-global render caches (shared, survive tab close)  │
  │      1. GPUI image asset table — no cap, no eviction         │
  │      2. DiagramCache — 128 entries, counted, never weighed   │
  │      3. MathCache — 256 entries / 128 MB budget              │
  │      4. highlight_cache — 128 entries, full clear when full  │
  ├──────────────────────────────────────────────────────────────┤
  │ C  Per-tab state (linear in tab count, never released while  │
  │    a tab stays open, inactive or not)                        │
  │      document text, source-mapped Arc<str> copy,             │
  │      preview blocks, visual blocks, undo history,            │
  │      last_lines (whole-document shaped text)                 │
  └──────────────────────────────────────────────────────────────┘
```

Layer B site 1 lives in GPUI, not in Markion: `img(source)` resolves through `Window::use_asset`, which stores the decoded `Arc<RenderImage>` in `App::loading_assets`, a plain map with no eviction. Markion never calls `App::remove_asset` and installs no `ImageCache`, so every image ever displayed stays decoded at full source resolution for the lifetime of the process. Layer C is Markion's own: `switch_active_tab` clears only transient interaction state, so a tab that has been visited once keeps its derived caches forever.

Which layer dominates depends on document content, and the reporter therefore has to attribute bytes to sites rather than produce a single total.

## Goals / Non-Goals

**Goals:**
- Produce a per-site retained-byte report covering every layer B and layer C site, on demand, from a normally built application.
- Make the report reproducible headlessly for a described document profile, so an optimization change can quote a before/after number.
- Keep accounting free of side effects: reading the report must not populate a cache, bump a document version, or change what any subsequent frame renders.
- Establish deterministic, machine-independent assertions so the accounting can be tested without depending on allocator or platform behavior.

**Non-Goals:**
- Reducing memory. No eviction policy, cap, downsampling, or lazy-loading change belongs to this change.
- Accounting layer A. The fixed baseline is not attributable to Markion data structures and is better measured with an external profiler; the report notes it as unaccounted rather than estimating it.
- Byte-exact accuracy. The report is an attribution instrument, and a site whose estimate is within the right order of magnitude is sufficient to rank the layers.

## Decisions

### A `MemoryFootprint` trait implemented by each retention site, not a central estimator

Each owner reports its own bytes: `EditorTab`, `MarkdownDocument`, `DiagramCache`, `MathCache`, and the highlight cache each implement a method returning their retained size, and the app-level reporter only aggregates and labels.

The alternative — one central function that walks the app and estimates sizes from the outside — was rejected because most of the interesting state is private (`MarkdownDocument`'s caches are private `RefCell` fields; the cache structs' maps are private), so a central estimator would force those fields public purely for measurement and would silently go stale whenever a field is added. Co-locating accounting with ownership means a new cache field is a compile-time-visible omission in the same file.

### Accounting is observational: an unpopulated cache reports zero

`MarkdownDocument`'s derived caches are lazily populated and shared per version via `Arc`. Accounting must read `cached_preview_blocks` and friends through their existing `RefCell` borrow without calling the deriving accessors, so a tab that has never been rendered in a given mode reports zero for that cache rather than deriving one to measure it. This preserves the cached-per-version invariant and, more importantly, makes the report answer the question that matters: how much is actually retained right now.

A consequence worth stating: because preview and visual blocks are held behind `Arc` and a tab also keeps its own `Arc` clone in `preview_list_blocks` / `visual_list_blocks`, the report must count the pointee once and label the tab-level handle as shared, or the total will double-count every rendered document.

### Shaped-line accounting uses the structural constant, not a walk of GPUI internals

`EditorTab::last_lines` holds one `gpui::WrappedLine` per logical line of the whole document. `WrappedLine`'s glyph data sits behind an `Arc<WrappedLineLayout>` whose interior is not publicly enumerable, so the report accounts for it as the count of retained lines multiplied by the known per-line structural cost, and reports the line count alongside the byte estimate so a reader can see the multiplier.

This site deserves its own label rather than being folded into a generic "editor state" bucket: `WrappedLine` embeds `SmallVec<[DecorationRun; 32]>` inline, so each retained line costs roughly three kilobytes independent of how many characters it holds, while Markion only ever supplies one to three runs. A several-thousand-line document therefore retains tens of megabytes per tab that a naive "bytes proportional to text" model would miss entirely.

### The application surface is a logged report, not a UI panel

The diagnostic is emitted through the existing `tracing` setup in `src/storage/logging.rs`, triggered by an action that is reachable but not advertised in the menu chrome. A UI panel would need layout, translation, and theme work for an audience of one, and a log line is what a reporting user can actually attach to an issue.

The action reuses an existing i18n-backed status message to confirm that the report was written; the report body itself is diagnostic English and stays out of `src/i18n.rs`.

### The headless harness lives in the root crate's test surface

Attribution needs a controlled input: N tabs built from fixture documents with a known profile (plain text, images, diagrams, math, code blocks). GPUI's `test-support` feature is already a dev-dependency and is what lets tabs be constructed and rendered without a real window, so the harness is a test-surface entry point in the root crate rather than a new binary or a workspace member. Members under `crates/*` cannot host it because they must not depend on `gpui`.

### Deterministic assertions: relative attribution and monotonic release

Absolute byte thresholds are not testable across platforms and allocators, matching the existing precedent in the `engineering-quality` spec that wall-clock benchmarks are informational and not merge gates. The tests therefore assert relationships that hold regardless of platform:

- A site that holds nothing reports zero.
- Opening an additional tab with the same document increases the per-tab total and leaves the global-cache total unchanged.
- Closing a tab returns the per-tab total to its prior value, which is the assertion that would catch a leak in tab teardown.
- Reading a report twice without intervening edits produces identical numbers, which is the assertion that catches accounting that accidentally populates a cache.

The absolute numbers are diagnostic output, reported and not gated.

## Risks / Trade-offs

**Accounting drifts as new cache fields are added** → Each site's accounting method lives in the same file as the struct it measures, and the change adds a test that fails when a document's derived caches are all populated yet a site reports zero, so an unaccounted new cache is visible rather than silent.

**Estimates are wrong enough to mislead prioritization** → The report prints the underlying counts (tabs, retained lines, cache entries, per-entry pixel dimensions) next to every byte figure, so a suspicious total can be checked by hand against the process's actual resident size before an optimization change is scoped on it.

**The GPUI image asset table is the prime suspect and is the one site Markion cannot enumerate** → Its contents live in `App::loading_assets` behind a private field. The report cannot list it directly, so it accounts for what Markion does know — the image references reachable from open documents and their decoded dimensions where available — and explicitly labels the site as externally owned and un-enumerable. Confirming its true size stays a job for an external profiler, and the report's role is to establish whether the accountable sites explain the observed total or leave a large unexplained remainder that points at it.

**Adding accounting to the typing path costs performance** → Accounting is only invoked by the diagnostic action and the harness, never during render or edit, and the tests assert the report is side-effect free rather than fast.
