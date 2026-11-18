## Decisions

- Keep the parser's full item end alongside the narrower visual row. Source-unit operations include content after descendants, and incremental derivation shifts both ranges together. Indentation uses the actual list ancestors and validates the changed nesting before emitting one edit.
- Keep separators at list boundaries during moves. Validate that a moved list cannot acquire unrelated following prose.
- Prefer existing link context in the rich model and suppress automatic anchors within existing HTML anchors. Track escaped extension delimiters from source offsets so HTML and preview keep them literal.
- Escape prose in the two importers while emitting actual source formatting through their existing markup writers. Traverse Word run children in order. Reuse the run renderer in rectangular cells and report fidelity limits in unsupported cells.
- Reuse the active document palette for native Markdown and HTML tables, including edit controls.
- Reuse the existing inline image view and image cache in rendered Markdown cells. Cell focus reveals the authored Markdown for editing.

## Verification

Compare parsed structure and complete source edits in regression tests. Verify real Windows table rendering and representative editing paths with an isolated profile.
