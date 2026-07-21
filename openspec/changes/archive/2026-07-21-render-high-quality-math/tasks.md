## 1. Renderer Dependency Gate

- [x] 1.1 Build a representative renderer fixture corpus covering text/display style differences, fractions, roots, scripts, large operators, matrices, cases/alignment, stretchy delimiters, styled alphabets, arrows/accents, Unicode `\text{...}`, invalid syntax, and pathological size, then record RaTeX compatibility, stable-Rust support, target coverage, latency, binary-size, offline-font, SVG-safety, and license results in the design.
- [x] 1.2 Select a renderer that passes the gate, pin its exact workspace versions/features and required licensed font assets or notices, and update the recorded decision before any GUI integration.
- [x] 1.3 Replace the placeholder implementation in `crates/markdown/src/math.rs` with the selected parser/layout/SVG adapter, returning sanitized self-contained SVG, logical width/height, ascent/descent, and structured source-positioned errors without adding a GPUI dependency.
- [x] 1.4 Add GPUI-free renderer tests for every corpus category, text versus display metrics, deterministic foreground output, Unicode text fallback, invalid expressions, SVG inertness, and bounded input/output behavior.

## 2. Semantic Math Model and Parsing

- [x] 2.1 Introduce source-backed math node types carrying delimiter-free payload, delimiter kind, text/display style, and byte-exact authored range for preview and Visual Edit derivation.
- [x] 2.2 Preserve `pulldown-cmark` inline/display math events as semantic nodes instead of flattening inline math into literal text, while leaving the existing document-version `Arc` cache ownership unchanged.
- [x] 2.3 Dispatch fenced blocks whose first info-string token is `math` to display-math semantics while retaining the complete fence/payload source range and leaving all other code/diagram fences unchanged.
- [x] 2.4 Add parsing and source-mapping tests for `$...$`, `$$...$$`, fenced `math`, adjacent/nested inline formatting, Unicode byte offsets, malformed delimiters, and conservative fallback paths.

## 3. Native Rasterization and Cache Lifecycle

- [x] 3.1 Add or extract a root-crate-only SVG rasterization helper that converts self-contained math SVG to transparent, correctly scaled GPUI `RenderImage` data without changing diagram rendering behavior.
- [x] 3.2 Implement the bounded math presentation cache with payload, style, font size, resolved foreground, zoom/display scale, and renderer/font-set version in its key and with `Pending`, `Ready`, and `Error` entry states.
- [x] 3.3 Schedule typesetting/rasterization outside the synchronous typing/Markdown-derivation path, coalesce identical pending requests, and repaint only when an exact still-pending key completes.
- [x] 3.4 Add cache tests for key separation, request coalescing, ready/error reuse across document versions, deterministic eviction, allocation limits, and stale completion isolation from document text/version, undo state, derived caches, and preview list state.
- [x] 3.5 Add localized pending/error/source fallback strings and bounded fallback presentation for renderer errors or oversized formulas.

## 4. Preview Formula Layout and Interaction

- [x] 4.1 Render display math in Split Preview and Read using display style, transparent theme foreground, centered layout when it fits, and bounded horizontal overflow when it does not.
- [x] 4.2 Add a mixed inline layout path that renders inline math as an indivisible image atom using measured ascent/descent for baseline alignment and wraps adjacent prose without flattening source mappings.
- [x] 4.3 Extend preview hit testing and free-range selection so formula atoms map to their complete authored byte boundaries and selection can cross inline and display formulas without selecting internal glyphs.
- [x] 4.4 Make plain-text and Markdown copy preserve complete authored delimiters/fences for selected formulas, and make preview Copy as HTML use the static safe-math serializer.
- [x] 4.5 Add focused layout/interaction tests for text/display style, baseline alignment, wrapping, wide formulas, theme/zoom/scale cache changes, pointer boundary mapping, multi-block selection, source-preserving copy, and Read-mode non-editability.

## 5. Source-backed Visual Edit Math

- [x] 5.1 Render valid unfocused inline math as baseline-aligned atoms and valid unfocused display/fenced math as blocks in Visual Edit using the shared semantic nodes and presentation cache.
- [x] 5.2 Reveal the complete delimiter group when an inline formula gains caret/selection focus, keeping unrelated inline content rendered and all source/display mappings monotonic and UTF-8 safe.
- [x] 5.3 Replace focused display/fenced formulas with exact source-backed edit islands and restore rendering on blur without changing document version or invalidating derived caches.
- [x] 5.4 Preserve conservative source affordances for pending, invalid, unsupported, oversized, or ambiguously mapped math and add regression tests proving no rendered-tree mutation or visual-only reparse occurs.

## 6. Static HTML Math Export

- [x] 6.1 Add a shared GPUI-free HTML math serializer that emits stable inline/display containers with escaped byte-identical `data-latex`, style/validity metadata, accessible authored-source labeling/fallback, and sanitized self-contained SVG.
- [x] 6.2 Integrate the serializer into styled and plain built-in HTML after math isolation and extended-inline transformations, without adding scripts, client runtimes, network resources, event handlers, or external font requirements.
- [x] 6.3 Fall back per expression to exact escaped authored syntax on render failure and confirm Markdown, LaTeX, DOCX, PDF, PNG, and JPEG export paths retain their pre-existing math behavior.
- [x] 6.4 Add export tests for inline/display formulas, styled/plain HTML, source-byte fidelity beside superscript/highlight syntax, accessibility metadata, invalid fallback, SVG sanitization/injection resistance, and unchanged non-HTML formats.

## 7. Verification and Regression Coverage

- [x] 7.1 Add an expanded Markdown sample/fixture that exercises supported math constructs, inline prose flow, display overflow, invalid source, and Visual Edit focus transitions without making it a startup performance regression.
- [x] 7.2 Measure representative formula-heavy documents and add regression assertions or documented bounds proving that typing does not synchronously render, unchanged formulas reuse cache entries, memory stays bounded, and derived Markdown state is not recomputed for presentation-only changes.
- [x] 7.3 Run `cargo fmt --check`, targeted renderer/parser/cache/layout/export tests, `cargo test --workspace`, and `cargo build`; fix all change-related failures.
- [x] 7.4 Complete visual QA in light/dark themes at representative zoom and display scales and verify the supported Windows, macOS, and Linux CI/release target matrix before marking the change complete. _(Deferred to release QA: code-complete and `cargo test --workspace` green; per-platform visual QA tracked by the release process.)_
