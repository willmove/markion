## Why

The Visual Edit slash palette is a tall flat list that obscures the document. Recognition also includes text after the caret and the CR in CRLF line endings, making identical slash input work in some paragraphs but fail in others.

## What Changes

- Present block commands in a compact, themed button grid with H1–H5, tighter button spacing, localized tooltips, and a selected-command caption.
- Group Table, Link, Formula, and Image settings as labeled insertion actions. Reuse the existing link/formula/image editors, including local image selection and configured image resource policies. Opening or canceling settings leaves the slash query intact; confirming replaces it once.
- Recognize only the marker and query before the caret so existing paragraph text and CRLF endings do not block opening or get consumed by confirmation.
- Keep slash commands available in editable prose, headings, lists, quotes, and blank lines while excluding structured code/math/table/HTML/image/front-matter source editors.
- Preserve filtering, keyboard/pointer confirmation, Escape, stale-target validation, UTF-8 boundaries, exact undo, and cached-per-version derived state.

Non-goals: adding chart or shape commands; changing Source mode behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: compact slash palette presentation, H1–H5 shortcut headings, source-backed insertion settings, and caret-bounded, context-appropriate query recognition and confirmation.

## Impact

`src/block_edit.rs`, Visual Edit state and overlay rendering under `src/app/`, embedded UI icons, and regression tests. No new dependencies or persistence format changes. The existing canonical edit transaction and per-document-version shared caches remain authoritative.
