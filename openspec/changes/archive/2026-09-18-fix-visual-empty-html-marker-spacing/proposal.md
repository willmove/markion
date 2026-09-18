## Why

In Visual Edit, a content-free HTML marker — typically a bookmark anchor like `<a id="english"></a>` on its own line — renders with roughly a blank line of pure chrome above and below the row, which the user reports as excessive whitespace. Two paths produce this:

1. The anchor line is not a CommonMark HTML block (a type-7 open tag must be followed only by whitespace, but `</a>` follows), so it parses as a paragraph whose only events are inline HTML. Those are consumed by the inline style state machine, the paragraph flush drops the zero-span draft, and the bytes fall into an `Unsupported` source-island row (border, padding, gray background, margins) that also swallows the following blank line.
2. Genuine empty HTML blocks (`<div></div>`) keep the collapsible island chrome (border, `p_2` payload padding, `mb_3`) around a presentation that flattens to nothing.

## What Changes

- Content-free HTML-only paragraphs (tag runs with no visible text at any depth) now route to the HTML block renderer instead of being dropped, so their bytes are owned by an `Html` preview block rather than an unsupported island.
- Visual Edit renders HTML blocks whose flattened parts carry no visible content (no image, no table, no non-whitespace text) as one slim, muted, monospace source line with no border, background, padding, or block margins. The slim row remains the block's editing surface: pointer placement, caret, IME, and typing keep working through the same payload field projection.
- Content-bearing HTML blocks, inline HTML mixed with prose text (which keeps the styled-paragraph path), tables, and Read/Split Preview rendering are unchanged.

Non-goals: hiding the marker text entirely, removing authored blank lines around the row, or extending the supported inline-HTML subset.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: adds a requirement for how Visual Edit presents content-free HTML markers — a compact editable marker row without island chrome or extra vertical spacing, and parse-level ownership by the HTML block path.

## Impact

- `src/lib.rs`: `html_only_paragraph_source` also accepts tag runs without visible text.
- `src/app/preview.rs`: `visual_html_editor` gains the content-free compact-row branch; `html_preview_block_view` is refactored to accept precomputed parts so the emptiness check does not duplicate flattening work.
- No mutation, cache, or persistence changes; blocks stay cached per version.
- Tests: lib unit test for the paragraph classification, parse-level flattening test, and a GPUI regression asserting the anchor row spans about one paragraph line and stays editable.
