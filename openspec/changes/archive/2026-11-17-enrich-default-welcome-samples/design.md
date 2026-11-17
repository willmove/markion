## Context

`DEFAULT_WELCOME_MARKDOWN` in `src/lib.rs` is the in-memory untitled document created at launch (`Application::new` / `new_document()`). It already tours CommonMark/GFM and Markion extensions, but its image line is `![Local image placeholder](markion-example.png …)`. That file is not in the repo, not next to the binary, and the welcome tab has no `document_dir`, so preview, Read, and Visual Edit all show a failed local image.

Raw HTML is already parsed: `html_preview_parts` renders tables (with `rowspan`/`colspan`), images, lists, headings, `<br>`, `<kbd>`/`<code>`, underline, and styled emphasis. The welcome sample never shows those constructs.

Untitled image keys are built in `PreviewImageKey::from_url`: remote/`data:` URLs stay as-is; relative local paths join `document_dir` when present, otherwise they are treated as CWD-relative. Packaged builds already copy `assets/` next to the executable (`packager.toml` `resources`). Development builds resolve bundled files through `CARGO_MANIFEST_DIR`, as `bundled_reference_doc_path` does for the DOCX template.

Changing the welcome constant does not touch per-keystroke caching: the starter is one document version; preview blocks, outline, stats, and the preview-image cache still key off that version and the resolved image identity.

## Goals / Non-Goals

**Goals:**

- Make the welcome Markdown image resolve and display on a typical first launch (dev `cargo run` and packaged installs) without requiring a saved file or a network fetch.
- Add HTML table and other HTML-tag samples that the existing HTML preview pipeline already understands.
- Keep the sample English, non-localized, and Markion-specific.

**Non-Goals:**

- New HTML or image rendering features, network as the primary image source, a `.md` template on disk, localization of the welcome text, or changes to `packager.toml` resource lists.
- Rewriting image resolution for saved documents (those keep document-directory relative paths).

## Decisions

### Use the packaged branding PNG as the sample image destination

The welcome Markdown image SHALL use the relative destination `assets/markion.png` (the 512×512 branding raster already shipped in `assets/` and listed in packager icons/resources). Alt text names the logo rather than a “placeholder”.

This is a real file in both the repository and packaged `assets/` trees, so the sample teaches ordinary relative-image syntax instead of a dummy filename.

Alternative considered: a GitHub raw HTTPS URL. Rejected as the primary destination because first launch would depend on the network and would still fail offline. Remote images remain supported elsewhere; this sample should work without a fetch.

Alternative considered: `assets/markion-logo.svg`. Viable (preview already rasterizes SVG) but the PNG is the README/packaging raster and avoids an extra SVG decode on the onboarding surface.

Alternative considered: a `data:` URI. Rejected because it bloats the constant and does not demonstrate a filesystem-relative image.

### Resolve untitled relative images against bundled resources when CWD misses

Keep `DEFAULT_WELCOME_MARKDOWN` as a `&str` constant (no runtime path injection, so the source the user sees stays `assets/markion.png`).

Extend local-image resolution only when `document_dir` is `None` and the path is relative: if the CWD-relative file is missing, look up the same relative path under the bundled resource root — `{current_exe parent}/`, `{current_exe parent}/resources/`, macOS `{Contents}/Resources/`, then `{CARGO_MANIFEST_DIR}/` — matching `discover_workspace_assets` and `bundled_reference_doc_path`. Prefer extracting a small shared helper (for example on `src/paths.rs`) so export and preview do not duplicate the search order.

Saved documents are unchanged: they still resolve relative URLs against the document directory only.

Data flow: authored URL in the cached preview/visual block → `PreviewImageKey::from_url(url, document_dir)` → file identity `local:<canonical path>` → existing `PreviewImageCache` fetch/decode. The helper only changes which filesystem path is canonicalized for untitled relative URLs; it does not reparse Markdown or invalidate caches on keystrokes.

Alternative considered: injecting an absolute path into the welcome string at construction time. Rejected because the visible source would become a machine-specific `D:\…` / `/Applications/…` path, which is a poor sample.

### Keep HTML samples as raw HTML blocks, not mixed into the Visual Edit prose paragraph

Add two new sections after the existing GFM table (or after Code and math — implementation may pick a readable heading order):

1. **HTML table** — one CommonMark HTML block (no blank lines between tags) with `<table>`, header cells, and at least one `colspan` and one `rowspan`, using short Markion-oriented labels.
2. **Other HTML** — one or more HTML blocks demonstrating tags the flattener already maps: `<p>`/`<div>`, `<strong>`/`<em>`/`<u>`/`<s>`, `<br>`, `<ul>`/`<ol>`/`<li>`, `<kbd>`/`<code>`, `align="center"`, and an `<img src="assets/markion.png" …>` so the HTML image path shares the same bundled file.

Do not splice attributed tags (`<p align=…>`, `<kbd>`, `<u>`, `<img …>`) into the existing “Write with *italic*…” paragraph. That paragraph is the Visual Edit inline-formatting regression surface; unsupported or attributed inline HTML would collapse it into a conservative island.

A separate short prose sentence MAY show only the Visual Edit–supported unattributed subset (`<em>`, `<strong>`, `<br>`) if it stays visually editable.

HTML blocks may remain Visual Edit source islands (existing behavior). GFM pipe tables, fenced code, and display math keep their current editors.

### Tests pin new markers, not the old placeholder

Update `welcome_prose_stays_visual_outside_the_focused_block` and any other assertions on `![Local image placeholder]` / `markion-example.png` to the new image dest and HTML markers (`<table`, `colspan` or `rowspan`, at least one other HTML tag such as `<kbd>`). Keep the “Write with” paragraph visual-edit assertions. Add a focused unit test that untitled resolution of `assets/markion.png` finds a real file via the bundled-resource helper.

## Risks / Trade-offs

- [Packaged layout differs from `{exe_parent}/assets/…`] → `packager.toml` already copies the `assets` directory as a resource next to the binary; the helper should assert `is_file()` before using a candidate, then fall back to `CARGO_MANIFEST_DIR` in dev builds.
- [Untitled user docs accidentally resolve app assets] → Only the same relative path string is eligible, and only when CWD has no such file. A user writing `assets/markion.png` in an unsaved buffer seeing the logo is acceptable; arbitrary `foo.png` still fails unless it exists on CWD.
- [HTML samples become conservative Visual Edit islands] → Expected for raw HTML blocks; the Visual Edit test continues to require substantial editable prose and the unchanged inline-formatting paragraph.
- [A longer welcome document feels noisy] → Keep HTML examples compact (one small table, one short tag block) under clear headings.

## Migration Plan

No persisted data. Existing sessions that restored real files are unaffected. First launch and File → New continue to use the updated constant.

Rollback is reverting the constant and the untitled resolution fallback.

## Open Questions

None. Network images stay out of the welcome document unless a later change wants a second, explicitly labeled remote example.
