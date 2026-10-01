## Context

Visual Edit builds rows from the per-version preview parse. Today's pipeline and the two failure points:

```
MarkdownDocument.text (version N)
 └─ lib.rs block parse (cached per version, shared via Arc)
     └─ PreviewBlock::BlockQuote { children, source_range }        ← Tag::BlockQuote(kind): kind discarded
         └─ visual.rs build_visual_blocks (cached per version, Arc)
             ├─ expand quote leaves into rows (each leaf = paragraph/list child)
             ├─ uncovered bytes between rows → gap_block()
             │    ├─ structural-only (`>`, blank) → Whitespace row with quote context
             │    └─ anything else → Unsupported source island   ← BUG 1: `> [!NOTE]` lands here
             │       (pulldown-cmark consumes the GFM alert marker as block structure, so no
             │        inline event ever owns those bytes; quote_gap_is_structural_only sees
             │        leftover `[!NOTE]` text and returns false)
             └─ visual_block_from_preview → inline_runs(slice)
                  └─ slice strips only the FIRST line's `> ` prefix                ← BUG 2
                     later lines' `> ` remain; CommonMark lets `>` interrupt the
                     paragraph → two paragraphs, no SoftBreak event → projection
                     drops the unowned `\n> ` gap → lines render merged
                      └─ build_visual_projection (per render, cursor-dependent reveal)
                          └─ app/preview.rs visual_block_view (kind arms + quote decoration)
```

Reveal mechanism reused by this design: `VisualQuoteContext.marker_ranges` are hidden by default and revealed as verbatim source when the caret/selection endpoint is inside them (`build_visual_projection`), which is how `>` prefixes behave today.

## Goals / Non-Goals

**Goals:**
- Alert marker bytes get exactly one visual owner that renders as a callout title row and reveals verbatim on focus.
- Lazy-continuation quoted paragraphs keep authored line breaks with byte-exact mapping.
- No new caching or parse layers: the alert kind rides the existing per-version preview cache; soft-break runs are built inside the existing visual-block build.

**Non-Goals (design-level):**
- No preview/Read/export alert styling; the parser still omits the `[!NOTE]` label there (unchanged behavior).
- No nested-quote (depth ≥ 2) alert handling; alert is captured at depth 1 only, matching current flattening.
- No new insertion commands, no localization of the alert label (canonical GFM English label, like list bullet glyphs — document content, not app chrome).

## Decisions

### D1: Thread the alert kind through the domain model
`Event::Start(Tag::BlockQuote(kind))` at quote depth 1 maps to a model-owned `AlertKind { Note, Tip, Important, Warning, Caution }` stored as `PreviewBlock::BlockQuote { children, source_range, alert }`. `push_nonempty_block` (parse.rs) keeps the block when `children.is_empty() && alert.is_some()` so body-less alerts survive into the model instead of being dropped.

Alternative rejected: re-recognizing `> [!TYPE]` text in the visual gap classifier. It duplicates pulldown-cmark's GFM recognition rules (valid types, casing, marker position, what counts as trailing content) and can silently diverge from the parser that decides which bytes are content in the first place.

### D2: Callout title row as a first-class visual block
New `VisualBlockKind::CalloutTitle { kind: AlertKind }` built from the leading gap of an alert quote group (or the whole group when body-less):

- **Ownership**: exactly the marker-line bytes including the trailing newline — the range `gap_block` receives today. No overlap with the first leaf (the existing `covered_until` accounting and the overlap debug_assert stay intact).
- **Content**: no `editable_runs`. The `> ` prefix and the `[!NOTE]` bytes both go into `quote_context.marker_ranges`, so the existing projection reveal shows `> [!NOTE]` verbatim when the caret is inside, and nothing (label only) otherwise. The row reuses the marker-only-row caret machinery that `Whitespace` quote rows already use.
- **Group membership**: carries the quote group's `group_source_range`, so `assign_quote_group_edges` makes it the group's `First` (or `Only`) edge and it inherits the quote bar/indent decoration automatically.
- **View**: a `CalloutTitle` arm in `visual_block_view` renders a bold label ("Note", "Tip", "Important", "Warning", "Caution") with a per-kind accent color (blue/green/purple/orange/red) drawn from the app palette, degrading to the existing quote gray when a palette color is absent.

Chosen over putting `[!NOTE]` into `editable_runs`: keeping the marker structural matches how `>` prefixes behave (hidden until focus) and satisfies the monotonic-mapping constraints with machinery that already exists; an always-visible `[!NOTE]` run would render authored syntax unfocused, contrary to the progressive-reveal contract.

### D5: Keyboard navigation into the title row (resolved during implementation)
Title rows carry no rendered runs, so the layout snapshot that `complete_pending_visual_navigation` (src/app/editing.rs) uses to place the caret never exists for them; Up from the body silently parked the caret on the body row instead. Rather than the design's fallback of a conservative verbatim run (which would demote the row back to a source island), navigation now falls back to the marker line's end: when the pending navigation target is a `CalloutTitle` block without a snapshot, the caret lands just inside the line end, which lies in the row's marker range so the existing reveal projection shows `> [!NOTE]` verbatim. Down from the title row exits through the ordinary text-line path. Pinned by an app-level gpui test.

### D3: Synthesize soft-break runs for quoted leaves
After `inline_runs`, when the row has a quote context, compute unowned gaps (same logic as `marker_ranges()`) and for every gap that contains a `\n`, insert one synthetic `VisualInlineRun { visible_text: "\n", content_range: <that single newline byte>, style: default }` in sorted position. The newline byte becomes owned and always rendered; the remaining `> ` bytes stay marker-revealed. Effects:

- `> a` / `> b` renders as two lines; focused reveal shows `\n> ` exactly as authored (newline run + revealed marker piece in source order).
- Hard-break spacing (`> a␣␣` / `> b`) still yields a break: the rule picks the first newline in the gap; the unowned spaces stay hidden.
- Scope guard: synthesis runs only for quote-context rows. Unquoted paragraphs already receive real `SoftBreak` events from pulldown-cmark, so no other block kind changes behavior.

Alternative rejected: parsing a marker-stripped owned input and remapping every event range back through the strip map — invasive offset remapping for the same visible result.

### D4: Unknown markers stay literal
`[!CUSTOM]` and friends are not alerts to the parser, so they remain paragraph text; D3 alone gives them their own line. Matches GitHub.

## Risks / Trade-offs

- [Title row caret placement depends on marker-only-row machinery] → The layout-snapshot gap is closed by D5's navigation fallback; the reveal scenario in the specs holds. Verified by gpui integration tests (keyboard entry, verbatim reveal, source-backed typing).
- [pulldown-cmark's exact recognition of `> [!NOTE] trailing`] → Pin actual behavior with a unit test during implementation; if the parser does not treat it as an alert, it renders as literal text (acceptable, GitHub-consistent).
- [Exhaustive matches over `VisualBlockKind` across document_memory / source_mapped / block_edit / editing] → Compiler-driven updates; title row carries quote context so existing quote-group reordering exclusions apply (no guessed moves).
- [Accent colors per theme] → Degrade to the quote gray when the palette lacks a dedicated color; colors are additive and theme-scoped.

## Migration Plan

Additive model change: one new enum, one new `PreviewBlock` field (single construction site in lib.rs), one new visual block kind. No persistence, file-format, or preference migration. Derived caches are per document version, so open documents pick up the new blocks on next parse; rollback is reverting the commit.

## Open Questions

- Exact accent color values per alert kind for light/dark palettes — resolve against `app.palette()` while implementing the view arm; fallback color is specified above.
- Whether mouse click into the title row lands before its first marker byte or inside `[!NOTE]` — depends on the existing whitespace-row pointer mapping; either is spec-compliant, test pins the behavior.
