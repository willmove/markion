## Why

Decoded HTML entities in prose (`&amp;`, `&#39;`, `&#x2014;`) are the highest-priority open gap on the `WYSIWYG coverage roadmap` (priority 1, archived by `commit-visual-edit-to-wysiwyg`): today a paragraph containing any entity becomes a **permanent** monospace source-island box, because the decoded `Event::Text` is not a byte-substring of the authored source and `push_text_runs` (`src/visual.rs:1826-1863`) falls back to `force_fallback`. The backslash-escape projection shipped in `2026-08-18-render-visual-edit-escapes-and-inline-html` proved the bidirectional decoded-text projection pattern; this change applies the same pattern to entities.

## What Changes

- A prose `Event::Text` whose difference from its authored slice is proven to consist of backslash escapes **and/or** HTML entity references decodes into normal rendered runs instead of a conservative fallback run: each entity token becomes a run whose visible text is the decoded character, with the remaining `&…;` bytes hidden until the caret enters the construct.
- New `VisualRevealKind::Entity` reveal group: entering an entity reveals its complete authored token (`&amp;`, `&#39;`, `&#x2014;`); leaving hides it again. It composes with Markdown formatting and with escapes, exactly as `Escape` groups do today.
- The decoder covers **all numeric character references** (`&#NN;`, `&#xHH;`) and a curated set of single-codepoint named entities that matches pulldown-cmark byte-for-byte (core punctuation/typographic/currency/symbol set including `amp`, `lt`, `gt`, `quot`, `apos`, `nbsp`→U+00A0, `copy`, `reg`, `trade`, `hellip`, `mdash`, `ndash`, quotes, `bull`, `deg`, `euro`, …). Unprovable forms — named entities outside the table, multi-codepoint entities such as `&NotEqualTilde;`, invalid/undecodable references — keep the conservative fallback (roadmap gap narrows, not silently guessed).
- `parse.rs`'s attribute-facing `decode_html_entity` is **not** reused for the projection (its `nbsp`→space mapping is lossy); the accurate table lives beside the visual projection and the two are kept separate.
- Roadmap bookkeeping: the decoded-entities gap is removed from the `WYSIWYG coverage roadmap` and the support matrix row moves to progressive-reveal WYSIWYG; docs and both READMEs' gap lists are updated.

## Capabilities

### New Capabilities

<!-- None — no new capability folders. -->

### Modified Capabilities

- `markdown-editing`: `Visual Edit inline formatting fidelity` gains entity rendering/reveal/composition scenarios and narrows its conservative clause to unprovable entity forms; `Maintained Visual Edit support classification` moves decoded entities from the gap list to progressive-reveal; `WYSIWYG coverage roadmap` removes the closed gap (front matter and indented code become priorities 1–2).

## Impact

- **Code**: `src/visual.rs` (`push_text_runs` arm + a `entity_matches`-style reconstruction prover + generalized decoded-text run splitting alongside `push_escaped_text_runs`), `src/model.rs` (`VisualRevealKind::Entity`), reveal-group validation (`reveal_candidate_is_exact`), and the view layer's reveal-kind handling in `src/app/preview.rs`. No parser-options, document-model, persistence, or exporter changes; mutations keep flowing through the canonical source path.
- **Behavior**: paragraphs with covered entities render as normal prose instead of permanent source islands; uncovered entity forms behave exactly as before (conservative island), so no regression path exists for exotic documents.
- **Docs**: `docs/visual-editing-quality.md` matrix + roadmap row; README/README.zh-CN gap wording.
- **Non-goals**: entities inside table cells (keep flattened text, Read-mode parity), inside link destinations/titles, in inline-code spans (already raw), multi-codepoint named entities, and a full HTML5 2200-entry table (extension path only).
