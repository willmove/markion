## 1. Spec deltas

- [x] 1.1 `specs/markdown-editing/spec.md` — RENAME `Source-backed Visual Edit mode` → `WYSIWYG Visual Edit mode`; MODIFIED body of that requirement (WYSIWYG-first contract, canonical-source invariant preserved, gaps point at roadmap); MODIFIED `Editor view modes` (drop "source-backed visual editing surface", point at roadmap); MODIFIED `Visual Edit inline formatting fidelity` (escaped/decoded syntax reclassified as a roadmap gap); MODIFIED `Maintained Visual Edit support classification` (3-class taxonomy: rendered WYSIWYG / progressive-reveal WYSIWYG / roadmap gap); ADDED `WYSIWYG coverage roadmap` (5 primary gaps + secondary gaps, priority/effort/seam per gap).
- [x] 1.2 `specs/code-and-math/spec.md` — MODIFIED `Direct fenced-code editing in Visual Edit` (diagram = WYSIWYG not island; unclosed/ambiguous fence = roadmap gap).
- [x] 1.3 `specs/diagram-rendering/spec.md` — RENAME `Diagram blocks remain source-backed in Visual Edit` → `Diagram blocks render WYSIWYG in Visual Edit`; MODIFIED body (rendered WYSIWYG framing, keep payload-editor invariant).
- [x] 1.4 `specs/engineering-quality/spec.md` — MODIFIED `Visual Edit invariant evidence` (drop "fallback strategy" as if accepted; require roadmap updates); MODIFIED `Markdown parser ownership` (drop "MUST select a complete source-backed fallback" as forced choice; reframe as roadmap gap).
- [x] 1.5 `specs/document-typography/spec.md` — MODIFIED `Configurable rendered-document font size` (replace "source-backed Visual Edit islands" with "Visual Edit surfaces").
- [x] 1.6 `specs/project-documentation/spec.md` — MODIFIED `Bilingual project overview` (replace "Visual Edit support/fallback behavior" + "support matrix" with "Visual Edit WYSIWYG coverage" + "WYSIWYG coverage matrix").

## 2. Design + proposal

- [x] 2.1 Write `proposal.md` (why: spec framing contradicts product goal; what: WYSIWYG-first commitment + roadmap; capabilities modified; non-goals: no code in this change).
- [x] 2.2 Write `design.md` with the full WYSIWYG gap inventory (already-WYSIWYG table, 5 primary gaps with severity/effort/seam, secondary gaps), the 5 design decisions (WYSIWYG-first not -only; canonical-source invariant preserved; progressive reveal is WYSIWYG-compatible; 3-class taxonomy; roadmap as spec requirement), and risks.

## 3. Validation

- [x] 3.1 `openspec validate commit-visual-edit-to-wysiwyg` — confirm deltas parse and MODIFIED requirement bodies match current spec headers. _(Result: "Change is valid")_
- [x] 3.2 `openspec doctor` — confirm no broken references after this change is staged. _(Result: "OpenSpec root: ok")_
- [x] 3.3 Note in proposal that this is a spec-only change: no `cargo test`/`cargo build` is affected (no code touched).

## 4. Future implementation changes (NOT part of this change — listed for traceability)

Each closes one or more roadmap gaps and will cite the `WYSIWYG coverage roadmap` requirement as motivation:

- [ ] `fix-escaped-punctuation-wysiwyg-projection` — closes gaps 1 & 2 (escaped punctuation + entity decoding).
- [ ] `render-inline-html-in-visual-edit` — closes gap 3 (inline HTML in prose).
- [ ] `render-html-blocks-in-visual-edit` — closes gap 4 (standalone HTML blocks, reusing `html_preview_block_view`).
- [ ] `render-frontmatter-form-in-visual-edit` — closes gap 5 (YAML/TOML/JSON frontmatter).
- [ ] Secondary-gap changes (indented code, inline-dollar math block, reference images, footnote definitions, heading attributes, task-list checkbox click, GFM alerts/definition lists) — opened as picked up.
