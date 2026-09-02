## Why

The default welcome document points at `markion-example.png`, a local path that does not exist on disk, so first launch shows a broken image instead of a working sample. The same starter also omits HTML that Markion already previews — raw HTML tables and other HTML tags — so new users never see those authoring paths.

## What Changes

- Replace the welcome document's missing local image with a destination that actually resolves: the packaged branding raster `assets/markion.png`, looked up from the application resource root (next to the executable in packaged builds, or the repository `assets/` tree during development).
- Expand the in-memory `# Welcome to Markion` sample with HTML demonstration sections covering a raw HTML `<table>` (including header cells and `colspan`/`rowspan`) and other HTML that Markion already renders (inline style tags, `<br>`, lists, `<kbd>`/`<code>`, centered `<img>`, and a small styled HTML block).
- Keep the welcome document as fixed, English, non-localized sample content. Update focused tests that assert welcome-document markers and Visual Edit coverage so they track the new image destination and HTML sections.

Non-goals: adding HTML or image rendering capabilities; localizing the welcome Markdown; fetching a network image as the primary sample; changing user documents, packaging resource lists, or per-version derived-cache identity.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Require the default welcome document to use a resolvable bundled image and to include HTML table and other HTML-tag samples alongside the existing Markdown tour.

## Impact

- Primary edit: `DEFAULT_WELCOME_MARKDOWN` in `src/lib.rs`.
- Image resolution for untitled documents must find the bundled `assets/markion.png` without a saved document directory; reuse the existing exe-adjacent / `CARGO_MANIFEST_DIR` resource lookup already used for the bundled DOCX reference template.
- Tests in `src/visual.rs` and `src/app/tests.rs` that pin welcome markers such as `![Local image placeholder]` need updating.
- Parser, preview, Visual Edit, and packaging stay as they are: HTML tables, HTML preview parts, SVG/PNG preview, and `packager.toml` `resources = ["assets", ...]` already cover the new sample.
- Invariants preserved: changing the constant starter text does not recompute derived Markdown state on keystrokes; the welcome document remains one in-memory version with shared `Arc` caches.
