## Why

Visual Edit is described throughout the OpenSpec specs as a "source-backed, WYSIWYG-**oriented**" surface whose default failure mode is a "conservative source island" — a monospace, bordered, gray-background box that shows raw Markdown source. The canonical requirement is even named `Source-backed Visual Edit mode` (`openspec/specs/markdown-editing/spec.md:156`). This framing was a pragmatic choice when Visual Edit was new and unproven, but it now contradicts the product's stated goal and the user's mental model: Visual Edit is the default editing mode (since `50deb4d`), and users expect WYSIWYG — **what they see is what they get**, not "what they see is sometimes a code box of the underlying source."

In practice the implementation is already WYSIWYG for the most common constructs (paragraphs, headings, lists, blockquotes, inline formatting, code, math, diagrams, images, tables, task lists, footnotes). But the spec's "source island" framing encodes the *gaps* as acceptable defaults rather than as known deficits to be closed. This change reframes the spec so the gaps become an explicit roadmap that future changes close, instead of a tolerated end state.

## What Changes

- The Visual Edit spec commitment shifts from "source-backed with WYSIWYG-oriented rendering when provable" to **WYSIWYG-first**: rendering is the default; raw source is the canonical storage representation, not the default presentation.
- The `Source-backed Visual Edit mode` requirement is **renamed** `WYSIWYG Visual Edit mode` and its body rewritten to commit to rendered presentation as the default. The invariant that `MarkdownDocument.text` is the single canonical editable representation (no second editable rendered tree) is preserved — WYSIWYG is a presentation/editing commitment, not a parallel document model.
- A new `WYSIWYG coverage roadmap` requirement is added to `markdown-editing`, enumerating the known WYSIWYG gaps (escaped punctuation, entity decoding, inline HTML, HTML blocks, frontmatter, plus secondary gaps) and committing that each SHALL be closed by a future change. The roadmap is the source of truth for prioritization.
- The `Maintained Visual Edit support classification` matrix is simplified from five classes (rendered / progressive-reveal / dedicated-editor / passive / source-island) to a WYSIWYG-oriented three-class view (rendered WYSIWYG / progressive-reveal WYSIWYG / **roadmap gap**), with the former "source-island" class reclassified as known gaps.
- Companion specs (`code-and-math`, `diagram-rendering`, `engineering-quality`, `document-typography`, `project-documentation`) are reworded to drop "source island" / "conservative fallback" as if they were accepted end states, replacing them with WYSIWYG-first language that points at the roadmap for known gaps.

## Capabilities

### New Capabilities

<!-- None — no new capability folders. -->

### Modified Capabilities

- `markdown-editing`: Purpose paragraph + `Source-backed Visual Edit mode` (→ renamed `WYSIWYG Visual Edit mode`) + `Visual Edit inline formatting fidelity` + `Maintained Visual Edit support classification`; **added** `WYSIWYG coverage roadmap`.
- `code-and-math`: `Direct fenced-code editing in Visual Edit` — reword the "diagram/unclosed/ambiguous → source-backed island" arm to point at the roadmap.
- `diagram-rendering`: `Diagram blocks remain source-backed in Visual Edit` — rename to `Diagram blocks render WYSIWYG in Visual Edit`, keep the payload-editor invariant.
- `engineering-quality`: `Visual Edit invariant evidence` and `Markdown parser ownership` — drop "MUST select a complete source-backed fallback" as a forced choice; replace with "MUST not introduce a second editable rendered tree; MUST classify any non-WYSIWYG presentation as a roadmap gap."
- `document-typography`: `Configurable rendered-document font size` — replace "source-backed Visual Edit islands" with "Visual Edit surfaces."
- `project-documentation`: `Bilingual project overview` — point the "Visual Edit support/fallback" reference at the new WYSIWYG coverage roadmap instead of the support matrix.

## Impact

- **Code**: none. This is a spec-only change. No `src/` files are touched. The existing implementation already matches the WYSIWYG-first framing for all common constructs; the gaps named in the roadmap remain implementation gaps to be closed by future changes.
- **Specs**: this change edits six capability specs. After archive, the support-matrix taxonomy and "source island" vocabulary no longer appear as if they describe accepted behavior; they describe a transitional state.
- **Future work**: each roadmap gap becomes a candidate for a follow-up change (e.g. `render-html-blocks-in-visual-edit`, `render-frontmatter-form-in-visual-edit`, `fix-escaped-punctuation-wysiwyg-projection`, `render-inline-html-in-visual-edit`, etc.). Those changes will cite this roadmap requirement as their motivation.
- **Non-goals**: implementing any of the WYSIWYG gaps in this change; removing the `VisualSourceIslandKind` type (it still exists in code as the *current* rendering path for the gaps, and will be removed construct-by-construct as the roadmap is closed); changing the canonical-source invariant (Visual Edit still mutates `MarkdownDocument.text`, never a parallel rendered tree).
