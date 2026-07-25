# Multi-tab memory retention

Informational baseline for Markion's retained-memory attribution. Absolute
byte figures vary by platform and allocator; use the site names and relative
relationships when scoping optimization work. Numbers below were captured with
the headless harness on Windows (`x86_64-pc-windows-msvc`) in a debug test
profile — treat them as order-of-magnitude diagnostics, not merge gates.

## Retention-site inventory

```
                       Markion resident memory
  ┌──────────────────────────────────────────────────────────────┐
  │ A  Fixed baseline (independent of tab count)                 │
  │      GPUI/platform renderer, syntect two-face grammar set,   │
  │      DIAGRAM_FONT_DB (load_system_fonts), embedded math fonts│
  ├──────────────────────────────────────────────────────────────┤
  │ B  Process-global render caches (shared, survive tab close)  │
  │      1. PreviewImageCache — 64 entries / 64 MB, display edge │
  │         2048; claimed per tab, drop_image on release         │
  │      2. DiagramCache — 128 entries / 32 MB completed budget  │
  │      3. MathCache — 256 entries / 128 MB budget              │
  │      4. highlight_cache — 128 entries, full clear when full  │
  ├──────────────────────────────────────────────────────────────┤
  │ C  Per-tab state                                             │
  │      Always: document text, path/dirty, selection, undo/redo │
  │      Active only: preview/visual/outline caches, shaped lines│
  │      Inactive tabs enter dormancy and drop those caches      │
  ├──────────────────────────────────────────────────────────────┤
  │ D  Process footprint (OS counters; not in accounted_total)   │
  │      resident current/peak, commit current/peak              │
  └──────────────────────────────────────────────────────────────┘
```

### Inactive-tab dormancy

When the user activates a different tab (or opens a new one), Markion immediately
dormants the previous tab:

- Clears `MarkdownDocument` derived caches (`preview` / `visual` / `outline` /
  `stats` / `line_count` / `source_mapped`) **without** bumping `text_version`
  or marking dirty.
- Clears shaped-line / layout snapshots and resets preview/visual list mirrors.
- Releases that tab's preview-image claims (same path as tab close).
- Retains text, selection, undo/redo, and scroll handles; reactivation rebuilds
  through the existing lazy accessors.

Harness evidence (`memory_harness_dormancy_drops_and_restores_derived_bytes`):
two `plain_long` Visual Edit tabs — after switching away, the inactive tab's
`document.visual_blocks` / `preview_blocks` / `shaped_lines` sites report 0
bytes while the active tab's `document_text` remains; switching back and warming
repopulates visual blocks without inventing edits.

### How to capture a report

In a running Markion window: **Ctrl+Shift+Alt+M**. The per-site report is
written to the diagnostic log (`tracing` target `markion::memory`) and the
status line shows the existing "Ready" message. The report body is not
localized. The log includes a process-footprint section with OS counters.

Headless: `MarkionApp::load_memory_profile` + `memory_report` in the root
crate's test surface (`src/app/memory.rs`), with fixtures under
`examples/memory_fixtures/`.

### Site names

| Site | Layer | Notes |
|------|-------|-------|
| `tabs[i].document_text` | C | Canonical Markdown source |
| `tabs[i].document.preview_blocks` | C | Lazy; 0 until Split/Read (or harness Preview warmup) |
| `tabs[i].document.visual_blocks` | C | Lazy; 0 until Visual Edit |
| `tabs[i].document.outline` / `stats` / `line_count` | C | Lazy derived |
| `tabs[i].document.source_mapped_cache` | C | Additive on top of preview blocks when Visual Edit ran |
| `tabs[i].undo_stack` / `redo_stack` | C | At most one full text copy on the stack top |
| `tabs[i].shaped_lines` | C | ~3.2 KB × retained `WrappedLine` count (Edit/Split only) |
| `tabs[i].preview_list_blocks` / `visual_list_blocks` | C | Shared handles; contribute 0 to the total |
| `global.preview_image_cache` | B | Owned decoded Markdown preview rasters (`completed_bytes` / `budget_bytes`) |
| `global.diagram_cache` | B | Completed raster bytes + key source strings; exposes `budget_bytes` |
| `global.math_cache` | B | Uses the cache's own `completed_bytes` |
| `global.highlight_cache` | B | Key + span text bytes |

### Process footprint counters

These measure the whole process and **do not** contribute to `accounted_total`.
Each counter is individually optional; unavailable counters are printed as
`unavailable`, never as zero.

| Counter | Meaning |
|---------|---------|
| `resident_current` | Working set / RSS right now (what Task Manager usually shows) |
| `resident_peak` | Highest resident size over the process lifetime |
| `commit_current` | Private commit / phys footprint right now |
| `commit_peak` | Highest private commit over the process lifetime |

Per-platform availability:

| Counter | Windows | Linux | macOS |
|---------|---------|-------|-------|
| resident current | `WorkingSetSize` | `VmRSS` | `resident_size` |
| resident peak | `PeakWorkingSetSize` | `VmHWM` | `resident_size_max` |
| commit current | `PagefileUsage` | unavailable | `phys_footprint` |
| commit peak | `PeakPagefileUsage` | unavailable | unavailable |

Linux leaves commit counters unavailable on purpose: `VmData` is not private
commit and would mislead. Markion never trims the working set or clears peak
watermarks — that would destroy the evidence these counters exist to collect.

### Interpretation rules

Let `A = accounted_total` and `B` ≈ the fixed Layer A baseline. Compare counters
**within one report**, not across machines.

| Observation | Reading |
|-------------|---------|
| `resident_current ≈ A + B` and both fall together | Working as intended |
| `A` falls, resident current does not, peak ≈ current | Allocator retention or GPU atlas growth |
| `A` falls, resident current does not, peak ≫ current | Transient allocation spike raised the watermark |
| `A` does not fall | Markion is still holding state — fix that first |
| commit current falls while resident current does not | OS trimmed the working set; no real private release |

## Harness attribution (informational)

Captured on 2026-07-25 via `memory_harness_attribution_dump` (Windows,
`x86_64-pc-windows-msvc`, debug test profile) after
`report-process-memory-footprint`. One tab per profile; warmup as listed;
`run_until_parked` waits for background decode/raster.

**Peak counters are process-lifetime figures shared across profiles in a single
test process** — do not read them as the cost of an individual profile. Current
counters are meaningful per profile; peaks only rise.

| Profile | Warmup | per-tab | accounted | resident_current | resident_peak | commit_current | commit_peak | Notes |
|---------|--------|--------:|----------:|-----------------:|--------------:|---------------:|------------:|-------|
| `plain_long` | VisualEdit | 3 945 484 | 4 425 633 | 190 877 696 | 193 282 048 | 201 904 128 | 206 995 456 | Text + visual blocks; process baseline dominates |
| `with_images` | Preview | 25 628 | 505 781 | 151 756 800 | 193 282 048 | 153 997 312 | 206 995 456 | Fixture is 1×1 PNG; cache bytes ≈ 4. Peak unchanged from prior profile |
| `with_diagrams` | Preview | 14 973 | 7 667 173 | 164 065 280 | 193 282 048 | 164 405 248 | 206 995 456 | ~7 MiB diagram rasters under 32 MiB budget |
| `with_math` | Preview | 25 414 | 9 515 619 | 165 978 112 | 193 282 048 | 165 974 016 | 206 995 456 | Diagram bytes carried from prior profile |
| `with_code` | Preview | 18 912 | 9 509 767 | 165 990 400 | 193 282 048 | 165 978 112 | 206 995 456 | Highlight cache grows with unique fences |

Budgets / decode policy (constants next to the caches):

- `PREVIEW_IMAGE_CACHE_MAX_BYTES` = 64 MiB; longer edge clamped to 2048 device pixels
- `PREVIEW_IMAGE_DECODE_CONCURRENCY` = 8 (overall in-flight safety cap; favors parallel warm)
- `PREVIEW_IMAGE_HEAVY_DECODE_CONCURRENCY` = 3 (probed oversized local sources only)
- Decode path resizes on `DynamicImage` then `into_rgba8` (no full-res RGBA intermediate)
- `DIAGRAM_CACHE_MAX_BYTES` = 32 MiB (entry capacity remains 128)

Re-run locally:

```text
cargo test --bin markion memory_harness_attribution_dump -- --nocapture
cargo test --bin markion memory_decode_spike_footprint_probe -- --nocapture
```

Or invoke `ReportMemory` (Ctrl+Shift+Alt+M) after opening fixtures by hand and
compare the process-footprint section to `accounted_total`.

## Decode-spike probe (image-heavy hypothesis)

The 1×1 `with_images` fixture cannot stress the preview decode path. A dedicated
probe (`memory_decode_spike_footprint_probe`) allocates then drops four
2048×2048 RGBA buffers (~64 MiB of intermediates) and samples process counters.
Captured 2026-07-25 after `lower-preview-image-decode-peak` (Windows debug test
profile; absolute figures vary by allocator / OS pressure):

| Sample | resident_current | resident_peak | commit_current | commit_peak |
|--------|-----------------:|--------------:|---------------:|------------:|
| before | ~8.9 MB | (lower) | ~1.8 MB | (lower) |
| after drop | ~8.9 MB | ~76 MB | ~1.8 MB | ~69 MB |

Reading: **peak ≫ after ≈ before**. The watermark still records a large
transient allocation; after-drop current returning near baseline shows the
allocator can reuse/release in this synthetic case. Real image docs previously
saw sticky RSS because the old path kept full-resolution RGBA intermediates
(`to_rgba8` then downsample) and stacked unbounded parallel decodes.

Mitigations now in tree (`lower-preview-image-decode-peak`):

1. Resize on `DynamicImage` then consuming `into_rgba8` (no full-res RGBA solely
   for downsampling).
2. Overall in-flight cap 8 + heavy-slot cap 3 for probed oversized local images
   (small images stay parallel under the overall cap).

Remaining follow-up if sticky RSS after close persists in the wild: optional
mimalloc (or similar) for more aggressive page return — separate from decode.

## Unexplained remainder

When process counters are available, use the interpretation table above instead
of treating the gap as a single unexplained blob. The report's
`accounted_total` still never includes:

1. **Layer A** fixed baseline (GPUI, fonts, grammar registry).
2. GPU atlas / allocator overhead after `drop_image` (CPU-side `RenderImage`
   ownership is what Markion site figures report).

After dormancy, multi-tab plain-text RSS should track mostly `document_text` +
undo rather than N× warm visual caches; remaining linear growth is usually
shaped lines on the **active** Edit/Split tab or undo snapshots.

## Follow-up optimization candidates

1. ~~Image asset lifecycle + optional downsampling~~ (done: `PreviewImageCache`).
2. ~~DiagramCache byte budget~~ (done: 32 MiB).
3. ~~Inactive-tab derived-state eviction~~ (done: dormancy on deactivate).
4. ~~Process footprint reporting~~ (done: resident/commit current+peak).
5. ~~Preview-image decode peak~~ (done: resize-before-`into_rgba8`, overall cap 8, heavy cap 3).
6. Lower `MATH_CACHE_MAX_BYTES`.
7. Highlight cache true LRU + hashed keys.
8. Viewport-limited editor `shape_text`.
9. Lazy session restore.
10. Optional mimalloc (or similar) if allocator retention remains the dominant after-close RSS story.
