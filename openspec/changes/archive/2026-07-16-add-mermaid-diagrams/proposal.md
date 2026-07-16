## Why

Markion currently treats Mermaid fences as ordinary syntax-highlighted code, so diagrams are not visible in preview or HTML exports. Adding Mermaid through a backend-neutral diagram layer provides useful rendering now while giving contributors a stable, GUI-free extension point for future PlantUML, D2, Typst, and other diagram engines.

## What Changes

- Add a GUI-free `markion-diagram` workspace crate that owns diagram source types, render requests/results/errors, a backend trait, and a registry/dispatch mechanism.
- Add Mermaid as the first in-process backend, using `mermaid-rs-renderer` with nonessential CLI/PNG features disabled to produce static SVG without Node.js, Chromium, a WebView, network access, or temporary files.
- Recognize fenced code blocks whose first info-string token is `mermaid` (ASCII case-insensitive) and render them as diagrams in Split Preview and Read mode.
- Render diagrams asynchronously and memoize bounded results by backend, source, and light/dark render context without changing Markion's per-document-version derived Markdown cache rules.
- Show a localized loading state or an actionable localized error with the authored source when rendering cannot complete.
- Keep Mermaid blocks source-backed code islands in Visual Edit mode.
- Inline successfully rendered Mermaid SVG in built-in styled and plain HTML exports, with a source-code fallback when the backend rejects a diagram.
- Add deterministic unit, integration, and cache-invariant coverage for the diagram crate, Mermaid backend, GPUI integration boundary, and HTML export.
- Non-goals: implementing PlantUML, D2, or Typst backends in this change; Mermaid.js-perfect syntax or visual parity; diagram scripts/click handlers; remote rendering services; and rich Mermaid rendering in PDF, DOCX, LaTeX, PNG, or JPEG exports.

## Capabilities

### New Capabilities

- `diagram-rendering`: Defines the extensible diagram backend contract, Mermaid fenced-block recognition, static preview rendering, error behavior, theme-aware memoization, and backend isolation.

### Modified Capabilities

- `crate-architecture`: Adds the GUI-free `markion-diagram` workspace member and its typing-path development optimization requirement.
- `code-and-math`: Distinguishes supported diagram fence identifiers from ordinary syntax-highlighted code blocks while preserving authored source and fallback behavior.
- `export`: Requires built-in styled and plain HTML exports to inline successful Mermaid SVG output and fall back safely on render errors.

## Impact

- New workspace member: `crates/diagram/` with package name `markion-diagram`; it must not depend on `gpui` or another GUI toolkit.
- Root manifest/workspace: add the member dependency, the Mermaid renderer dependency behind the new crate, and an explicit `[profile.dev.package.markion-diagram]` optimization override.
- Root application: diagram scheduling/cache state, preview presentation, GPUI SVG-byte conversion, localization, and tests under `src/app/`.
- Markdown/export paths: fenced-language classification and built-in HTML fragment generation; existing source ranges, preview selection semantics, and Visual Edit source islands remain intact.
- Dependencies: add `mermaid-rs-renderer` with default features disabled; no runtime Node.js, browser, subprocess, network, or temporary-file dependency.
- Invariants: Markdown-derived state remains cached once per document version and shared via `Arc`; diagram work is not repeated per frame or per keystroke; stale background completions cannot overwrite newer visible state.
