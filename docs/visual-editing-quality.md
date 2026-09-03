# Visual Edit Support and Engineering Contract

Markion's Visual Edit mode is WYSIWYG-first: rendering is the default presentation, and raw Markdown source is the canonical storage representation, not the default view. `MarkdownDocument.text` remains the only persisted representation — there is no second rich-text document model. A visual interaction is supported only when it can map to an exact, UTF-8-safe source mutation. Constructs whose WYSIWYG rendering is not yet implemented show their authored source as a **transitional editing affordance** and are tracked on the WYSIWYG coverage roadmap below — they are known gaps to close, not an accepted end state.

## Support Matrix

| Construct | Normal Visual Edit presentation | Canonical editable range | Roadmap-gap trigger (transitional source view) | Required evidence |
|---|---|---|---|---|
| Paragraphs, headings (including empty ATX headings such as `##` / `###     `), list/task items (including empty items such as `- `) | Rendered direct text with reserved heading/list row height when the payload is empty; mixed-fragment rows (link/footnote icons, math atoms, HTML images) keep authored soft/hard line breaks as stacked wrap rows | Exact inline content and structural prefix ranges | Byte-inexact or crossing parser events | Projection round-trip, formatting, structural Enter/Backspace, pointer (click, drag, double-click word selection with hidden-marker edge exclusion), IME, undo, mixed-layout line stacking, empty-heading/list placeholder height |
| Mixed blockquote flows (paragraph/list/task leaves) | Ordinary rendered rows inside one quote boundary; no recursive editor | One disjoint leaf range plus exact per-line quote-marker ranges and optional inner list prefix | Overlapping ownership, ambiguous marker partition, or byte-inexact inline events | Intro/list/outro order, complete non-overlapping coverage, independent prefix reveal, quoted structural editing, stable identity |
| Emphasis, strong, strike, inline code, links (including angle-bracket autolinks), highlight, super/subscript, inline math, backslash-escaped ASCII punctuation, decoded HTML entity references (proven named table, including multi-codepoint names), supported inline HTML (`em`/`i`, `strong`/`b`, `s`/`del`/`strike`, `code`, `mark`, `sub`, `sup`, `br`, ignorable `class`/`id`/`clear`) | Rendered with progressive source reveal; navigation icons and atoms stay on the logical line that owns the construct; escapes hide the backslash byte, entity runs render the decoded character(s) while the complete authored `&…;` token stays hidden until reveal, and HTML tags stay hidden markers whose styles compose with Markdown formatting (`<br>` is an atomic line-break run). Unknown or unpaired inline HTML stays as inert conservative atoms in the same mixed row | Smallest complete proven syntax group (`\X` pair, complete `&…;` entity token, full open-tag…close-tag element, or `<url>` / `<email>` autolink) | Malformed, overlapping, or ambiguous syntax; entity references outside the proven decode tables; escapes or inline HTML outside the byte-proven subset | Reveal-group containment, caret affinity, cross-run selection, UTF-8 input, mixed-layout line stacking, escape gap-splitting, entity decode-table parity with the parser, HTML pairing/demotion, `<br>` atomic boundaries, autolink reveal |
| Ordinary fenced code | `VisualBlockEditor::Code`: highlighted payload editor with fences hidden; the language label is hidden while the pointer is outside the fence and the fence does not own the caret, and appears as an explicit click-target chip over the first info-string token (localized placeholder for bare fences) while hovered or caret-owned | Payload only; opening fence, info string, and closing fence are immutable metadata — except the first info token, which is editable through the revealed chip and commits one exact token replacement (inserted after the fence when bare) | Unclosed/ambiguous fence or registered diagram backend | Exact fence/payload/info ranges, memoized highlights, delimiter preservation, edge handoff, language-chip reveal on hover/caret, token sanitization (no whitespace/backticks), bare-fence insertion, IME/history |
| List item with nested fenced code | The item text renders as a normal editable list row; the nested fence renders below it as the same `VisualBlockEditor::Code` payload editor, exactly once each | The item row owns only its direct text (its source range ends at the nested block start); the code editor owns the complete authored fence including list indentation | Unclosed/ambiguous nested fence (lenient closing scan requires the opening fence indented 4+ and a uniformly indented payload); blockquote/HTML constructs nested in items keep their existing presentation | Document-ordered disjoint block stream, contiguous coverage without unsupported islands, no duplicated text, exact caret positions at the partition boundary, guard against fence-like payload lines |
| Display/fenced block math | `VisualBlockEditor::Math`: rendered formula plus LaTeX payload editor | LaTeX payload only; delimiters remain authored | Inline-only/ambiguous form | Valid/invalid/pending render, delimiter preservation, CJK/emoji IME geometry, one-action undo |
| Inline Markdown image | Image-only paragraphs keep the bounded preview, caption, missing-resource placeholder, and focused replace/width/alignment controls. A paragraph, heading, quote, or list item that **mixes** `![...](...)` with other prose (including leading same-line `![img](url)trailing` and `text ![img](url) more`) keeps the image as an inline atom on the same visual line as adjacent text, matching Read / Split Preview; focused caret reveals the authored `![…](…)` bytes. A block-level image with a proven exact span additionally carries the collapsible-source chrome: hover `</>` expands one monospaced payload over the complete authored span above the image, and an unloadable destination forces the payload visible; data-URI payloads render as one atomic, chip-styled size-labeled summary token (`…{size}…`) that edits as a single unit while the structural syntax stays verbatim — the same shared token covers raw-HTML block payloads, source-island fallbacks, focused table cells, caret-revealed inline image/`<img>` runs, and revealed link destinations | Complete authored inline-image source for a standalone image row; mixed atoms own the `![…](…)` bytes inside the parent prose row | Reference, angle-bracket destination, multiline, malformed form, or stale/ambiguous exact-image target on a standalone image-only row (such rows show no source toggle and keep the island fallback) | Source coverage, mixed inline atoms (no-blank-line, same-line, leading same-line, multiple images), quoted/heading/list-item leaves, blank-line-separated and image-only cases, caption and presentation, broken/local/remote image, whole-span payload edit as one exact replacement, forced expand on load failure, UTF-8, and one-edit undo; data-URI elision on every surface via the shared scanner (token boundaries, atomic replacement/deletion, fingerprint-based forced expand, collapsed-frame zero span work, HTML/inline/island/table-cell coverage) |
| Contextual inline formatting and links | Selection toolbar for bold, italic, inline code, and links; focused URL/title link editor | One non-empty selection inside one proven editable inline run, or one complete exact inline-link source | Cross-run selection, conservative/malformed syntax, stale document version/range, or reference-style link | Presentation-only version stability, exact UTF-8 range, URL/title escaping, confirm/cancel, IME input, and one-edit undo/redo |
| Slash commands and block transformations | Localized filtered slash palette; focused-row Turn Into menu for Text, H1-H6, bulleted/numbered/task list, quote, fenced code, divider, and table | Versioned slash-query line or one validated `VisualBlockId` plus exact source range | Stale version/range, source island without a proven payload, nested list ownership, multi-row quote group, or overlapping source | Keyboard/pointer palette, Escape dismissal, UTF-8/CRLF templates, exact prefix/payload preservation, IME query input, and one-edit undo/redo |
| Block duplicate, delete, and reorder | Focused-row duplicate/delete/move actions plus a separate drag grip and before/after drop zones | One complete newline-safe source unit including deterministic following separator whitespace | Nested list parent/child, quote-group leaf, overlapping range, unsupported island, stale drag, or drop into self | EOF/final-newline cases, button/drag helper equivalence, stale no-op, multi-tab isolation, and one-edit undo/redo |
| GFM table | `VisualBlockEditor::Table`: editable header/body cells plus an interaction-gated row/column toolbar (hover or caret in the table) with structural boundary disabling and whole-table delete through exact block delete | Logical cell owned by the canonical caret for row/column edits; one deterministic full-table replacement for width reflow; whole-table delete uses the complete newline-safe source unit | Unequal/ambiguous cell boundaries, malformed table, stale toolbar target, a toolbar whose table does not own the caret, or nested/quoted tables whose block delete is unsupported | Escaped-pipe projection, UTF-8 width reflow, alignment/export preservation, traversal, hover/caret toolbar visibility without version changes, toolbar cell ownership and boundary availability, whole-table delete undo, multi-table isolation, IME, and one-edit undo/redo history |
| Horizontal rule | Passive rendered rule | Exact block boundary through ordinary navigation/format commands | N/A | Source coverage and navigation tests |
| Blank lines and trailing whitespace | Empty-paragraph-height Visual Edit row (body line height); click and Up/Down enter the existing source range; focused thin caret, not a source island. Click is caret placement only (no inserted newline) | Exact whitespace range | N/A | Complete source coverage; gap click/arrow without version change; typing at the separator newline; no pointer-created phantom edits |
| Mermaid/registered diagrams | Rendered diagram image plus its source payload editor | Payload only; the complete fence is immutable metadata | Unclosed/ambiguous fence | Registry classification, source preservation, async/cache/version isolation |
| Inline raw-HTML `<img>` in prose | Rendered image atom in the inline flow (same loader as preview: local/remote/data-URI) with progressive source reveal of the complete tag | Complete byte-exact `<img …>` tag | A non-tag/partial/multi-tag slice, missing `src`, or GFM table-cell context (pipe-table cells keep flattened alt/URL text, matching Read mode) | Exact tag ranges, reveal-on-focus/restore-on-blur, no whole-block island, claim/preload/eviction parity with block-level images |
| Raw HTML blocks (including `<img>`) | Rendered through the shared HTML-parts pipeline (text, images, tables with content-proportional `rowspan`/`colspan` columns, headings, lists, `pre`, alignment/color/underline); `VisualBlockEditor::Html` collapsible source payload on the full authored block | Complete authored block | Overlapping ownership or a failed HTML-parts parse | Exact source preservation, preview/export behavior, shared-pipeline parity with Read mode, nested list/quote HTML images |
| Other inline HTML | Inert conservative atoms (verbatim tag source) in the mixed prose row; no whole-block island | The authored tag range | N/A (unknown tags stay atoms; YAML front matter is a separate construct) | Mixed-layout focused and unfocused, unpaired tags do not island the paragraph |
| YAML front matter | Complete source-backed island with lightweight chrome (left accent, faint fill, tight padding — not a padded bordered card) | Complete authored block | Always until the frontmatter gap closes | Exact source preservation |
| Unsupported or malformed constructs | Transitional source view over the complete containing range, using the same lightweight island chrome | Complete containing source range | Exact mapping cannot be proven | Lossless source-mode round-trip and no guessed mutation |

## WYSIWYG Coverage Classes

Every user-visible construct belongs to exactly one of three classes:

1. **Rendered WYSIWYG** — shown in its rendered form. Dedicated field/payload editors (fenced code payload plus the hover/caret-revealed info-token chip, block math, diagrams, Markdown and inline-HTML images with the collapsible whole-span source toggle above the image, GFM tables with cell editors, HTML blocks via `VisualBlockEditor::Html`) are the rendered form of their constructs. Covers prose rows, headings (including empty ATX headings), lists/task items (including empty items), blockquote flows, GFM alerts, horizontal rules, whitespace rows, footnote definitions/references, link reference definitions, and HTML blocks (shared pipeline plus collapsible source payload).
2. **Progressive-reveal WYSIWYG** — rendered by default; reveals its smallest complete source syntax group when the caret enters it. Covers emphasis, strong, strikethrough, inline code, highlight, super/subscript, links (including reference-style and angle-bracket autolinks), inline math, backslash escapes, decoded HTML entity references (proven named table, including multi-codepoint names), the supported inline-HTML subset (style pairs with ignorable `class`/`id`/`clear`, `<br>`, inline `<img>`), inert unknown inline-HTML atoms, structural prefixes, and heading attributes.
3. **WYSIWYG coverage gap** — currently shows authored source as a transitional affordance; tracked on the roadmap below until a change closes it.

The old five-class taxonomy folded its "dedicated editor" class into rendered WYSIWYG and reclassified "source island" as roadmap gaps.

## WYSIWYG Coverage Roadmap

Prioritized open gaps (refreshed 2026-08-29; each closes via a future change citing the `WYSIWYG coverage roadmap` requirement):

| Priority | Construct | Current rendering | Target class | Effort | Implementation seam |
|---|---|---|---|---|---|
| 1 | Frontmatter (YAML `---`) | Permanent FrontMatter island; TOML/JSON not detected | Rendered (document header form; collapsible YAML editor) | Medium | `src/visual.rs:445-453`; `src/frontmatter.rs:7-38`; `YamlFrontMatter` in `src/model.rs` |
| 2 | Indented code blocks | Code island; no payload editor (fence scan requires `` ` ``/`~`) | Rendered (payload editor over the indented body) | Small | `src/visual.rs::fenced_payload_ranges` (`:1183-1185`), `visual_block_editor` |
| 3 | Unclosed/malformed fenced code | Code island | Rendered (highlighted code, fences visible) | Small | `src/visual.rs:1164-1268` |
| 4 | Reference-style/malformed inline images | Renders unfocused; focused → island; no field controls | Rendered | Medium | `src/inline_edit.rs:66-82` (`LinkType::Inline` only) |
| 5 | Malformed tables (ragged rows) | Best-effort grid unfocused; island focused | Rendered | Small | `src/table.rs:182-221` |
| 7 | Task-list checkbox click | Glyph not interactive | Rendered (click toggles `[ ]`/`[x]`) | Small | `src/app/preview.rs:2919-2944` |
| 9 | GFM definition lists | Not enabled; `: Def` renders as literal prose | Rendered | Small–Medium | `src/parse.rs:1738-1747` (`ENABLE_DEFINITION_LIST` absent) |
| 11 | Math render-failure states | Island on focus while KaTeX is Pending/Error | Rendered (payload editor until Ready) | Small | `src/app/preview.rs:3149-3151` |
| 12 | Residual gap bytes between known blocks | Lightweight Unsupported island (catch-all) | Closed construct-by-construct | — | `src/visual.rs:857-892` |

Closed in `keep-empty-structure-visual-and-soften-islands`: empty ATX headings and empty list items (former gap 10) stay rendered with prefix reveal; remaining source islands use lightweight chrome instead of a padded bordered card.

Closed in `improve-visual-edit-html-rendering`: unsupported inline HTML (former gap 6) is mixed inert atoms; angle-bracket autolinks (former gap 8) share link reveal; residual named/multi-codepoint entities (former gap 13) decode through the proven tables.

Reviewed divergences (deliberate, not gaps): bare-URL autolinking and `:emoji:` conversion are Preview-only (`src/parse.rs` extended-inline vs Visual Edit `visual_markdown_options`); GFM *pipe*-table cells that contain HTML `<img>` stay flattened to alt/URL text (Read-mode parity). HTML `<td>`/`<th>` images and links render through the HTML-parts table grid.

## Source-Range Invariants

Every derived range must:

1. Be contained by the current canonical document and start/end on UTF-8 boundaries.
2. Be contained by its owning `VisualBlock`; direct fields and delimiter metadata must not overlap unrelated source.
3. Round-trip to the intended authored slice, including CRLF, indentation, blank lines, escaping, and final-newline semantics.
4. Move only through a consecutive `SourceEdit` chain or be rebuilt by the full-derivation fallback.
5. Reject a `VisualBlockEdit` when document version, `VisualBlockId`, field metadata, or replacement policy is stale.
6. Preserve exact post-edit selection/marked ranges after sanitization or table reformatting.

For blockquotes, the container owns no visual row. Its ordered paragraph/list
children partition the quote source into disjoint rows, while structural-only
`>` separator lines become quote-context whitespace rows. Quote markers and an
inner list/task marker are separate reveal and edit layers: Enter continues the
combined structure, and Backspace removes the inner layer before the quote.

Interaction-only state such as focus, hover, selection, caret affinity, layout geometry, Tab traversal, and scroll position must not change the document version or invalidate per-version derived caches.

## Parser Ownership

- `pulldown-cmark` in the root document model owns semantic Markdown classification.
- Visual Edit uses the same semantic extensions but disables smart punctuation
  for its source-backed inline projection. ASCII quotes and dash sequences stay
  byte-identical and editable there; Split Preview, Read, HTML/LaTeX/DOCX export,
  and other presentation parsers retain smart punctuation.
- `src/visual.rs` proves byte boundaries only inside an already-classified preview block. It must return no direct metadata when round-trip proof fails.
- `src/table.rs` owns the shared GFM table range, parse, format, and mutation rules used by source commands and Visual Edit.
- `src/inline_edit.rs` owns the narrow exact parser and serializer for inline links and images. It accepts only complete byte-proven inline forms and returns no mutation target for reference, multiline, angle-bracket, escaped, or otherwise ambiguous syntax.
- `src/block_edit.rs` owns slash-query recognition, canonical block templates, validated structural transforms, and the shared newline-safe source-unit operation used by duplicate/delete/button move/drag drop. It is GPUI-free and refuses stale, nested, quoted-group, or overlapping ownership.
- `src/storage/resources.rs` owns managed image naming, collision/content reuse, copying, and safe workspace-relative Markdown URLs. The canonical document still contains only Markdown text and never stores a parallel image object.
- `crates/markdown` owns its GPUI-free parser/AST and exporter contracts. It does not own a parallel Visual Edit mutation model.
- Exporters consume canonical Markdown semantics; a widget never mutates an exporter model, preview block, image object, or math cache directly.

A new boundary helper must not become a second document parser. Prefer an exact, narrow recognizer plus source-island fallback over broader guessed support.

## Verification Layers

- Pure document tests: exact ranges, escapes, stale edits, table reflow, full/source round-trips.
- Differential/property tests: randomized UTF-8 edits and incremental output versus a fresh full derivation.
- Rendered GPUI tests: projection, caret/selection, keyboard handoff, platform input, IME bounds, undo/redo, mode switching, multi-tab isolation, and virtualization.
- Workspace tests: parser, diagram, exporter, and doctest compatibility.
- Deterministic performance tests: parsed/reused-region counters, stable IDs, shared `Arc` identity, and bounded dirty-region work.
- Informational benchmark: `cargo run --release --example bench_large_doc`. Timing is diagnostic and is not a merge threshold without dedicated benchmark hardware.

## Quality Gate

Run the complete local gate from the repository root:

```powershell
pwsh ./scripts/check-quality.ps1
```

It runs formatting, `cargo test --workspace`, and strict validation of every OpenSpec change/spec. CI runs the same gate in `.github/workflows/quality.yml`. Tests that explicitly require external network access, pandoc, or a PDF engine remain reported as ignored when their prerequisites are absent.

## Change Checklist

Every proposal that adds or changes a visual strategy must state:

- The strategy: rendered direct text, progressive reveal, dedicated editor, passive exact position, or source island.
- The semantic parser owner and the exact source-range proof owner.
- The canonical edit range, post-edit selection, escaping/normalization, and stale-event policy.
- The malformed/ambiguous fallback trigger.
- UTF-8/CRLF, pointer/keyboard, CJK/emoji IME, semantic undo, multi-tab, cache/identity, large-document, source-mode, and exporter evidence affected by the change.
- The corresponding row update in this matrix and delta spec changes.
