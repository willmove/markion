## Why

When pandoc is installed, DOCX export delegates everything to a pandoc subprocess — but the invocation is bare: only `-V papersize=` is passed. There is no `--reference-doc` (the standard mechanism Typora and Pandoc users rely on to control Word styles), no `--toc`, no `--highlight-style`, no `--resource-path` (so images referenced by relative path silently drop out of the pandoc-produced document), the pandoc binary path is not configurable (`DocxExporter::with_pandoc_path` exists but nothing calls it), and any engine failure collapses silently into the fallback with no indication of why. The engine path also round-trips the Markdown through the internal AST and back before feeding pandoc, which loses information without buying anything (e.g. `==highlight==` depends on pandoc extensions that are never enabled).

## What Changes

- Ship a default reference docx (`assets/templates/reference.docx`) styled for CJK-friendly typography, and pass `--reference-doc` on the engine path; the template path is overridable via `[export] reference_doc` in `config.toml`.
- Pass `--resource-path=<document directory>` so pandoc resolves relative image paths; enable the pandoc Markdown extensions matching Markion's inline syntax (`mark`, `superscript`, `subscript`, etc.) so extended syntax survives the engine path.
- Add a `--toc` capability (off by default, toggled by the options work in `add-docx-export-options`) and a `--highlight-style` for fenced code.
- Make the pandoc binary path configurable via `[export] pandoc_path` (falling back to PATH lookup); wire it from the app layer into `DocxExporter::with_pandoc_path`.
- Surface engine failures: when the engine fails and the fallback is used, the status message names the failure reason (missing binary vs. conversion error) in addition to the existing backend disclosure.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `export`: the pandoc engine path gains requirements for reference-doc styling (bundled default + config override), resource-path image resolution, Markdown extension alignment, configurable pandoc binary path, and engine-failure disclosure.

## Impact

- `crates/export/src/docx.rs`: pandoc argument construction (reference-doc, resource-path, extensions, toc, highlight-style); `ExportOptions` gains the corresponding fields.
- `crates/export/src/engine.rs`: option plumbing; failure reasons propagated instead of flattened to `None`.
- `src/export.rs`, `src/app/documents.rs`: read `[export]` config values (new `pandoc_path`, `reference_doc` keys beside `pdf_engine`); failure-reason disclosure in the status message (new i18n strings in `src/i18n.rs`).
- `assets/templates/reference.docx`: new bundled template; build/packaging (`build.rs`, `packager.toml`) must include it.
- The internal-AST round-trip before pandoc stays for now (it normalizes Markion-specific syntax); extension flags close the fidelity gap instead.

Non-goals: bundled/auto-installed pandoc, PDF engine changes, export-options UI (tracked by `add-docx-export-options`), changes to the built-in fallback writer.
