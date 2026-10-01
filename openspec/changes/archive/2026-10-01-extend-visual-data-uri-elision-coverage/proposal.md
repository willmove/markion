# Proposal: extend-visual-data-uri-elision-coverage

## Why

Device testing after `fix-visual-data-uri-source-toggle-freeze` showed multi-megabyte base64 still rendered verbatim in Visual Edit: the elision only covered the block-level Markdown image source toggle, while raw-HTML `<img src="data:…">` blocks, caret-revealed inline images (Markdown and `<img>` atoms in prose), revealed link destinations, and the source-island fallback for unprovable image spans all display authored data-URI bytes through other paths. Users reasonably expect the opaque payload to collapse everywhere it appears, not only behind one toggle.

## What Changes

- **One conservative scanner, all surfaces.** A shared derivation of "opaque data-URI payload ranges" (after the RFC 2397 comma, up to the enclosing delimiter, with mediatype and prefix-delimiter guards against prose false positives) replaces the image-editor-specific elision policy. Every Visual Edit surface that displays authored data-URI bytes splices the same atomic summary token (`…{size}…`, chip styling): the image source payload, raw-HTML block payloads, source-island fallbacks, caret-revealed inline image/`<img>` runs, and revealed link destinations.
- **Removal of the per-editor elision policy.** `VisualBlockEditor::Image`'s `elision` field (and its incremental-shift handling) is deleted; the render path computes splices from the scanner, keeping per-frame work linear with a tiny constant (a `data:` scan) while the spliced display text and clones stay bounded by the shown text. The data-URI destination fingerprint for forced-expand stays.
- **Unified locale-neutral token.** The token display becomes `…{size}…` everywhere (e.g. `…4.2 MB…`) so the library-side reveal path and the app-side payload editors render identically; the `VisualImageElidedPayload` localization is retired (the token is locale-neutral by construction: ellipsis marks plus a binary-unit size).
- **Atomic deletion generalizes.** Backspace/Delete adjacent to any elided token (image payload, HTML block payload, island, revealed run) removes the whole opaque payload in one undoable replacement, mirroring the image source toggle.

Non-goals: no elision inside fenced code, math, or diagram payloads (their bytes are the user's authored content, not an opaque blob embedded in a URL); no change to Read mode / Split rendering; no change to image decoding or the fingerprint forced-expand semantics; no new localization for the token.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: Adds a requirement that every Visual Edit surface which displays authored data-URI bytes elides them with the shared atomic token (covering raw-HTML payloads, source islands, caret-revealed inline images and `<img>` atoms, and revealed link destinations), superseding the image-toggle-only scope of the elision paragraph added by `fix-visual-data-uri-source-toggle-freeze`.

## Impact

- `src/visual.rs` — scanner (`data_uri_payload_ranges`), shared size formatter, splice into `build_visual_projection`'s revealed source pieces; removal of the image-specific elision policy.
- `src/model.rs` — `ImageSourceElision` removed; `VisualBlockEditor::Image` keeps only the fingerprint.
- `src/source_mapped.rs` — elision shift handling removed.
- `src/app/preview.rs` — field projections for `ImageSource`/`HtmlSource` and the source-island view splice via the scanner; token styling for multiple tokens.
- `src/lib.rs` — atomic token deletion scans the active block's span.
- `src/i18n.rs` — `VisualImageElidedPayload` retired.
- Invariants touched: projections remain per-caret-move builds with linear cost class (the scan replaces the previous whole-piece copy); per-version derivation shape is unchanged (no new cached state); collapsed payload editors stay lazy.
