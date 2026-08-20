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

Each closes one or more roadmap gaps and will cite the `WYSIWYG coverage roadmap` requirement as motivation. Refreshed 2026-08-19 against the implementation:

Already closed by interim changes (removed from the roadmap):

- [x] `render-visual-edit-escapes-and-inline-html` — closed escaped punctuation + supported inline-HTML subset (archived 2026-08-18; `abb0ca6`).
- [x] `render-visual-edit-html-images` — closed inline raw-HTML `<img>` (archived 2026-08-14; `a7a23ab`).
- [x] HTML-block rendering — closed standalone HTML blocks (`6220b33`).
- [x] `resolve-reference-links-in-visual-edit` — closed reference-style links (archived 2026-07-21).
- [x] `fix-visual-edit-footnotes-and-link-defs` — closed footnote definitions + link reference definitions (complete, awaiting archive).
- [x] `fix-visual-edit-inline-markdown-images` — closed nested Markdown images in prose (`564aff2`, awaiting archive).
- [x] Closed by existing mechanisms (no change needed): smart-punctuation substitution (disabled in the Visual Edit parser), inline-dollar math at block position (routes as inline math), heading attributes (hidden marker ranges), GFM alerts (callout rendering).

Remaining roadmap gaps (candidates):

- [ ] `fix-visual-edit-entity-projection` — closes primary gap 1 (decoded HTML entities in prose).
- [ ] `render-frontmatter-form-in-visual-edit` — closes primary gap 2 (frontmatter form + TOML/JSON detection).
- [ ] `render-indented-code-in-visual-edit` — closes primary gap 3 (indented code payload editor).
- [ ] Secondary-gap changes (unclosed fences, reference-style images, malformed tables, unsupported inline-HTML forms, angle-bracket autolinks, task-list checkbox click, definition lists, empty list items, math render-failure states) — opened as picked up.

## 5. Roadmap refresh 2026-08-19 (rebase before archive)

- [x] 5.1 Verify every gap in the 2026-07-21 inventory against the current implementation (`src/visual.rs`, `src/parse.rs`, `src/app/preview.rs`, `src/table.rs`, `src/inline_edit.rs`); record closed/partial/open with file:line evidence in `design.md`.
- [x] 5.2 Rebase the `Visual Edit inline formatting fidelity` delta on the current synced spec body so archiving cannot revert the escapes/inline-HTML/reference-link commitments synced by `2026-08-18-render-visual-edit-escapes-and-inline-html`.
- [x] 5.3 Rewrite the `WYSIWYG coverage roadmap` delta with the remaining gaps (3 primary + secondary) and newly discovered gaps (angle-bracket autolinks, empty list items, math render-failure states).
- [x] 5.4 Update `proposal.md`, `tasks.md` (this section), and the `Maintained Visual Edit support classification` gap enumeration to match.
- [x] 5.5 `openspec validate commit-visual-edit-to-wysiwyg` + `openspec doctor` pass after the refresh.
