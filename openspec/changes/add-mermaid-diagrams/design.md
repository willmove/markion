## Context

`pulldown-cmark` already reports fenced block info strings and Markion preserves the first token as `PreviewBlock::CodeBlock.language`. A `mermaid` fence therefore reaches the preview today, but only the ordinary syntax-highlighting branch. Preview blocks and outline data are parsed in a debounced background task, cached per document version, shared through `Arc`, and reconciled into a virtualized list. Syntax highlighting has a separate cross-edit cache.

Mermaid rendering adds a second kind of derived work with different invalidation rules: diagram output depends on backend, source, and render theme, but not on the containing document version. It can also take longer than Markdown parsing and must never run during each GPUI frame. HTML export needs the same backend output without depending on GPUI.

The repository is already a root-package Cargo workspace. New member crates must remain GUI-free, and compute-heavy members on the typing path need an explicit development-profile optimization override.

The intended flow is:

```text
Markdown source + document version
        |
        v
existing background pulldown pass
        |
        +-- ordinary fence ----------> syntax highlight cache
        |
        `-- registered diagram fence -> cache key(backend, source, theme)
                                        | missing
                                        v
                                 background backend render
                                        |
                         passive sanitized SVG / typed error
                                        |
                          root GPUI Image conversion + notify
```

## Goals / Non-Goals

**Goals:**

- Establish a small, stable, GUI-free backend API that future diagram engines can implement without depending on Markion application internals.
- Ship an in-process Mermaid backend and render `mermaid` fences as static SVG in Split Preview and Read mode.
- Keep diagram work asynchronous, bounded, theme-aware, and memoized independently of document-version caches.
- Reuse the same backend contract for built-in HTML export.
- Treat backend output as untrusted until it has passed passive-SVG validation/sanitization.
- Preserve source-backed editing, selection source ranges, and safe source fallback behavior.

**Non-Goals:**

- Dynamic library loading, a stable C ABI, runtime plugin discovery, or downloading backends.
- PlantUML, D2, Typst, or other non-Mermaid implementations in this change.
- Full Mermaid.js compatibility or browser-only Mermaid features.
- JavaScript, click handlers, hyperlinks, animation, remote resources, or interactive diagram events.
- Rich diagram rendering in PDF, DOCX, LaTeX, PNG, or JPEG exports.
- Mapping every custom Markion palette color into Mermaid; v1 uses light/dark diagram themes.

## Decisions

### Put the backend contract in a GUI-free `markion-diagram` crate

Create `crates/diagram` with Cargo package name `markion-diagram`. The crate defines pure, owned data and a `DiagramBackend: Send + Sync` trait. The contract includes:

- a stable backend identifier and one or more normalized fence aliases;
- a `DiagramRenderRequest` carrying authored source, `DiagramTheme` (`Light` or `Dark`), and resource limits;
- a passive SVG `DiagramRender` result with bytes and optional intrinsic dimensions;
- a typed `DiagramError` category (`UnsupportedBackend`, `InputTooLarge`, `InvalidSource`, `UnsafeOutput`, or `RenderFailed`) plus non-localized diagnostic detail;
- a `DiagramRegistry` that rejects duplicate identifiers/aliases, resolves the first info-string token case-insensitively, and dispatches render requests.

The Mermaid adapter lives behind a default-disabled `mermaid` Cargo feature in the same crate. The root application enables that feature and explicitly registers the built-in backend. A future in-tree module or external crate can depend on `markion-diagram`, implement the trait for its own local type, and provide one constructor for compile-time registration. Registration remains explicit; the trait does not promise runtime binary compatibility.

Alternative considered: implement Mermaid directly in `src/app/preview.rs`. That is smaller initially but couples parsing, backend selection, and rendering to GPUI and gives future contributors no reusable boundary.

Alternative considered: create separate `markion-diagram-core` and `markion-diagram-mermaid` crates immediately. This provides stronger package separation but is unnecessary for one backend; an optional feature keeps the core API usable without Mermaid while avoiding extra workspace and release plumbing.

### Use `mermaid-rs-renderer` for an in-process static Mermaid backend

Add `mermaid-rs-renderer` with default features disabled so Markion uses its Rust SVG API without its CLI and PNG/resvg stack. The adapter maps light requests to the renderer's standard light theme and dark requests to its dark theme, and maps structured parser errors into `DiagramError::InvalidSource`.

This backend avoids Node.js, Chromium, subprocesses, network access, and temporary files. Its supported subset, not Mermaid.js as a whole, defines Markion v1 compatibility. Backend conformance tests pin representative supported diagram families and error behavior so dependency upgrades cannot silently change the contract.

Alternative considered: bundle Mermaid.js in a WebView or JavaScript runtime. Markion has no browser surface, and Mermaid relies on DOM/layout behavior; adding one would materially increase packaging, security, and platform complexity.

Alternative considered: shell out to official `mmdc`. It gives stronger compatibility but would make preview depend on Node/Chromium installation and introduce slow process startup and temporary-file lifecycle concerns.

### Sanitize every backend result before exposing it to consumers

The registry, rather than each caller, applies `svg-hush` to backend SVG before producing `DiagramRender`. It rejects malformed SVG, active content, external resources, unsupported output types, oversized output, and sanitizer failures. Both preview and HTML export consume the same sanitized bytes. GPUI additionally rasterizes SVG through its existing static image path, so preview never executes diagram-authored scripts or events.

Initial limits are constants owned by `markion-diagram`: 64 KiB maximum authored source and 4 MiB maximum sanitized SVG. They are included in the request/config API so later application preferences can adjust them without changing the backend trait. Tests cover exact boundaries.

Alternative considered: trust the built-in Mermaid adapter. That would make the safety contract backend-specific and would allow a future contributor's backend to accidentally inject active SVG into exported HTML.

### Preserve `PreviewBlock::CodeBlock` and dispatch at the presentation boundary

Do not add a Mermaid-specific Markdown AST or preview-block variant. A fence remains authored code with its existing source range, plain-text selection behavior, and Visual Edit code source island. A shared helper extracts and normalizes the first info-string token, asks the registry whether it is a diagram alias, and routes recognized blocks to diagram presentation before ordinary highlighting. Unknown aliases and unsupported languages continue down the existing code-block path.

This keeps Markdown parsing backend-neutral and means adding a future alias does not require another exhaustive `PreviewBlock` variant. The `code_line_numbers` preference does not affect rendered diagrams; it applies only when a block is presented as source fallback.

Alternative considered: introduce `PreviewBlock::Mermaid`. That makes the first backend explicit but bakes one diagram language into the core model and forces unrelated selection, Visual Edit, export, and splice matches to change for every future backend.

### Keep diagram rendering in an application-level asynchronous cache

Add a bounded root-application cache keyed by `(backend_id, source, DiagramTheme)`. Entries are `Pending`, `Ready` (sanitized core result plus `Arc<gpui::Image>`), or `Error`. After preview blocks are synchronized, the application scans the current block slice for recognized diagram fences, inserts missing entries as `Pending`, and launches one background render per missing key. Landing results update by cache key and notify GPUI.

The cache is shared across tabs and document versions because identical diagram source and theme produce identical static output. A completed task for text that is no longer visible cannot overwrite document-derived state; it only populates its immutable content key and can be reused if that key appears again. Pending entries are never duplicated. The cache uses a deterministic bounded eviction policy for completed entries and never evicts in-flight work.

Theme changes request the other light/dark key without invalidating or reparsing the document. Until a result lands, the preview displays a localized loading placeholder. Errors display a localized category/detail and the authored source in a code-style fallback. No render is initiated in `preview_block_view` itself.

Alternative considered: store SVG in `PreviewBlock`. That would couple theme changes to document-version caches, enlarge every cloned preview block, and either block the first parse or mix backend work into Markdown derivation.

Alternative considered: render synchronously on a cache miss during GPUI rendering. Even a memoized first miss can stall a frame, and a document with several diagrams compounds the delay.

### Inline sanitized SVG in built-in HTML exports after Markdown transformations

Styled and plain HTML exports use the same registry and render context. The HTML event path identifies registered diagram fences and records opaque placeholders while `pulldown-cmark`, extended-inline transforms, and math annotation run normally. After those transforms, placeholders are replaced with sanitized inline SVG wrapped in a stable diagram container. This prevents Markdown text-node extensions from rewriting labels inside generated SVG.

If a diagram is invalid or exceeds limits, export succeeds with an escaped `<pre><code class="language-mermaid">` fallback containing the exact authored diagram source. No scripts or external Mermaid runtime are added. Other export formats preserve their existing code-block behavior.

Alternative considered: export `<pre class="mermaid">` plus Mermaid.js. That makes exported documents network/runtime dependent and introduces executable content. Alternative considered: data-URI SVG images. Inline SVG is easier to size responsively and avoids base64 growth while remaining static after sanitization.

## Risks / Trade-offs

- [The Rust renderer accepts only a subset of Mermaid.js or changes output across releases] -> Document subset compatibility, pin representative conformance fixtures, and review dependency upgrades deliberately.
- [SVG sanitization removes styling required by the renderer] -> Add a task-level spike using representative light/dark diagrams and require sanitized results to render through GPUI before completing integration.
- [A large or adversarial diagram consumes CPU before output limits apply] -> Reject source above the registry limit, render only on background executors, deduplicate pending keys, and keep the completed cache bounded. Native backend cancellation is not promised in v1.
- [Theme switching briefly shows a placeholder] -> Cache light and dark results independently and retain completed results; only the first use of a theme renders.
- [Trait design is prematurely broad] -> Standardize only static SVG, identifiers, aliases, themes, limits, and typed errors. Dynamic plugins, backend-specific configuration, and multiple output media types remain out of scope.
- [HTML export output becomes active content] -> Sanitize centrally, reject external references/active features, and test exported fragments for script/event/foreign-resource removal.
- [Concurrent work overlaps `fix-html-preview-regressions` in `src/lib.rs`] -> Land or rebase that change before editing the shared HTML/preview transformation areas; keep Mermaid dispatch separate from raw-HTML preview classification.

## Migration Plan

1. Add and test the GUI-free crate, trait, registry, limits, sanitizer, and optional Mermaid adapter without changing application behavior.
2. Wire the root dependency, explicit development optimization, and compile-time backend registration.
3. Add asynchronous preview cache/scheduling and GPUI SVG conversion behind recognized fence dispatch.
4. Add localized loading/error UI and verify source selection and Visual Edit source islands remain unchanged.
5. Add static HTML export substitution and fallback behavior.
6. Run targeted crate/root tests, the full workspace suite, formatting, clippy, and OpenSpec validation.

Rollback removes the root registration/dependency and leaves `mermaid` fences on the ordinary code-block path; Markdown files require no migration because the authored syntax is unchanged.

## Open Questions

No blocking questions remain for v1. Future changes can decide whether to expose backend enablement/preferences, map complete custom palettes, split individual backends into separate packages, or add raster/vector bridges for non-HTML exports.
