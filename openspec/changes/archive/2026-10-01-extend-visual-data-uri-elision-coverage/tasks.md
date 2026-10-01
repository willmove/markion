# Tasks: extend-visual-data-uri-elision-coverage

## 1. Scanner and shared splice

- [x] 1.1 Implement `data_uri_payload_ranges(text, range)` in src/visual.rs (prefix-delimiter + mediatype guards, comma-to-delimiter token, char-boundary clamp) with unit tests covering data URIs in Markdown/angle/HTML-attribute contexts, uppercase `DATA:`, prose false positives (`(see data:foo,bar)`, `metadata:x`), empty payloads, and range clamping.
- [x] 1.2 Move `format_byte_size` to src/visual.rs (exported via lib.rs) and build the shared token text helper `…{size}…`.
- [x] 1.3 Splice `ProjectionPiece::Source` in `build_visual_projection`: verbatim segments + one atomic segment per token, token spans styled through the inline-code chip family; unit tests assert the revealed inline image/`<img>`/link projections contain the token and not the payload bytes.

## 2. Field editors, island, and model cleanup

- [x] 2.1 Splice `ImageSource` and `HtmlSource` field projections via the scanner (replacing the derivation-time elision branch), with per-token chip highlights; update the image/HTML editor wiring so no elision parameter is threaded.
- [x] 2.2 Splice the `visual_source_island_view` projection and its `StyledText` highlights.
- [x] 2.3 Delete `ImageSourceElision` (model.rs), the `Image.elision` field, `image_source_elision`, and the shift handling in source_mapped.rs; keep `data_uri_fingerprint`; compile clean and update existing elision tests to scanner behavior.
- [x] 2.4 Retire the `VisualImageElidedPayload` message from src/i18n.rs (all language tables).

## 3. Interaction and coverage tests

- [x] 3.1 Generalize `visual_atomic_token_edit` to scan the containing image/HTML block span; Backspace/Delete adjacency tests for an HTML block payload.
- [x] 3.2 Windowed tests: expanded raw-HTML `<img src="data:…">` payload shows the token (not bytes); caret entry into an inline prose image/`<img>` reveals an elided token; island fallback for an unprovable data-URI image elides; a table cell containing a data-URI image never displays payload bytes.
- [x] 3.3 Update the prior change's integration tests to the unified token text; verify the collapsed-frame and navigation-query gates still pass unchanged.

## 4. Validation

- [x] 4.1 `cargo test --workspace` green; `openspec validate extend-visual-data-uri-elision-coverage --strict` passes; WYSIWYG coverage matrix rows for raw-HTML images and inline images mention the shared elision.
