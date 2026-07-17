## Context

Markion already parses `pulldown-cmark` inline/display math events, but the root preview model flattens inline math into literal text and renders display math through `src/math.rs` as a Unicode approximation plus raw LaTeX. `crates/markdown/src/math.rs` contains a GPUI-free `MathRenderer` API, but its SVG and dimensions are placeholders. Visual Edit currently treats block math as a full source island and does not retain a semantic inline-math atom.

The existing diagram pipeline is the closest architectural precedent: a GPUI-free renderer produces SVG, the root crate rasterizes it with `usvg`/`resvg`, rendering runs off the interaction path, and a bounded cache stores pending, ready, and error entries. Math must additionally participate in baseline-aligned inline flow and retain byte-exact source mappings.

The change touches the per-document derived preview/Visual Edit state, but those `Arc` caches remain keyed only by document version. Rendered formula assets are presentation data held separately and MUST NOT cause Markdown reparsing or document-version changes.

### Data flow

```text
pulldown-cmark math events / fenced `math` blocks
    -> source-backed InlineMath or DisplayMath nodes
       (payload, delimiter kind, byte range, text/display style)
    -> crates/markdown MathRenderer [no GPUI dependency]
       parse -> layout -> self-contained SVG + width/height/ascent/descent
    -> root presentation layer
       async request -> bounded cache -> SVG rasterization -> GPUI RenderImage
    -> mixed inline layout or centered display block

HTML export
    -> same GPUI-free MathRenderer
    -> sanitized self-contained SVG + escaped authored-source fallback/metadata
```

## Goals / Non-Goals

**Goals:**

- Faithfully typeset common KaTeX-compatible and AMS-style inline/display notation offline on all supported desktop platforms.
- Preserve exact authored source, source ranges, selection, copying, and focused editing behavior.
- Share one GPUI-free parsing/typesetting implementation between native preview and built-in HTML export.
- Keep typesetting and rasterization off the keystroke path and reuse unchanged results across document versions.
- Produce deterministic, transparent, theme/zoom/display-scale-correct images with measured baseline metrics.
- Fail locally and safely for an individual expression.

**Non-Goals:**

- A complete TeX engine, arbitrary packages/macros, TikZ, or MathML authoring.
- `\(...\)` and `\[...\]` delimiter aliases in this change.
- Direct mutation of a rendered formula tree in Visual Edit.
- Rich formula upgrades for DOCX, PDF, PNG, or JPEG exports.
- A general rewrite of all diagram/image rendering infrastructure.

## Decisions

### 1. Gate and pin a pure-Rust RaTeX renderer

The first implementation task will validate the current `ratex`/`ratex-svg` release against Markion's representative corpus, stable Rust toolchain, Windows/macOS/Linux targets, offline font behavior, SVG safety, license obligations, binary-size impact, and latency. If it passes, the exact compatible versions and features will be pinned in the workspace and their notices recorded.

RaTeX is preferred because it exposes parser/layout stages, distinguishes text and display math styles, and can produce SVG without a browser, JavaScript runtime, subprocess, or external TeX installation. The adapter in `crates/markdown/src/math.rs` will replace the placeholder implementation and expose only Markion-owned types: math style, SVG, dimensions, ascent/descent, and structured error position/kind.

Alternatives considered:

- MathJax through an embedded JavaScript engine offers broad compatibility but adds a runtime, worker lifecycle, binary cost, and an inline/display adapter that the evaluated wrapper does not currently expose cleanly.
- KaTeX Rust bindings primarily produce HTML/CSS, which is a poor match for GPUI's native image/layout pipeline.
- A subprocess or web view would weaken offline portability and failure isolation.

The dependency spike is a gate, not permission to ship the placeholder. If RaTeX fails a required target or corpus case, implementation pauses at the gate and records a concrete fallback-engine decision before downstream integration.

#### Resolved renderer gate (2026-07-17)

RaTeX 0.1.13 passed the implementation gate and is the selected renderer. Markion pins `ratex-parser`, `ratex-layout`, `ratex-svg`, and `ratex-types` to exactly `0.1.13`; `ratex-svg` enables `embed-fonts`, which includes the KaTeX font set without runtime file lookup.

The checked-in corpus covers inline/display style, fractions, roots, scripts, large operators, matrices, cases/alignment, stretchy delimiters, styled alphabets, arrows/accents, Latin and CJK `\text{...}`, malformed environments/braces, unknown commands, and pathological input bounds. On Windows x86_64, all 13 valid probe expressions produced self-contained path SVG (119,230 aggregate bytes) with no `<text>`, script, remote resource, or event handler; the invalid batch returned failure. A warm complex display probe averaged 26.79 ms over 20 isolated process runs (121.63 ms maximum, including process/font discovery overhead), so application rendering remains asynchronous and coalesced. The embedded font package contains 22 notice/font files totaling 518,172 bytes; the standalone debug probe binary was 8,688,128 bytes, and the final Markion release delta remains a verification item.

The dependency builds with the installed stable Rust 1.97.0 toolchain. Upstream stable CI builds/tests the workspace and standalone embedded-font SVG on Linux, while its release matrix builds embedded-font CLI artifacts for x86_64/aarch64 Linux, macOS, and Windows. Markion's own platform matrix remains authoritative before completion. RaTeX code is MIT-licensed and the embedded KaTeX fonts are SIL OFL 1.1; `THIRD_PARTY_NOTICES.md` records both and is included by the packager.

Self-contained output is enforced rather than assumed: Markion rejects generated SVG that retains `<text>` glyphs or active/external content. Consequently a missing platform Unicode fallback glyph becomes a localized per-expression fallback instead of silently depending on an external font.

### 2. Preserve semantic math nodes and byte-exact source identity

Preview and Visual Edit derivation will retain inline math as an atom rather than appending `$...$` to an ordinary text string. Each math node carries the delimiter kind, delimiter-free payload, exact authored source range, and `Text` or `Display` style. A fenced block whose first info-string token is `math` becomes display math while retaining the fence and payload range for copy/edit fallback.

This separates authored identity from rendering state: renderer results are never serialized back into `MarkdownDocument.text`. The same source metadata drives hit testing, selection, plain-text/Markdown copy, and Visual Edit reveal. Malformed or byte-ambiguous mappings stay source-backed.

### 3. Split GPUI-free typesetting from root-owned rasterization

`crates/markdown` will return self-contained SVG plus logical metrics. It will not depend on `gpui`, satisfying the workspace/orphan-rule invariant. The root app crate will validate/rasterize SVG through the existing `usvg`/`resvg` stack and construct `RenderImage` values.

The diagram implementation may be factored into a small shared root-only SVG rasterizer, but diagram registry/cache semantics remain unchanged. Rasterization uses transparent backgrounds and a scale derived from requested font size, zoom, and display scale; logical dimensions remain stable while the backing raster has enough pixels to avoid blur.

### 4. Use a bounded asynchronous presentation cache

The math cache stores `Pending`, `Ready`, and `Error` states under a key containing:

- delimiter-free LaTeX payload;
- text/display style;
- logical font size;
- resolved foreground color (rather than a coarse theme name);
- effective zoom/display scale;
- renderer/font-set version.

Requests are reserved before background work begins so concurrent views coalesce. Completed entries use bounded least-recently-used or equivalent deterministic eviction; pending entries are not evicted as completed data. A completion applies only if the same key is still pending, then requests a presentation repaint. It does not change document text/version, replace derived Markdown `Arc`s, or reset preview lists. Error entries are cached to avoid retry loops and are naturally bypassed when the source or presentation key changes.

The implemented cache is bounded by both 256 completed/pending keys and 128 MiB
of completed raster backing data. A single raster is limited to 8,192 pixels on
either edge and 32 million pixels; a result larger than the aggregate budget is
converted to a cached local `OutputTooLarge` fallback. Completed entries are
evicted in deterministic completion order, while pending work is never evicted.

A 500-formula document measured on the Windows stable-Rust development profile
derives both semantic preview and Visual Edit state in 1.80 ms. Repeated reads
reuse the exact preview and visual `Arc`s, and a regression test asserts that
presentation completion leaves document version/text, undo depth, list state,
and those derived `Arc`s unchanged. RaTeX and rasterization occur only from the
root presentation scheduler after derivation, never inside the typing/parser
path.

HTML export consumes the GPUI-free SVG directly and does not populate the GUI raster cache.

### 5. Lay out inline math as a measured baseline-aligned atom

Inline formulas use text math style and expose measured ascent/descent. Mixed prose layout wraps text around atomic formula items and aligns each formula baseline with its surrounding run. Selection may cross an atom but never split its internal glyph tree; source mapping resolves to the atom's byte boundaries. A formula wider than the available viewport receives a bounded horizontal overflow affordance instead of being clipped.

Display formulas use display style, occupy their own block, center when they fit, and use horizontal scrolling when necessary. Both forms inherit the current text foreground and render on transparent backgrounds.

### 6. Keep Visual Edit source-backed on focus

Unfocused valid inline/display formulas use the same rendered presentation as preview. When the caret or a selection endpoint enters inline math, Visual Edit reveals the complete delimiter group as one source-backed inline edit range. Focusing display or fenced math replaces that formula's presentation with a source edit island. Moving focus away restores rendering without changing the document version.

Invalid, unsupported, pending, or ambiguously mapped formulas use a localized bounded fallback containing the exact authored source. No operation attempts to infer or rewrite a rendered formula tree.

### 7. Reuse the SVG result for safe static HTML

Styled and plain HTML exporters call the same renderer after Markdown math payloads have been isolated from extended-inline rewriting. Valid formulas are emitted in stable inline/display containers with `data-latex`, validity/style metadata, an accessible authored-source label or fallback, and self-contained sanitized SVG. Generated output contains no scripts, remote resources, event handlers, or external font requirements.

Invalid or unsupported formulas remain escaped authored source in a stable error container and do not fail the document export. SVG insertion occurs after Markdown text transformations, as with static diagrams.

## Risks / Trade-offs

- **[RaTeX is young or misses required syntax/platform behavior]** → Make corpus, target, licensing, and performance verification the first gated task; pin exact versions and stop for an explicit alternative decision if it fails.
- **[Bundled fonts increase binary size]** → Measure the release delta during the spike, include only required licensed assets/features, and prefer path-embedded glyph output for deterministic SVG.
- **[Mixed text/image inline wrapping is more complex than current RichText]** → Introduce a focused semantic inline layout path with baseline/source-mapping tests; leave non-math runs on existing cached paths.
- **[Async completion paints stale content]** → Key completions by all presentation inputs, commit only into a matching pending entry, and keep document-derived state immutable.
- **[Large formulas consume excessive memory or layout space]** → Bound input/output dimensions, cache capacity, raster scale, and fallback error/source panels; provide horizontal overflow rather than unbounded allocation.
- **[SVG becomes an injection surface in HTML]** → Generate SVG internally, sanitize it with an allowlist, escape all authored metadata/fallback text, and prohibit scripts, handlers, links, and external resources.
- **[Theme/scale changes temporarily show pending content]** → Treat them as new cache keys, retain source-sized placeholders to reduce reflow, and repaint when the correct result arrives.

## Migration Plan

1. Complete the renderer spike and record the pinned dependency/features/license outcome.
2. Replace the placeholder GPUI-free math adapter and lock its syntax/metrics/error contract with fixtures.
3. Introduce semantic math nodes without removing the existing source fallback.
4. Add root rasterization/cache and display blocks, then baseline-aligned inline layout and selection mapping.
5. Enable the same presentation in Visual Edit with source reveal/edit islands.
6. Enable sanitized SVG in built-in HTML export.
7. Run focused tests, `cargo test --workspace`, release builds/checks for supported targets where available, and visual QA across themes/scales before removing the valid-expression Unicode preview path.

Rollback is configuration-free: revert consumers to the retained authored-source/Unicode fallback and remove the renderer dependency/cache. Documents require no migration because Markdown remains canonical and no rendered data is persisted.

## Open Questions

- Which exact RaTeX release/features pass the required corpus and target matrix? This is intentionally resolved by the first gated task.
- What cache capacity and maximum formula/raster dimensions meet memory and responsiveness targets on representative documents? Measure before fixing constants.
- Whether Unicode CJK text inside `\text{...}` can use an acceptable deterministic fallback font on every target must be verified by the spike; unsupported glyphs must fail locally rather than disappear.
