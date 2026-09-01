## Context

Visual Edit already materializes every source-backed blank range as `VisualBlockKind::Whitespace` so offsets stay covered. That model is correct. Interaction and paint then treat those rows as CSS gaps:

```
  MarkdownDocument.text / version
            │
            ▼
  cached visual blocks (Whitespace ranges, height_signature = newline count)
            │
            ├── render: 12px strip, no I-beam / mouse-down unless it owns the caret
            └── Up/Down: skip consecutive Whitespace until a non-gap block (or EOF)
```

Two archived decisions produced the current dead zone:

- `prevent-visual-edit-gap-activation` (2026-07-17) made unfocused gaps ignore pointer input so heading-to-heading spacing would not become an accidental typing surface.
- A later navigation change made Up/Down skip those same rows because a 12px empty strip plus a thin caret looked like the arrow key did nothing.

`markdown-editing` now says both “clicking a passive gap does not activate editing” and “a blank line remains reachable by clicking it”. Implementation follows the first sentence. The only remaining entry is structural Enter from an adjacent block, which inserts a newline rather than parking on the one that is already there.

Canonical storage is still `MarkdownDocument.text`. CommonMark has no empty-paragraph node; the blank line is the `\n` itself. This design keeps that byte range as `Whitespace` and makes the row behave like an empty paragraph in the visual surface.

```
  today                                      this change
  ─────────                                  ───────────
  Heading + mt/mb                           Heading + mt/mb
  12px dead gap                             body-height empty line (click / arrows)
  Heading + mt/mb                           Heading + mt/mb
```

## Goals / Non-Goals

**Goals:**

- Unfocused and focused `Whitespace` rows occupy rendered body paragraph line height, one painted line per covered newline (existing floor of one line and pathological cap stay).
- Clicking a blank-line row moves the caret onto an existing offset inside that range. Document text, version, dirty state, undo, and derived caches are unchanged.
- Up/Down (and Select Up/Down) stop on a blank-line row. A further press continues into the block on the far side. `preferred_x` is retained.
- Focused whitespace still paints a thin paragraph-like caret, never a source-island box.
- Typing after click or arrow insert uses the existing source-backed input path at that caret.

**Non-Goals:**

- Clicking must not insert a newline (no Notion-style “click gap → create paragraph”).
- Source mode, Read mode, and Split Preview compact spacing.
- Empty ATX headings / empty list items.
- Inventing an empty-paragraph preview block kind or collapsing extra blank lines on save.
- Folding heading `mt_2`/`mb_2` into the whitespace row (heading chrome may still steal clicks in the margin band).
- New quote-prefix structural rules.

## Decisions

### 1. Keep `Whitespace` as the model; change paint and hit-testing

Do not add `PreviewBlock::Paragraph` for empty payloads and do not drop gap rows. Coverage, incremental splice (`height_signature` = newline count), and Enter-created insertion lines already depend on these rows.

Rejected: mapping extra blank lines to empty paragraphs in the parser. That would be a second document model and would lose 1-vs-N blank-line fidelity.

### 2. Height is `paragraph_line_height`, not 12px and not `paragraph_spacing`

`WHITESPACE_ROW_LINE_HEIGHT` (12px) is replaced by `DocumentTypographyMetrics.paragraph_line_height` (24px at the default rendered size, scaled with the rendered-document font). Quote-context whitespace uses the same body metric for this change (not `quote_line_height`) so heading/changelog gaps and quoted gaps share one empty-line rhythm.

`paragraph_spacing` remains a bottom gap after **paragraph** blocks only. It MUST NOT be added to Whitespace height and MUST NOT rewrite source. Authored blank lines and the paragraph-spacing preference stay independent, matching `document-typography`.

`visual_block_splice` keeps comparing newline-count signatures mapped through the live px function so growing/shrinking gaps still remasure. Typography preference changes already remasure virtualized rows without bumping document version.

### 3. Unfocused rows are hit-testable empty paragraphs

Today only `owns_caret` whitespace builds `VisualEditableText` (I-beam, Y→source, caret). Unfocused rows are a bare `div` with `debug_selector("visual-whitespace-gap")` and no pointer handler.

Always build the whitespace editor surface: I-beam, Y mapping, same row height. Paint the caret only when that row owns the caret (`caret_active`). Click and drag still go through `move_to` / `select_to`.

The `visual-whitespace-gap` test hook stays on the row so GPUI tests can click it.

### 4. Landing offset is inside the range; never the next block’s first content byte

Click and vertical navigation MUST place the caret on an existing blank-line offset, typically the start of the targeted newline inside `source_range`, **not** `source_range.end` when that equals the following block’s start.

For the usual single-newline gap (`## A\n\n## B` → Whitespace covering the separator `\n`), the landing offset is `source_range.start`. Typing there becomes a paragraph between the headings without consuming the next heading’s first character and without inserting an extra `\n`.

`whitespace_source_at_line` currently returns the offset **after** each newline (`start + index + 1`), which for a one-newline gap is `source_range.end` — the next block. Adjust it (and `whitespace_source_at_y`) so line *i* maps to the *i*-th newline’s own offset, clamped to `[source_range.start, source_range.end)` at EOF-exclusive ranges. Trailing document-edge whitespace may use `end` when there is no following content block.

Vertical navigation restores the symmetric `source_range.start` landing from `fix-visual-edit-vertical-gap-navigation` for a one-line gap. Multi-line gaps: Down/Up that enter the row land on the near-side line (first line when entering from above, last line when entering from below); further vertical moves walk painted lines inside the row, then leave to the next non-whitespace block.

Remove the skip-consecutive-Whitespace loop in `move_visual_vertical`.

Rejected: inserting `\n` on click so “there is a paragraph to type in”. The bytes already exist.

### 5. Interaction-only until the user types

```
  click / Up / Down
       │
       ▼
  selection + caret ownership   ← version, Arc caches, dirty, undo unchanged
            │
  user types / IME / Enter
            │
            ▼
  canonical replace_text_in_range (existing path)
```

Caret ownership is derived from the current selection vs `block.source_range` at render time (same as today). No activation flag on cached visual blocks.

### 6. Coverage class stays rendered WYSIWYG

Blank lines were already rendered WYSIWYG. This change does not close a roadmap gap. Update the matrix row in `docs/visual-editing-quality.md` so it no longer says “passive visual row”.

## Risks / Trade-offs

- [Heading `mt_2`/`mb_2` still occupy part of the visual gap] → The clickable empty line is the Whitespace row at body height (~24px), not the heading margin band. Clicking the margin still focuses the heading. Accept for this change; do not reflow heading margins.
- [Documents with many heading separators grow taller] → Intentional: empty lines are empty lines. Read/Preview stay compact.
- [Parking on a blank line used to look like a no-op] → Body line height plus a visible caret is the mitigation; skipping is no longer the mitigation.
- [Quote-structural `Whitespace` (`>`-only rows)] → Same landing rules; typing uses existing source-backed input. Do not invent quote-specific click insertions here.
- [Multi-line Y mapping off-by-one glues text to the next heading] → Decision 4; regression tests for heading-to-heading click then type.
- [Virtual list stale heights] → Keep newline-count `height_signature`; map through the typography-aware px helper; typography changes remasure.

## Migration Plan

No data migration. Rollback is restoring passive unfocused gaps, the 12px constant, and the skip-gap loop. Tests that currently assert passive clicks and skip-gap arrows invert to enter-and-type.

## Open Questions

None. Heading-margin hit stealing is a documented leftover, not a blocker.
