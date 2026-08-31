## Context

Visual Edit is source-backed WYSIWYG: a construct stays rendered when the editor can map it to an exact source mutation, and only falls back to a source island when it cannot. Empty ATX headings and empty list items are valid CommonMark (pulldown-cmark already classifies them). Two later gates throw that classification away:

1. **Derivation drops empty payloads.** `push_nonempty_block` (`src/parse.rs`) omits `PreviewBlock::Heading` / `Paragraph` when `text.is_empty()`. `flush_list_item` omits unordered/ordered items with empty text (task items survive because they have a checkbox). The full-document path in `src/lib.rs` uses the same nonempty gates. Visual Edit’s `build_visual_blocks` then sees uncovered bytes and `gap_block` wraps any non-whitespace gap as `VisualSourceIslandKind::Unsupported`.
2. **The view re-islands empty runs.** `visual_block_view` (`src/app/preview.rs`) sends a caret-owning block to `visual_source_island_view` when `editable_runs.is_empty()` (`focused_conservative`), even if the block is a heading or list item. That helper always paints `p_3` + border + gray fill + code-slot monospace — the “poster” chrome also used for front matter, unclosed fences, and residual unsupported gaps.

The whitespace-caret change already proved the right pattern: keep the rendered kind, paint a caret through `VisualEditableText`, do not borrow island chrome. Empty structure needs the same split, plus prefix reveal (Typora / Obsidian: focused line shows `## ` / `- `).

Read and Split Preview currently skip empty heading/list text, so the row disappears there while Visual Edit shows a gray box. Unfocused placeholder height must match across those surfaces.

```
  source "##   " / "- "
           │
           ▼
  pulldown-cmark ── Heading / ListItem (empty text)
           │
           ▼
  today: drop ──▶ gap ──▶ Unsupported island (heavy chrome)
  this change: keep ──▶ VisualBlockKind::Heading/ListItem
                         unfocused: heading/list row height
                         focused: reveal prefix, caret, no island
```

## Goals / Non-Goals

**Goals:**

- Empty ATX headings (levels 1–6, hashes only or hashes plus spaces) remain `PreviewBlock::Heading` / `VisualBlockKind::Heading` with a byte-exact source range. They are never `Unsupported` islands.
- Empty unordered and ordered list items remain list rows the same way. Empty task items already survive derivation; they no longer island when they own the caret.
- Unfocused empty headings and empty list items occupy the same heading/list layout height in Visual Edit, Read, and Split Preview (placeholder, not a hidden row).
- When that row owns the caret or a selection endpoint, Visual Edit reveals the structural prefix (`#`–`######` plus following spaces, or the list/task marker) through the existing prefix/projection path, keeps heading or list typography, and shows a caret. The row is not replaced by `visual_source_island_view`.
- `focused_conservative` no longer treats “empty `editable_runs`” as sufficient to island a rendered kind.
- Remaining true source islands (front matter, indented/unclosed code, residual unsupported gaps that are not empty structure) keep an exact source-backed editor but use lighter chrome: no poster padding/border/height jump.
- Coverage matrix: empty list items leave the roadmap; empty ATX headings are recorded as rendered + progressive-reveal prefix, not as a new gap.

**Non-Goals:**

- Front-matter form, indented-code payload editor, unclosed-fence highlighting, malformed tables, definition lists, task-checkbox click.
- Setext headings (`Title` / `===`).
- Always showing ATX markers on headings that already have visible content (existing caret-in-prefix reveal stays; this change only guarantees reveal for empty payload and for caret at `prefix.end`).
- Changing Source-mode behavior, exporters beyond consuming the kept empty heading/list blocks, or `crates/*` GPUI rules.
- Empty paragraphs that are not headings or lists (they remain whitespace/gaps). Empty `>`-only quote lines stay on the existing quote-whitespace path.

## Decisions

### Decision 1: Keep empty headings and list items in the derived model

**Choice:** Stop dropping empty `PreviewBlock::Heading`. Stop dropping empty unordered/ordered `ListItem`s in `flush_list_item` (and the matching full-document flush). Keep dropping empty **paragraphs** — those are blank lines and already become `Whitespace`.

**Why:** The parser already knows the construct. Dropping it is what creates the unsupported gap. Keeping the block preserves source coverage, outline entries (already pushed today even when preview drops the heading), incremental region reuse, and a real `block_prefix` (`### ` / `- `).

**Alternatives considered:**

- *Classify the gap as heading-shaped in `gap_block`.* Rejected: duplicates parser knowledge, misses Read/preview (they never see a heading), and still fights `focused_conservative`.
- *Keep empty paragraphs too.* Rejected: empty paragraphs are layout, not a typed structure; whitespace already owns that.

### Decision 2: Empty structure uses the existing heading/list render arms

**Choice:** Unfocused empty heading = heading typography and spacing with no visible payload (reserved height). Unfocused empty list item = marker column + empty content (reserved height). Focused: same arms, with prefix included in the visual projection so `VisualEditableText` paints the marker and caret. No new block kind.

**Why:** Matches Typora/Obsidian and reuses caret, IME, and click-to-place behavior. Read/Split Preview paint the same reserved height so mode switches do not collapse the row.

**Alternatives considered:**

- *Unfocused: hide the row (Read today) / focused: island.* That is the current inconsistency; rejected by the placeholder decision.
- *Synthetic visible placeholder glyph (e.g. “Heading”).* Rejected: mutates neither source nor WYSIWYG; empty reserved space is enough.

### Decision 3: Reveal the structural prefix when the empty row is focused, and at `prefix.end`

**Choice:** In `build_visual_projection`, a heading/list `block_prefix` is revealed when the caret or a selection endpoint is inside the prefix **or at `prefix.end`** (`include_end: true` for prefixes). If the block has no visible content runs, owning the caret also reveals the prefix (so an empty `###` with the caret on the line always shows the hashes).

**Why:** Empty headings put the caret at the end of `###` / `###     `, which today’s `include_end: false` does not treat as active, so the marker would stay hidden even after Decision 2. Typora/Obsidian show the marker on the focused empty line. Headings with content still hide `## ` once the caret is inside the title (unchanged except the `prefix.end` boundary, which is the first title position — revealing there is correct).

**Alternatives considered:**

- *Always reveal ATX markers whenever a heading owns the caret.* Stronger Typora-gutter behavior; deferred so this change does not restyle every non-empty heading.
- *Whole-block source for the focused line (Obsidian Live Preview).* Rejected: Markion already has progressive reveal; a full-line source dump would look like a mini-island.

### Decision 4: `focused_conservative` is island-kind only, not empty-runs

**Choice:** A caret-owning block goes to `visual_source_island_view` only when it actually carries `source_island` (FrontMatter / Code / Unsupported, and other island kinds that still lack an editor), with the existing exemptions for whitespace, callout titles, and reference definitions. Empty `editable_runs` alone MUST NOT island Heading, ListItem, Paragraph, Quote, Rule, FootnoteDefinition, or HTML-with-editor.

**Why:** Empty runs were a caret-paint hack. Empty headings/lists will have a projection once the prefix is revealed; unfocused they do not own the caret. HTML blocks already use a collapsible payload editor rather than a whole-block island.

**Alternatives considered:**

- *Exempt Heading and ListItem only.* Narrower, but the same empty-runs trap would still island an empty task row or a future empty rendered kind.

### Decision 5: One lighter chrome for remaining islands

**Choice:** Keep a single `visual_source_island_view` for remaining source-only blocks, but restyle it: tight vertical padding, no large `p_3` height jump, no heavy rounded bordered “code poster.” Use a left accent (or 1px hairline) plus a faint background, code-slot font, and source-island type size closer to surrounding body line-height. Front matter, unclosed/indented code, and residual unsupported gaps share this chrome.

**Why:** The user asked to soften island chrome itself, not only empty structure. True source-only constructs still need to look like source (monospace, distinct from prose) without inserting a padded card into the document flow. One style avoids a two-tier matrix nobody can remember.

**Alternatives considered:**

- *Two-tier: line island vs block island.* More faithful to multi-line YAML vs one-line junk, but extra branching for little gain if padding is already tight.
- *Leave remaining islands unchanged.* Rejected: a leftover `###` gap or a one-line unsupported row would still look like the old poster.
- *Inline proportional source (rendered slot).* Rejected for remaining islands: they are source, and the code slot is already specified for Visual Edit source islands.

### Data flow (caching / versioning)

Derivation still runs per document version. Empty headings/lists become additional `PreviewBlock` / `VisualBlock` entries inside the same `Arc` caches; they do not recompute on caret, hover, or prefix reveal. Prefix reveal and island restyle are per-frame view concerns (`build_visual_projection`, `visual_block_view`). Incremental parse must treat an empty heading/list region as a real block so it is not a gap that forces unsupported fallback. Edits still go through `MarkdownDocument.text` (typing after `## ` fills the heading payload and the next derive is a normal heading with content).

## Risks / Trade-offs

- **[Caret at prefix end on non-empty headings]** Revealing `## ` when the caret sits at the first title character is a small behavior change. → Mitigation: specified as intentional; title-interior caret still hides the prefix. Tests for `# Hello` with caret in `Hello` keep markers hidden.
- **[Read/preview height]** Empty headings in a long outline-heavy doc add rows that were previously invisible. → Mitigation: required for mode consistency; rows are heading-height, not island-height.
- **[Enter on empty list item]** Existing structural Enter already exits an empty list item. Keeping the item in the model must not break that transition. → Mitigation: reuse current empty-prefix detection (`is_empty_list_marker`); add a regression test.
- **[Enter on empty heading]** Already starts a paragraph without copying the heading prefix. → Mitigation: keep that path; tests for Enter after `## `.
- **[Softened islands look “not source enough”]** YAML/unclosed fences lose the heavy box. → Mitigation: keep monospace + accent + faint fill; they remain visually distinct from headings/lists.
- **[Residual `###` if derivation regresses]** If a future nonempty gate drops headings again, they become light islands instead of heavy ones — still wrong. → Mitigation: model tests assert `VisualBlockKind::Heading` and `source_island.is_none()`.

## Migration Plan

No persistence, settings, or file-format change. Rollback is `git revert`. Update `docs/visual-editing-quality.md` in the same change as the code.

## Open Questions

None. Placeholder height, prefix-on-focus, empty lists, and lighter remaining chrome were decided before this design.
