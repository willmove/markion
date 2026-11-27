## Why

The HTML preview places every image in a full-width row. In README badge paragraphs, this stacks linked and unlinked badges vertically even when their combined width fits.

## What Changes

- Lay out consecutive HTML images on the same line and wrap whole images when the pane narrows.
- Align each line as a group using the HTML alignment and preserve badge order, sizing, and individual links.
- Non-goals include mixed text/image reflow and browser-specific CSS layout.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing` specifies wrapping and alignment for adjacent HTML images in the existing rendered WYSIWYG coverage class.

## Impact

The shared HTML parts and rendered view gain image grouping metadata. Document source, image cache identities, and per-version Markdown caching retain their existing ownership.
