## Context

Visual Edit builds blocks from `PreviewBlock`s, then re-parses each prose slice in `inline_runs` with document link-reference definitions appended so reference-style links resolve. Footnote references need the same document-scoped context, but footnote stubs are not appended today — so `[^label]` falls apart into literal `[` / `^label` / `]` runs.

Separately, `derive_preview_and_outline` ignores `FootnoteDefinition` wrappers, so the definition body becomes an ordinary `Paragraph` while the `[^label]:` marker falls into an uncovered gap. `build_visual_blocks` turns non-whitespace gaps into `Unsupported` source islands — the gray boxes in the Notes screenshot. Link reference definition lines never emit preview events either, so they become the same islands.

## Goals / Non-Goals

**Goals:**

- Footnote references in Visual Edit render as superscript labels (matching preview rich text / HTML).
- A footnote definition is one visual block covering the full authored `[^label]: …` range, not a split island + orphan paragraph.
- Link reference definition lines remain source-backed and editable but do not use Unsupported source-island chrome.
- Regression test locks the Notes fixture symptoms.

**Non-Goals:**

- Redesigning footnote numbering UI or click-to-jump.
- Changing HTML export (already correct).
- Reference-style images or other roadmap gaps.

## Decisions

1. **Append footnote definition stubs to `inline_runs` parse input** (same pattern as link defs).
   - *Why:* pulldown-cmark only emits `FootnoteReference` when a matching definition exists; empty stubs (`[^label]:`) are enough and emit no in-block events past the slice.
   - *Alternative:* full-document parse for every prose block — rejected (too expensive; breaks the per-block model).

2. **Track footnote depth in `derive_preview_and_outline`; emit `PreviewBlock::FootnoteDefinition`.**
   - *Why:* stops orphan paragraphs and gives Visual Edit an exact full-range block.
   - *Alternative:* post-process gaps to glue marker + following paragraph — rejected (fragile with multiline footnotes).

3. **Classify pure link-reference-definition gaps as `VisualBlockKind::ReferenceDefinition` (no `source_island`).**
   - *Why:* keeps caret/source coverage while dropping gray island chrome; still editable as source text.
   - *Alternative:* hide completely like HTML — deferred (harder caret/navigation story); muted source line is enough for this fix.

4. **Render footnote definitions as a compact labeled prose block** (superscript/muted label + content runs), not a source island.

## Risks / Trade-offs

- **[Incomplete footnote bodies]** Complex nested content inside footnotes may still be imperfect → Mitigation: single-paragraph Notes happy path first; deeper nesting stays on the roadmap if discovered.
- **[Gap misclassification]** A gap mixing definitions with other junk must stay Unsupported → Mitigation: only classify when every non-blank line is a link reference definition (not `[^…]:`).
- **[Cache invalidation]** Definition edits already force full fallback via `has_reference_use` / footnote heuristics in incremental parsing → keep those triggers intact.
