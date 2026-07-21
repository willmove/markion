## Context

Visual Edit is Markion's WYSIWYG editing mode and, since `50deb4d`, the default view mode. Its spec framing, however, still reads as if raw-source presentation is an acceptable end state: the canonical requirement is named `Source-backed Visual Edit mode` (`openspec/specs/markdown-editing/spec.md:156`), and the support matrix requirement (`:623`) explicitly classifies HTML/front-matter/diagram/ambiguous constructs as a "complete conservative source island" — a monospace, bordered, gray-background box showing the raw source.

This framing was reasonable when Visual Edit was introduced (`2026-07-10-add-visual-edit-mode`) and the safe path was to fall back to source whenever an exact byte-mapped mutation could not be proven. But it has two costs:

1. **It misrepresents the product.** Users opening Visual Edit expect a rendered document, not a document that intermittently degenerates into code boxes for escaped punctuation, HTML, or frontmatter.
2. **It encodes the gaps as a tolerated end state.** Once "source island" is an accepted classification, there is no spec pressure to close the gaps.

This change reframes the spec so the gaps become an explicit, prioritized **roadmap**. The implementation is already WYSIWYG for every common construct; the remaining gaps are concrete and bounded.

## Goals / Non-Goals

**Goals:**

- Commit the spec to WYSIWYG as the default presentation contract for Visual Edit.
- Preserve the invariant that `MarkdownDocument.text` is the single canonical editable representation — no parallel rendered-tree editing model.
- Replace the five-class support taxonomy with a three-class WYSIWYG-oriented view: **rendered WYSIWYG**, **progressive-reveal WYSIWYG** (industry-standard "reveal markers on caret entry", compatible with WYSIWYG), and **roadmap gap** (currently shows source; SHALL be closed by a future change).
- Produce a concrete, prioritized roadmap of known WYSIWYG gaps that future implementation changes cite as motivation.

**Non-Goals:**

- Implementing any of the WYSIWYG gaps. Each gap is a separate future change.
- Removing the `VisualSourceIslandKind` Rust type or the `visual_source_island_view` code path. They remain as the *current* rendering for the gaps; they will be removed construct-by-construct as the roadmap is closed.
- Changing the canonical-source mutation invariant. Visual Edit never edits a parallel rendered tree.
- Rephrasing every neutral mention of "source range" or "source-mapped" — those refer to the byte-mapping model and are orthogonal to WYSIWYG presentation.

## WYSIWYG gap inventory (basis for the roadmap)

The table below is the evidence base for the new `WYSIWYG coverage roadmap` requirement. Severity reflects how often the construct appears in real Markdown × how jarring the fallback is. Effort is a rough implementation estimate; each gap will get its own design doc when picked up.

### Already WYSIWYG (no gap)

| Construct | Current rendering | Notes |
|---|---|---|
| Paragraphs / headings / lists / blockquotes (prose) | Rendered text, structural prefix hidden until caret enters prefix | `src/app/preview.rs:2117+` |
| Inline formatting (`**`/`*`/`~~`/`` ` ``/`==`/`^`/`~`/links) | Styled text; markers revealed on caret entry | Industry-standard progressive reveal; `src/visual.rs:1142-1204` |
| Fenced code (well-formed) | Syntax-highlighted payload editor, fence hidden | Code is its own rendered form; `src/app/preview.rs:2535` |
| Math blocks (`$$…$$`, fenced) | Rendered formula + LaTeX editor below | `src/app/preview.rs:2567` |
| Mermaid / registered diagrams | Rendered image + payload editor | `src/app/preview.rs:2636` (post `render-mermaid-in-visual-edit`) |
| Markdown images (well-formed) | Image + alt/dest/title field editors | `src/app/preview.rs:2709` |
| GFM tables (well-formed) | Editable grid + row/col toolbar | `src/app/preview.rs:2836` |
| Task lists (`- [ ]` / `- [x]`) | ☑/☐ glyphs; `[ ]` revealed on prefix entry | Interaction gap: checkbox not clickable (minor) |
| Footnote references `[^id]` | Rendered as superscript | |
| Blockquotes | Left-border styled block, no `>` shown | `src/app/preview.rs:2250` |
| Horizontal rule | Styled rule | |
| Whitespace / blank lines | Passive row; thin caret line when focused | `src/app/preview.rs:2271` (post `fix-visual-edit-whitespace-caret-box`) |

### Primary WYSIWYG gaps (roadmap priorities 1–5)

| # | Construct | Current rendering | Trigger | Ideal WYSIWYG rendering | Severity | Effort | Implementation seam |
|---|---|---|---|---|---|---|---|
| 1 | Escaped punctuation (`\*`, `\_`, `\!`, …) | Whole block becomes monospace source-island box | `contains_markdown_escape(block)` sets `runs[0].conservative_fallback = true` (`src/visual.rs:998-1003`) | Render decoded text; map display runes back to escaped source ranges | High (very common in real Markdown) | Large — requires bidirectional escaped/source projection | `src/visual.rs::inline_runs` + `push_run`; projection piece kind for escaped spans |
| 2 | Entity decoding (`&amp;`, `&copy;`, `&#39;`) and smart-punctuation substitution | Whole block becomes source-island box | decoded text is not a byte-substring of source → `exact.is_none()` → `conservative_fallback` (`src/visual.rs:1125-1129`) | Render decoded text; reverse-map display runes to encoded source ranges | High | Large — requires entity/smart-punct decode/encode projection (same mechanism as gap 1) | same as gap 1; encode table lives in pulldown-cmark |
| 3 | Inline HTML embedded in a paragraph (`<br>`, `<em>…</em>`, `<img …>`) | The **entire paragraph** becomes an HTML source island | `Event::Html | Event::InlineHtml` sets `contains_html = true` (`src/visual.rs:993`); `:493` then forces `source_island = Some(Html)` | Render supported inline HTML (`<b>`, `<i>`, `<strong>`, `<em>`, `<code>`, `<del>`, `<br>`, `<img>`) as styled runs; reveal source on caret entry | Medium-high (common in mixed Markdown/HTML docs) | Medium — reuse `html_preview_parts` (`src/parse.rs:545`) at inline granularity | `src/visual.rs::inline_runs` HTML event arm; `HtmlPreviewBuilder` |
| 4 | HTML block (`PreviewBlock::Html`) | Raw HTML bytes in monospace box | `src/visual.rs:437-440` always maps `PreviewBlock::Html` to `(Unsupported, Some(Html))` | Render via existing `html_preview_block_view` (`src/app/preview.rs:2939-2978`) read-only; reveal source on caret entry | Medium-high (Split Preview already does this) | Small-medium — code path exists in the same file | `visual_block_from_preview` HTML arm + reuse `html_preview_block_view` |
| 5 | Frontmatter (YAML/TOML/JSON) | Raw YAML bytes in monospace box; TOML `+++` not detected at all | `src/visual.rs:292-300` always pushes a `FrontMatter` source island; `split_front_matter` (`src/frontmatter.rs:7-38`) only handles `---` | Render title/author/date as a styled document header; expose a small form for scalar keys; keep complex mappings as a collapsible YAML editor. Detect TOML `+++` and JSON `;;;` | Medium | Medium — `YamlFrontMatter` model already parsed (`src/model.rs:984`); needs UI + TOML detection | `src/frontmatter.rs` + new `visual_frontmatter_view` |

### Secondary WYSIWYG gaps (roadmap priorities 6+)

| # | Construct | Current rendering | Notes / effort |
|---|---|---|---|
| 6 | Indented code blocks | `Code` source island (no fence to parse payload from) | Small — already highlighted via `app.highlighted_code` |
| 7 | Unclosed / malformed fenced code | `Code` source island | Small — render as highlighted code without hiding fences |
| 8 | Inline-dollar math at block position | `Math` source island | Small — render as math image |
| 9 | Reference-style images, malformed images | `Image` source island | Medium — reference resolution requires document-wide lookup |
| 10 | Malformed tables (cell-count mismatch, escaped pipes) | `Table` source island | Small — render best-effort grid |
| 11 | Footnote **definitions** (`[^id]: …`) | No `PreviewBlock::FootnoteDefinition`; falls to `Unsupported` island | Medium — needs new `PreviewBlock` kind + visual block kind |
| 12 | Heading attributes (`# H {#id}`) | Literal `{#id}` text inside heading run | Small — strip and reveal on caret entry |
| 13 | Task-list checkbox interaction | Rendered as ☑/☐ glyph but not clickable | Small — add `on_mouse_down` on marker div (`src/app/preview.rs:2229`) |
| 14 | GFM definition lists, alerts/callouts | Not enabled in `markdown_options`; render as ordinary paragraphs/blockquotes | Small (enable option) to Medium (visual design) |
| 15 | Non-whitespace gap bytes between known blocks (`gap_block`) | `Unsupported` source island | Catch-all; close construct-by-construct as parser extends |

## Decisions

### Decision 1: WYSIWYG-first, not WYSIWYG-only

**Choice:** The spec commits to WYSIWYG as the **default presentation contract**: every Markdown construct SHALL be rendered as close to its preview/result form as the editor can edit through an exact, lossless source mutation. Constructs that currently cannot be rendered are classified as **roadmap gaps**, not accepted end states.

**Why:** This matches the product's stated goal and the user's mental model. The implementation already meets this contract for every common construct; the gaps are concrete and bounded (see inventory).

**Alternatives considered:**
- *Keep the source-backed framing.* Rejected — it contradicts the default-mode flip and encodes gaps as a tolerated end state, removing spec pressure to close them.
- *WYSIWYG-only (no roadmap).* Rejected — the gaps exist today and ignoring them in the spec would make the spec lie about the implementation.

### Decision 2: Preserve the canonical-source invariant

**Choice:** WYSIWYG is a **presentation/editing** commitment, not a parallel document model. `MarkdownDocument.text` remains the single canonical editable representation; every Visual Edit mutation SHALL go through the existing source-mutation path (dirty-state, undo/redo, autosave, recovery, per-tab isolation). No rendered-tree editing.

**Why:** This invariant is the entire reason Visual Edit can coexist with Edit/Source mode and with external file changes. Dropping it would require a bidirectional Markdown↔rendered-tree sync, which is an unsolved problem in the editor space.

**Alternatives considered:**
- *Permit a rendered-tree editing model for rich constructs (e.g. direct table cell drag, image resize handles).* Rejected for now; such interactions would still need to round-trip through canonical source. Each future interaction change can re-evaluate this trade-off for its specific construct.

### Decision 3: Progressive marker reveal is WYSIWYG-compatible

**Choice:** The existing "reveal `**` / `[…](…)` / `$…$` markers when the caret enters the construct" mechanism is classified as **progressive-reveal WYSIWYG**, a first-class WYSIWYG class — not a fallback. This is the industry-standard pattern (Notion, Typeractive, Google Docs' Markdown mode).

**Why:** WYSIWYG does not mean "never see Markdown syntax" — it means "the default view matches the rendered result." Revealing syntax only at the point of editing is the established way to keep WYSIWYG editable without a rich-text-tditor overlay.

**Alternatives considered:**
- *Always hide markers (pure rich-text-style editing).* Rejected — would require inferring user intent (did they delete one `*` or both? did they mean to break the link?), which is exactly what the canonical-source invariant forbids.

### Decision 4: Three-class support taxonomy

**Choice:** Replace the five-class matrix (rendered / progressive-reveal / dedicated-editor / passive / source-island) with three classes:

1. **Rendered WYSIWYG** — the construct is shown in its rendered form (paragraphs, headings, code blocks with hidden fences, rendered math/diagrams/images, tables, blockquotes, lists, task-list glyphs, footnote refs, horizontal rules).
2. **Progressive-reveal WYSIWYG** — the construct is shown rendered by default and reveals its smallest complete source syntax group when the caret enters it (inline formatting, links, inline math, structural prefixes).
3. **Roadmap gap** — the construct currently shows raw source; the WYSIWYG coverage roadmap commits to closing it in a future change (escaped punctuation, entities, inline HTML, HTML blocks, frontmatter, malformed variants of otherwise-supported constructs, footnote definitions, heading attributes, definition lists/alerts).

The "dedicated field/payload editor" class (code/math/diagram/image/table editors) is folded into Rendered WYSIWYG: those editors ARE the rendered form of their constructs (a code block's rendered form is highlighted code; a math block's rendered form is the formula image with its LaTeX one keystroke away).

**Why:** The old taxonomy treated "source island" as a peer of "rendered," which made the gaps invisible. The new taxonomy makes the gaps a distinct, committed-to-close class.

**Alternatives considered:**
- *Keep five classes.* Rejected — see above.
- *Two classes (WYSIWYG / gap).* Rejected — collapses the useful distinction between pure-rendered and reveal-on-edit, which affects UX expectations.

### Decision 5: Roadmap as a spec requirement, not a doc

**Choice:** The roadmap lives in `openspec/specs/markdown-editing/spec.md` as an ADDED requirement (`WYSIWYG coverage roadmap`), not in `docs/`. Future implementation changes cite this requirement by name in their proposal's "Why."

**Why:** A spec requirement has spec-level force (it SHALL be maintained; changes that affect coverage MUST update it). A `docs/` file is advisory and drifts.

**Alternatives considered:**
- *Put the roadmap in `docs/visual-edit-wysiwyg-roadmap.md`.* Rejected — loses spec-level force; the README link obligation in `project-documentation` already points at "the Visual Edit support matrix," which this requirement replaces.

## Risks / Trade-offs

- **[Spec drift if roadmap is not maintained]** Once the roadmap is a spec requirement, every WYSIWYG-gap change MUST update it or the spec lies. → Mitigation: the `engineering-quality` `Visual Edit invariant evidence` requirement is MODIFIED in this change to require roadmap updates; `openspec validate` surfaces missing updates when the matrix scenario is touched.
- **[Spec now describes behavior the implementation doesn't fully deliver]** The WYSIWYG-first commitment is aspirational for the named gaps. → Mitigation: the `WYSIWYG coverage roadmap` requirement explicitly says the gaps are *open* and SHALL be closed by future changes; scenarios are written in the present tense only for constructs that are already WYSIWYG. The roadmap is honest about what is not yet implemented.
- **[Reconciliation with recently-archived changes]** `render-mermaid-in-visual-edit`, `fix-visual-edit-whitespace-caret-box`, and `render-high-quality-math` were archived with source-backed vocabulary in their deltas (now synced into specs). This change supersedes that wording in the same archive cycle. → Mitigation: this change's deltas MODIFY exactly those requirements; archive order is `render-high-quality-math` → `render-mermaid-in-visual-edit` → `fix-visual-edit-whitespace-caret-box` → `commit-visual-edit-to-wysiwyg`, so the final synced spec carries the WYSIWYG-first wording.
- **[Future change authors must learn the new taxonomy]** Contributors proposing Visual Edit changes now need to classify against the three-class model and update the roadmap. → Mitigation: the `Maintained Visual Edit support classification` scenarios spell this out; the support-matrix requirement is updated in lockstep.

## Migration Plan

Spec-only change; no code, persistence, settings, or network impact. The single archive step syncs the new wording into `openspec/specs/`. Rollback is `git revert` of the archive commit.

## Open Questions

- Should the roadmap name **target versions** for each gap (e.g. "v0.2.0: HTML blocks"), or stay version-free and let release planning pick up items by priority? **Default:** version-free. The roadmap records *priority* and *effort*, not commitments to a release; release planning picks items off the roadmap.
- Should the `VisualSourceIslandKind` type be renamed (e.g. to `VisualPresentationGapKind`) to reflect the new framing, or left as-is until each variant is individually removed? **Default:** leave as-is in this spec-only change; renaming is a code change that belongs with the first gap closure.
