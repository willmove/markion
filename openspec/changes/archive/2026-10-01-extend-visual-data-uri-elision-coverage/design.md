# Design: extend-visual-data-uri-elision-coverage

## Context

The prior change elided data-URI payloads only inside the block-level Markdown image source toggle (`VisualBlockEditor::Image.elision`, computed at derivation). Device testing showed four other surfaces still render authored data-URI bytes verbatim:

```
authored data-URI bytes displayed by Visual Edit
├── image source toggle payload      ← elided (prior change)
├── raw-HTML block payload editor    ← verbatim  (visual_html_editor → HtmlSource field)
├── source-island fallback row       ← verbatim  (visual_source_island_view, identity projection)
└── caret-revealed inline runs       ← verbatim  (build_visual_projection: inline Markdown
                                             images AND inline <img> atoms share
                                             VisualRevealKind::HtmlImage reveal groups;
                                             revealed pieces are whole-span source;
                                             revealed Link destinations expose data: URLs too)
```

The prior change's `engineering-quality` requirement already forbids per-character layout queries and mandates display-bounded elided text; this change extends coverage without weakening it.

## Goals / Non-Goals

**Goals:** one deterministic elision mechanism covering every surface above; identical token and atomic-edit semantics everywhere; less model surface (delete `ImageSourceElision`).

**Non-Goals:** fenced code / math / diagram payloads; Read/Split rendering; image decode pipeline; localization of the token (locale-neutral by construction: `…` + binary-unit size).

## Decisions

### D1. Shared conservative scanner replaces the per-editor policy

`data_uri_payload_ranges(text, range) -> Vec<Range<usize>>` (visual.rs, exported) scans `text[range]` once and returns opaque payload ranges:

- A candidate starts at `data:` (case-insensitive) only when the preceding byte is one of `( " ' = <`, whitespace, or the range start — the delimiters that actually introduce a URI in Markdown destinations, HTML attributes, and angle destinations. This excludes `metadata:`, `mydata:` inside words.
- Between `data:` and the first `,` there must be no whitespace, quotes, parens, or angle brackets (a mediatype), and it must contain a `/`, start with `;` (e.g. `;base64` without a mediatype), or be empty — `(see data:foo,bar)` prose fails this and stays verbatim.
- The token runs from after the comma to the first of `) " ' < >`, whitespace, or the range end, and must be non-empty and char-boundary-clamped. Base64 and percent-encoded payloads contain none of those bytes, so the end delimiter is exact for well-formed URIs; malformed ones degrade to verbatim (safe direction).

Cost is one linear memchr-class scan per displayed range — the same order as the existing text copy the reveal path already performs per projection rebuild, with no per-character layout queries (the display-bounded requirement's letter and spirit hold).

### D2. Splice at projection build, not stored ranges

- **Reveal path** (`build_visual_projection`, `ProjectionPiece::Source` arm): the piece's payload ranges are spliced — verbatim segments around one atomic segment per token; token spans carry the inline-code style family (soft background tint) through the existing span→highlight pipeline, with the `…{size}…` text itself carrying the ellipsis semantics.
- **Field editors** (`visual_editor_field_projection` consumers in preview.rs): `ImageSource` and `HtmlSource` fields splice via the same scanner; multiple tokens per field each get their own atomic segment and chip highlight.
- **Source island** (`visual_source_island_view`): the identity projection becomes a spliced projection; the `StyledText` gains token highlights.

Rationale: stored per-block ranges would need incremental shifting and a new cached field; the scan is per-display-build (caret-move / expanded paint), linear, and stateless — no new versioned state to keep coherent. If profiling later shows jank on huge HTML blocks, memoize per (version, block id) as a follow-up.

### D3. Token display is locale-neutral everywhere

`…{size}…` with `format_byte_size` (moved to visual.rs, exported) — e.g. `…4.2 MB…`. The library-side reveal path has no app/i18n access, and threading a label callback through `build_visual_projection`'s ~30 call sites (including the incremental-vs-full comparison in source_mapped.rs) is churn without user value; ellipsis marks plus a binary size read unambiguously in every locale. The `VisualImageElidedPayload` message is retired. The prior spec's example token (`…4.2 MB…`) already matches this form.

### D4. Atomic deletion scans the active span

`visual_atomic_token_edit` drops its dependence on the removed `elision` field: it scans the caret's containing image/HTML block span with the same scanner and treats token-boundary adjacency (Backspace at token end, forward-Delete at token start) as whole-payload removal. Runs at keypress time only, linear in the block.

### D5. Model surface shrinks

`ImageSourceElision` (model.rs), the `elision` field on `VisualBlockEditor::Image`, `image_source_elision` (visual.rs), and its shift handling (source_mapped.rs) are deleted. The `data_uri_fingerprint` for forced-expand and its completion-time failure set stay exactly as shipped. Table cells render flattened alt text (image spans contribute empty span text), so no splice is needed there; a regression test pins that a data-URI image in a table cell never pastes payload bytes.

## Risks / Trade-offs

- [Scanner false positives elide prose that looks like a data URI] → the mediatype and prefix-delimiter guards reject the realistic prose shapes; a false negative (verbatim display) is the failure direction for anything that slips through, and edits remain byte-exact regardless.
- [Per-frame scan on expanded HTML payloads] → linear with a tiny constant, and the expanded editor previously copied the whole span into the display text anyway; the splice actually reduces bytes copied.
- [Inline token styled via the code-chip family rather than a bespoke chip] → visual consistency with inline code; the `…` framing plus size label distinguishes it.
- [Removing the derivation-time policy churns tests added hours ago] → those tests are rewritten against the scanner; behavior-level assertions (token presence, atomic replace, undo) stay identical.

## Migration Plan

Presentation-only; no persisted state. Reverts cleanly with the prior change since both only touch rendering paths.

## Open Questions

- Whether an HTML block mixing many small data-URI icons (hundreds of tokens) needs a per-field token count cap — defer until a real document shows a problem.
