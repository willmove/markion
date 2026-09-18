## Why

Markdown images already accept Markion's source-backed presentation metadata such as `{width=50 align=center}`, and the stable Markdown editing requirements already promise that preview and Visual Edit apply those settings. The current implementation drops that metadata before Read/Split rendering and uses the wrong flex axis for Visual Edit, so images appear left-aligned and ignore proportional sizing.

## What Changes

- Preserve parsed Markdown image presentation metadata through the cached preview-block and inline-image model.
- Apply width percentages and horizontal alignment to standalone Markdown images in Read mode and Split Preview.
- Correct Visual Edit's Markdown image layout to use horizontal flex justification while retaining its existing resize interaction and aspect-ratio behavior.
- Add parser, layout, and cross-mode regression coverage for centered 50%-width Markdown images.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- None. The existing `markdown-editing` and `document-resources` requirements already define the intended image presentation behavior; this change restores their implementation.

## Impact

- `src/parse.rs`, `src/lib.rs`, and `src/model.rs` will carry image presentation metadata in the per-document-version derived state.
- `src/app/preview.rs` will share the presentation contract across Visual Edit, Read, and Split Preview.
- Existing image cache/loading, source mutation, and export paths remain unchanged.
- The cached-per-version Markdown derivation invariant is preserved; rendering will not reparse source text on every frame.

Non-goals: changing the Markdown metadata syntax, adding new width/alignment values, or altering mixed inline-image layout.
