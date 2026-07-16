## 1. Diagram Crate Foundation

- [x] 1.1 Create `crates/diagram` as package `markion-diagram`, add the root dependency and optional `mermaid` feature dependencies, and add `[profile.dev.package.markion-diagram]` without introducing any GPUI/GUI dependency into the member crate.
- [x] 1.2 Define owned backend identifiers, normalized aliases, light/dark render context, render request/result types, resource-limit configuration, and typed error categories in `markion-diagram`.
- [x] 1.3 Define the `DiagramBackend: Send + Sync` trait and an explicitly constructed `DiagramRegistry` with duplicate identifier/alias rejection and first-info-token ASCII case-insensitive resolution.
- [x] 1.4 Add headless unit tests for trait dispatch, alias normalization, duplicate registration, unknown aliases, and the public extension path used by a mock contributor backend.

## 2. Passive SVG Safety Boundary

- [x] 2.1 Enforce the 64 KiB source limit before backend invocation and the 4 MiB sanitized-output limit before returning a result, with exact-boundary tests.
- [x] 2.2 Integrate `svg-hush` in the registry result path so every consumer receives only well-formed passive SVG with active content and external references removed or rejected.
- [x] 2.3 Add security regression fixtures covering scripts, event attributes, interactive/external links, foreign or remote resources, malformed XML, unsupported output, and oversized output.

## 3. Mermaid Backend

- [x] 3.1 Implement the optional in-process Mermaid backend with `mermaid-rs-renderer` default features disabled, register the `mermaid` alias, and map Markion light/dark requests to deterministic renderer themes.
- [x] 3.2 Map structured Mermaid parser failures and unexpected renderer failures into stable `DiagramError` categories while retaining diagnostic line/column detail where available.
- [x] 3.3 Add representative conformance tests for flowchart, sequence, class, state, ER, and another supported diagram family, plus invalid-source and light/dark-output cases.
- [x] 3.4 Verify representative Mermaid SVG survives sanitization with intrinsic sizing and can be decoded by the same static SVG stack GPUI uses; resolve sanitizer compatibility issues before application wiring.

## 4. Root Application Integration

- [x] 4.1 Add a root diagram integration module that constructs the built-in registry and classifies `PreviewBlock::CodeBlock` values by their first normalized info-string token without adding format-specific preview-block variants.
- [x] 4.2 Add a bounded application-level diagram cache keyed by backend identifier, exact authored source, and light/dark theme, with distinct pending, ready, and error entries plus deterministic completed-entry eviction.
- [x] 4.3 Scan synchronized preview blocks for missing diagram keys, insert pending entries before spawning work, render on the background executor, land results by immutable cache key, and notify GPUI without modifying document-derived caches.
- [x] 4.4 Render ready sanitized SVG bytes through `gpui::Image::from_bytes(ImageFormat::Svg, ...)`; render localized loading and error/source fallback panels for pending and failed entries; keep ordinary code blocks on the existing syntax-highlight path.
- [x] 4.5 Add all diagram loading/error/fallback strings to `src/i18n.rs` for every supported interface language and avoid exposing raw backend diagnostics without localized context.
- [x] 4.6 Preserve preview source ranges, plain-text/source-copy behavior, code-line-number semantics for fallbacks, and the existing Visual Edit fenced-code source island; add focused regression tests for each boundary.

## 5. Built-in HTML Export

- [x] 5.1 Reconcile the current raw-HTML preview/export changes before modifying shared transformation code, then add registered-diagram collection using opaque placeholders that survive existing extended-inline and math transformations.
- [x] 5.2 Replace successful placeholders after Markdown transformations with sanitized inline SVG in stable diagram containers for styled and plain HTML, without adding scripts, external runtimes, or network references.
- [x] 5.3 Replace failed diagram placeholders with correctly escaped `language-mermaid` code fallback containing the exact authored source while allowing the overall export to succeed.
- [x] 5.4 Add HTML regression tests for styled/plain output, multiple diagrams, invalid source, extended-syntax-like SVG labels, sanitization, source fallback, and unchanged behavior in non-HTML export formats.

## 6. Cache and Runtime Invariants

- [x] 6.1 Add tests proving repeated frames and identical diagrams across tabs share one completed render, identical pending keys launch once, and completed-entry eviction keeps the cache bounded without evicting in-flight work.
- [x] 6.2 Add tests proving light/dark theme switches use independent keys without reparsing Markdown and that switching back reuses the prior result.
- [x] 6.3 Add tests proving edits, tab closure, and late background completion cannot overwrite newer preview blocks, increment document versions, alter dirty/undo state, or invalidate cached outline/stats/text handles.

## 7. Verification

- [ ] 7.1 Run `cargo test -p markion-diagram`, targeted root Mermaid/HTML/cache tests, `cargo test`, and `cargo test --workspace`; resolve regressions within this change's scope.
- [ ] 7.2 Run `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 7.3 Run `cargo tree -p markion-diagram` to verify the GUI-free boundary and disabled Mermaid CLI/PNG features, then run `openspec validate add-mermaid-diagrams`.
