## Context

Visual Edit is Markion's WYSIWYG editing mode and, since `50deb4d`, the default view mode. Its spec framing, however, still reads as if raw-source presentation is an acceptable end state: the canonical requirement is named `Source-backed Visual Edit mode` (`openspec/specs/markdown-editing/spec.md:162`), and the support matrix requirement (`:727`) explicitly classifies HTML/front-matter/diagram/ambiguous constructs as a "complete conservative source island" — a monospace, bordered, gray-background box showing the raw source.

This framing was reasonable when Visual Edit was introduced (`2026-07-10-add-visual-edit-mode`) and the safe path was to fall back to source whenever an exact byte-mapped mutation could not be proven. But it has two costs:

1. **It misrepresents the product.** Users opening Visual Edit expect a rendered document, not a document that intermittently degenerates into code boxes for escaped punctuation, HTML, or frontmatter.
2. **It encodes the gaps as a tolerated end state.** Once "source island" is an accepted classification, there is no spec pressure to close the gaps.

This change reframes the spec so the gaps become an explicit, prioritized **roadmap**. The inventory below was **refreshed on 2026-08-19** against the implementation: five of the original primary/secondary gaps were closed by interim changes between 2026-07-21 and 2026-08-18, and three previously unlisted gaps were discovered (angle-bracket autolinks, empty list items, math render-failure states).

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

The tables below are the evidence base for the `WYSIWYG coverage roadmap` requirement, verified against the implementation on 2026-08-19. Severity reflects how often the construct appears in real Markdown × how jarring the fallback is. Effort is a rough implementation estimate; each gap will get its own design doc when picked up.

### Already WYSIWYG (no gap)

| Construct | Current rendering | Notes |
|---|---|---|
| Paragraphs / headings / lists / blockquotes (prose) | Rendered text, structural prefix hidden until caret enters prefix | `src/app/preview.rs:2117+` |
| Inline formatting (`**`/`*`/`~~`/`` ` ``/`==`/`^`/`~`/links) | Styled text; markers revealed on caret entry | Industry-standard progressive reveal; `src/visual.rs:1142-1204` |
| Backslash-escaped punctuation (`\*`, `\\`, …) | Literal punctuation rendered; `\X` pair revealed on caret entry | Closed 2026-08-18; `escape_matches` (`src/visual.rs:1999`) + `VisualRevealKind::Escape` (`src/model.rs:737`) |
| Supported inline HTML (`em`/`i`, `strong`/`b`, `s`/`del`/`strike`, `code`, `mark`, `sub`, `sup`, `<br>`) | Styled runs; complete element revealed on caret entry | Closed 2026-08-18; `VisualRevealKind::InlineHtml` (`src/model.rs:738`), pair validation `src/visual.rs:2213` |
| Inline raw-HTML `<img>` in prose | Image atom in the inline flow (local/remote/data-URI) | Closed 2026-08-14 (`render-visual-edit-html-images`) |
| Standalone HTML blocks | Rendered read-only through the shared HTML-parts pipeline; focused block keeps the source-island editing affordance | Closed by `6220b33`; `src/visual.rs:993,1038-1044` |
| Reference-style links (`[text][label]`, collapsed, shortcut) | Resolved against document definitions, rendered like inline links | Closed 2026-07-21 (`resolve-reference-links-in-visual-edit`) |
| Fenced code (well-formed, incl. list-nested) | Syntax-highlighted payload editor, fences hidden | `src/app/preview.rs:2535`; list-nested fix 2026-08-18 |
| Math blocks (`$$…$$`, fenced) and inline math | Rendered formula + payload editor; inline `$…$` is a baseline atom with reveal (including at block position — `Event::InlineMath` routes through the paragraph, never a Math island) | `src/app/preview.rs:2567`, `src/lib.rs:2471-2493` |
| Mermaid / registered diagrams | Rendered image + payload editor | `src/app/preview.rs:2636` (post `render-mermaid-in-visual-edit`) |
| Markdown images (well-formed) | Image + alt/dest/title field editors; images nested in paragraphs/headings split into disjoint text/image rows | `src/app/preview.rs:2709`; nested split `564aff2` |
| GFM tables (well-formed) | Editable grid + row/col toolbar | `src/app/preview.rs:2836` |
| Task lists (`- [ ]` / `- [x]`) | ☑/☐ glyphs; `[ ]` revealed on prefix entry | Interaction gap: checkbox not clickable (roadmap #8) |
| Footnote references `[^id]` and **definitions** `[^id]: …` | Superscript refs; definitions render as editable rows with hidden marker (`VisualBlockKind::FootnoteDefinition`, island `None`) | Definitions closed by `fix-visual-edit-footnotes-and-link-defs`; `src/visual.rs:1014`, `src/app/preview.rs:3177-3207` |
| Link reference definitions (`[label]: url`) | Editable muted rows without island chrome (`VisualBlockKind::ReferenceDefinition`) | Same change; `src/visual.rs:881`, `src/app/preview.rs:3709-3750` |
| Heading attributes (`# H {#id}`) | `{#id}` consumed by the parser and hidden as marker ranges; revealed on caret entry | `src/visual.rs:2609-2627` |
| GFM alerts / callouts (`> [!NOTE]`) | Styled callout with accent title row and left-border body group | `CalloutTitle` `src/visual.rs:896-926`, `src/app/preview.rs:3041-3074` |
| Blockquotes | Left-border styled block, no `>` shown | `src/app/preview.rs:2250` |
| Setext headings (`===`/`---` underlines) | Rendered as headings; underline bytes hidden as markers | Same pipeline as ATX; `src/visual.rs:2609-2627` |
| Horizontal rule | Styled rule | |
| Whitespace / blank lines | Passive row; thin caret line when focused | `src/app/preview.rs:2271` (post `fix-visual-edit-whitespace-caret-box`) |
| Smart punctuation (`--` → en dash, quotes) | Not substituted: Visual Edit parsing disables `ENABLE_SMART_PUNCTUATION`, so ASCII quotes/dashes stay byte-identical and editable | `src/parse.rs:1750-1753`; presentation parsers keep smart punctuation |

### Closed since the roadmap was first drafted (2026-07-21 → 2026-08-19)

| Original gap | Closed by | Evidence |
|---|---|---|
| 1. Escaped punctuation | `2026-08-18-render-visual-edit-escapes-and-inline-html` (`abb0ca6`) | `escape_matches` + `VisualRevealKind::Escape` |
| 2 (half). Smart-punctuation substitution | Visual Edit parser options (`src/parse.rs:1750-1753`) | `ENABLE_SMART_PUNCTUATION` removed for the visual projection |
| 3 (most). Inline HTML in prose | `2026-08-18-render-visual-edit-escapes-and-inline-html` + `2026-08-14-render-visual-edit-html-images` | Supported subset + `<img>` atoms; residual forms stay on the roadmap |
| 4. Standalone HTML blocks | `6220b33` "Render HTML blocks via shared preview pipeline" | `VisualBlockKind::Html` renders read-only; island on focus |
| 8. Inline-dollar math at block position | Existing routing (`src/lib.rs:2471-2493`) — never was a Math island | `Event::InlineMath` → inline math atom with `$…$` reveal |
| 11. Footnote definitions (+ link reference definitions) | `fix-visual-edit-footnotes-and-link-defs` (complete, awaiting archive) | `FootnoteDefinition`/`ReferenceDefinition` block kinds |
| 12. Heading attributes `{#id}` | Marker-ranges mechanism (`src/visual.rs:2609-2627`) | Hidden until caret enters the row |
| 14 (half). GFM alerts | Callout rendering (`src/visual.rs:896-926`) | Title row + styled body group |
| (adjacent) Nested Markdown images in prose | `564aff2` `fix-visual-edit-inline-markdown-images` (complete, awaiting archive) | Paragraph/heading split into disjoint text/image rows |

### Open WYSIWYG gaps (roadmap priorities, refreshed 2026-08-19)

Primary gaps:

| # | Construct | Current rendering | Trigger | Ideal WYSIWYG rendering | Severity | Effort | Implementation seam |
|---|---|---|---|---|---|---|---|
| 1 | Decoded HTML entities in prose (`&amp;`, `&#39;`, …) | Whole paragraph becomes a **permanent** monospace source-island box (worse than focused-only islands) | `push_text_runs` force-fallback when visible ≠ source and escapes can't explain it (`src/visual.rs:1836-1863`) → `always_source` (`src/app/preview.rs:2838-2842`) | Render decoded text; reverse-map display runes to encoded source ranges — same bidirectional projection pattern as escapes | High (common; permanent island) | Medium — mirror `escape_matches` (`src/visual.rs:1999`) with an entity decode/encode table; `decode_html_entity` already exists (`src/parse.rs:1471`) | `src/visual.rs::push_text_runs` + a new `entity_matches` sibling |
| 2 | Frontmatter (YAML `---`) | Raw YAML bytes in a permanent monospace box; TOML `+++`/JSON not detected at all | `build_visual_blocks` always pushes a `FrontMatter` island (`src/visual.rs:445-453`); `split_front_matter` handles `---` only (`src/frontmatter.rs:7-38`) | Render title/author/date as a styled document header; small form for scalar keys; complex mappings as a collapsible YAML editor; detect TOML/JSON forms | Medium-high (every Jekyll/Hugo-style doc) | Medium — `YamlFrontMatter` model already parsed (`src/model.rs:984`); needs UI + TOML detection | `src/frontmatter.rs` + new `visual_frontmatter_view` |
| 3 | Indented code blocks | `Code` source island; never gets the highlighted payload editor | `CodeBlockKind::Indented` → `PreviewBlock::CodeBlock { language: None }` (`src/lib.rs:2270-2279`); `fenced_payload_ranges` requires a `` ` ``/`~` opening fence (`src/visual.rs:1183-1185`) → editor `None` | Highlighted payload editor whose payload is the indented body and whose "fences" are the shared indent | Medium (common in older docs) | Small — add an indented-payload arm to `visual_block_editor` | `src/visual.rs::fenced_payload_ranges`/`visual_block_editor` |

Secondary gaps:

| # | Construct | Current rendering | Notes / effort |
|---|---|---|---|
| 4 | Unclosed / malformed fenced code | `Code` island (strict closing scan `src/visual.rs:1205-1216`; lenient scan only for list-nested indented fences `:1217-1258`) | Small — render as highlighted code without hiding the fences |
| 5 | Reference-style images (`![alt][ref]`) and malformed images | Image renders unfocused from the resolved destination; focused → island; no replace/width/alignment controls (`src/inline_edit.rs:66-82` accepts `LinkType::Inline` only) | Medium — extend `inline_image_at` to reference forms; malformed images already stay literal prose |
| 6 | Malformed tables (ragged rows) | Best-effort grid unfocused; island on focus (`src/table.rs:182-221` refuses unequal cell counts) | Small — best-effort grid editor |
| 7 | Unsupported inline-HTML forms (unknown/attributed tags, stray or crossing pairs) | Prose renders with verbatim tag fragments; island on focus (`src/visual.rs:1732-1746,1749-1773,1786-1821`) | Medium — broaden the exact recognizer or render unknown tags as inert atoms |
| 8 | Task-list checkbox click | ☑/☐ glyph not interactive (`src/app/preview.rs:2919-2944`); toggle only via transform menu | Small — `on_mouse_down` flips `[ ]`/`[x]` |
| 9 | Angle-bracket autolinks (`<https://…>`, `<a@b.c>`) | **Whole paragraph** falls back to a source island: the link reveal candidate fails the `starts_with('[')` check (`src/visual.rs:2276-2278`) → reveal invalidation (`:2202-2205`) | Small — teach `reveal_candidate_is_exact` the angle-bracket link form; code-path derived, no dedicated test yet (discovered 2026-08-19) |
| 10 | GFM definition lists | Not enabled (`ENABLE_DEFINITION_LIST` absent from `src/parse.rs:1738-1747`); `Term` / `: Def` render as ordinary paragraphs with literal `:` | Small (enable + basic styling) to Medium (visual design) |
| 11 | Empty list items (`- ` with no text) | Dropped by `flush_list_item` (`src/parse.rs:112-126`) → their line becomes an `Unsupported` gap island (`src/lib.rs:1280-1283`) | Small — keep empty items as editable rows (discovered 2026-08-19) |
| 12 | Math render-failure states | Editorless math whose KaTeX render is Pending/Error → island on focus (`src/app/preview.rs:3149-3151`) | Small — show payload editor until the render is Ready |
| 13 | Non-whitespace gap bytes between known blocks (`gap_block`) | `Unsupported` island except whitespace-only, quote-marker, and all-reference-definition gaps (`src/visual.rs:857-892`) | Catch-all; close construct-by-construct as the parser extends |

Reviewed divergences (not source-island gaps; recorded here so they are re-evaluated deliberately):

- **Bare-URL autolinking and `:emoji:` conversion are Preview-only.** Visual Edit's `ExtendedInlineKind` covers only highlight/superscript/subscript (`src/parse.rs:440-445`), while the preview builder converts bare URLs and emoji codes (`src/parse.rs:262-267`). Visual Edit shows the authored text literally — a source-fidelity choice, acceptable under WYSIWYG, but a visible Preview/Visual difference.
- **Inline HTML inside table cells** keeps flattened alt/URL text, matching Read mode.

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

**Choice:** The existing "reveal `**` / `[…](…)` / `$…$` markers when the caret enters the construct" mechanism is classified as **progressive-reveal WYSIWYG**, a first-class WYSIWYG class — not a fallback. This is the industry-standard pattern (Notion, Typeractive, Google Docs' Markdown mode). The 2026-08-18 escapes/inline-HTML work validated the model further: `\X` pairs and inline-HTML elements now render literally and reveal their smallest complete source group on caret entry.

**Why:** WYSIWYG does not mean "never see Markdown syntax" — it means "the default view matches the rendered result." Revealing syntax only at the point of editing is the established way to keep WYSIWYG editable without a rich-text-editor overlay.

**Alternatives considered:**
- *Always hide markers (pure rich-text-style editing).* Rejected — would require inferring user intent (did they delete one `*` or both? did they mean to break the link?), which is exactly what the canonical-source invariant forbids.

### Decision 4: Three-class support taxonomy

**Choice:** Replace the five-class matrix (rendered / progressive-reveal / dedicated-editor / passive / source-island) with three classes:

1. **Rendered WYSIWYG** — the construct is shown in its rendered form (paragraphs, headings, code blocks with hidden fences, rendered math/diagrams/images, tables, blockquotes, alerts, lists, task-list glyphs, footnote rows, rules, HTML blocks).
2. **Progressive-reveal WYSIWYG** — the construct is shown rendered by default and reveals its smallest complete source syntax group when the caret enters it (inline formatting, links, inline math, escaped punctuation, supported inline HTML, structural prefixes, heading attributes).
3. **Roadmap gap** — the construct currently shows raw source; the WYSIWYG coverage roadmap commits to closing it in a future change (decoded entities, front matter, indented code, unclosed fences, reference-style images, malformed tables, unsupported inline-HTML forms, autolinks, task-list checkbox interaction, definition lists, empty list items, math failure states).

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

### Decision 6: The roadmap is refreshed against the implementation before this change archives

**Choice:** Because implementation changes landed between drafting (2026-07-21) and archiving, this change's delta was rebased on 2026-08-19: the `Visual Edit inline formatting fidelity` delta now starts from the current spec body (which already commits escapes/inline-HTML/reference-link rendering) and reframes only the conservative wording; the roadmap lists the remaining gaps, not the ones interim changes closed.

**Why:** MODIFIED deltas replace the whole requirement body at archive time. Archiving the stale draft would have silently reverted the escape/inline-HTML WYSIWYG commitments that `2026-08-18-render-visual-edit-escapes-and-inline-html` synced into the spec — and would have shipped a roadmap that misstates five constructs as gaps.

**Alternatives considered:**
- *Archive the original draft and fix the drift in a follow-up change.* Rejected — a spec window where the spec lies about shipped behavior, plus avoidable churn.

## Risks / Trade-offs

- **[Spec drift if roadmap is not maintained]** Once the roadmap is a spec requirement, every WYSIWYG-gap change MUST update it or the spec lies. → Mitigation: the `engineering-quality` `Visual Edit invariant evidence` requirement is MODIFIED in this change to require roadmap updates; `openspec validate` surfaces missing updates when the matrix scenario is touched. The 2026-08-19 refresh (Decision 6) is the first exercise of this discipline — five constructs were removed from the draft roadmap because interim changes closed them.
- **[Spec now describes behavior the implementation doesn't fully deliver]** The WYSIWYG-first commitment is aspirational for the named gaps. → Mitigation: the `WYSIWYG coverage roadmap` requirement explicitly says the gaps are *open* and SHALL be closed by future changes; scenarios are written in the present tense only for constructs that are already WYSIWYG. The roadmap is honest about what is not yet implemented.
- **[Delta staleness between proposal and archive]** This change sat unarchived while gap-closing changes landed. → Mitigation applied: Decision 6 rebase. Going forward, gap-closing changes should archive in the same cycle they land, or check this change's delta for collisions.
- **[Future change authors must learn the new taxonomy]** Contributors proposing Visual Edit changes now need to classify against the three-class model and update the roadmap. → Mitigation: the `Maintained Visual Edit support classification` scenarios spell this out; the support-matrix requirement is updated in lockstep.

## Migration Plan

Spec-only change; no code, persistence, settings, or network impact. The single archive step syncs the new wording into `openspec/specs/`. Rollback is `git revert` of the archive commit.

## Open Questions

- Should the roadmap name **target versions** for each gap (e.g. "v0.2.0: frontmatter"), or stay version-free and let release planning pick up items by priority? **Default:** version-free. The roadmap records *priority* and *effort*, not commitments to a release; release planning picks items off the roadmap.
- Should the `VisualSourceIslandKind` type be renamed (e.g. to `VisualPresentationGapKind`) to reflect the new framing, or left as-is until each variant is individually removed? **Default:** leave as-is in this spec-only change; renaming is a code change that belongs with the first gap closure.
- Should the reviewed divergences (bare-URL/emoji Preview-only conversion) eventually become parity gaps on the roadmap, or stay documented divergences? **Default:** stay documented divergences until user feedback asks for Visual-Edit conversion.
