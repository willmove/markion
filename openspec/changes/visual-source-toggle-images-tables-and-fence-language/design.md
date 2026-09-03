# Design — visual-source-toggle-images-tables-and-fence-language

## Context

Visual Edit already has a collapsible-source family: `visual_collapsible_source_block` (bordered chrome, hover `</>`, outside-click collapse) hosts block math, registered diagrams, and raw-HTML blocks; expand state lives on the tab as `expanded_visual_source_blocks: HashSet<VisualBlockId>` and is pruned when block ids disappear. Payload editors are `VisualEditorField`s carried on `VisualBlock.editor`; every payload edit is one atomic canonical source replacement validated against `document_version` (checked mutation boundary), and Tab traversal / caret routing walk `VisualBlockEditor::fields()`.

Current gaps this change closes (see proposal.md — Why): standalone images have no source affordance (`simplify-visual-image-presentation` removed the three-field editor and left "switch mode" as the only path), GFM tables with proven cell editors have no raw-source view, and fence info strings cannot be edited (`fenced_payload_ranges` already yields `info_range` but nothing edits it).

Data flow (all existing, extended in three places):

```text
document version N
   └─ visual_blocks_shared() — Arc, cached per version          [src/visual.rs]
        VisualBlock {
          kind: Image | Table | CodeBlock,
          editor: Some(Image { payload })                       ← NEW (only when span proven)
                   | Table { cells, source })                   ← source NEW
                   | Code  { opening_fence, payload, info, closing_fence },  ← info NEW
        }
view layer                                                    [src/app/preview.rs]
   Image arm  → visual_image_source_editor → visual_collapsible_source_block
                 forced = preview-image entry for url is Failed/Unavailable
   Table arm  → visual_table_view(grid; read-only while raw expanded)
                 + payload below via visual_collapsible_source_block
   code/diagram header → language field (VisualEditableText on info range,
                 mounted only when the block owns the caret)
tab state (presentation-only)                                  [src/app/state.rs]
   expanded_visual_source_blocks / hovered_visual_source_block
edit path                                                      [src/app/editing.rs, src/lib.rs]
   VisualBlockEdit { document_version, field, range, replacement }
   → checked mutation → one source replacement → version N+1 → re-derive
```

Toggles never touch document text, dirty state, undo, or derived caches; image/table render caches stay content-keyed.

## Goals / Non-Goals

**Goals:**

- Images, tables, and fence info strings join the existing collapsible/payload-editor family with zero new interaction paradigms — one shared chrome, one expand state, one edit path.
- Keep the default presentation of images and tables exactly as today (no new visual noise when unfocused).
- All new payload fields flow through the existing version-checked mutation boundary, IME, undo, and dirty-state paths.

**Non-Goals:** (beyond the proposal's)

- No completion dropdown for the language field; no re-introduction of structured image field controls; no collapse mode for ordinary code blocks; no changes to one-column/ambiguous tables (they keep today's fallback).
- No persistence of expand state across sessions (family behavior).

## Decisions

### 1. Payload fields live on `VisualBlockEditor`, not synthesized in the view

`visual_editor_tab_target`, caret routing, and mutation validation all enumerate `block.editor.fields()`. Synthesizing payload fields in the view layer would fork that machinery and silently break Tab traversal. Concretely:

- `VisualBlockEditor::Image { payload }` — one field (`ImageSource` kind) covering the complete authored span. Name revives the removed variant but holds a single whole-span field, not three structured ones; the dormant `ImageAlt`/`ImageDestination`/`ImageTitle` field kinds stay untouched and unused.
- `VisualBlockEditor::Table { cells, source }` — `source` (`TableSource` kind) covers the whole table block range. `cells` stays exactly as today; a table with no proven cells keeps `editor: None` and today's fallback (per the modified `tables-outline` requirement).
- `VisualBlockEditor::Code` gains `info: VisualEditorField` (`CodeInfo` kind) whose range is the first info-string token, or an **empty range at `opening_fence.end`** when no info string is authored (an empty field range is the insertion point; replacing it inserts the token directly after the fence with no added space).

Rejected: view-layer field synthesis (breaks traversal/validation); making the table payload a block-level special case outside `VisualBlockEditor` (same reason).

### 2. Image span proof is a local byte-scan inside `visual_block_editor`, never a re-parse

`inline_image_at` builds a full pulldown-cmark `Parser` — running it per image block per version derivation would re-parse the document per image (quadratic on image-heavy files). Instead, `visual_block_editor` proves the span from data already in `PreviewBlock::Image` plus cheap local checks on the authored slice: slice starts with `![`, ends with `)`, label/destination bounds resolvable with the existing unescaped-delimiter scan helpers (`authored_image_destination_range`-style logic). If any check fails (reference-style, multiline, malformed), the editor stays `None` and the block keeps today's island/rendered behavior — no guessed ranges.

### 3. Table dual-editor rule: expansion demotes the grid, cell activation collapses

While the table's block id is expanded (or the caret sits in the source payload):

- `visual_table_view` renders read-only rendered cells (no `VisualEditableText` cell editors mounted) and the row/column toolbar is not interactive;
- the raw payload below is the block's only active editor and owns the caret (expanding while a cell owns the caret routes the caret into the payload);
- clicking a cell while expanded toggles the block collapsed and `move_to`s into that cell's current-version range — the round-trip users expect;
- Tab traversal (`visual_editor_tab_target` in `src/lib.rs`) skips cell fields for an expanded table block and stops at the source field, so keyboard focus cannot enter unmounted cell editors.

Rejected: keeping cell editors live alongside the payload (two editors over the same bytes — ambiguous caret ownership and double mutation seams); hiding the grid while expanded (loses the WYSIWYG anchor the family preserves).

### 4. Image forced-expand rides the preview-image cache state

`forced` for the image chrome = the preview-image entry for that destination (resolved against `document_dir`) is in its failed/unavailable state — exactly the math/diagram pattern (`forced = !matches!(entry, Ready)`). A permanently unloadable remote image keeps the source visible; that mirrors the family's "invalid stays editable" contract and is the accepted trade-off (risk below).

### 5. Language field: live sanitized editing on the exact token range

The language field is a `VisualEditableText` bound to the `CodeInfo` range, mounted in the block header only when the fence owns the caret (Read mode, Split Preview, and unfocused headers keep the static label — the header function gains an optional editable element rather than a parallel header). Editing is live like every other field, with a keystroke filter in the field's input path that rejects characters outside a single language token (whitespace, backticks, newlines) — the same live-sanitize pattern the dormant image-field kinds used. An empty commit is not possible: the field rejects input that would empty the token range while mounted; clearing a language remains a payload/source-mode operation. Dispatch after a retyped token needs no special case: version bump → re-derivation → the block presents as ordinary code / diagram / math per the new first token, and stale ids prune the expand set automatically.

### 6. Row re-measurement reuses the family mechanism

Mounting/unmounting a payload child changes the row's height under the virtualized list; math/diagram rows already re-measure through the existing id/`height_signature` machinery shipped with the collapse change. Image and table rows join the same path; no new height invalidation scheme is introduced (covered by a verification task, not new code paths).

### 7. i18n

Reuse the existing source-toggle tooltip. Add: language-field placeholder/aria label (EN + zh-CN) in `src/i18n.rs`. No other new strings.

## Risks / Trade-offs

- **[Table traversal leaks into unmounted cell editors]** → `visual_editor_tab_target` must skip cell fields while the block is expanded; add a Tab-traversal test for the expanded table (cells skipped, payload participates, boundary hand-off intact).
- **[Caret caught in a cell when the user expands]** → expanding routes the caret into the payload (decision 3); regression-test expanding from a focused cell.
- **[Forced-expand noise for permanently unloadable images]** → accepted, mirrors math/diagram errors; the payload is exactly where the user fixes the URL.
- **[Live token editing temporarily breaks dispatch (e.g. mid-way through typing `merm`)]** → ordinary re-parse handles it (`merm` is just an unregistered language → lexer fallback); no fence is ever malformed because the field cannot emit whitespace/backticks/newlines.
- **[Image span proof drifts from the parser's image grammar]** → proof uses the same conservative delimiter-scan helpers as `inline_image_at`; any unprovable form falls back rather than guessing. Property-style tests over escaped labels/titles/angle destinations keep the two in sync.
- **[CRLF documents]** → payload ranges carry `\r\n` verbatim; multi-line payloads already flow through the HTML payload editor path, and single-line fields (image, language token) never contain line endings.

## Migration Plan

No document, preference, or session migration — everything is presentation-only or additive derived state. Rollback is reverting the change. Archive `simplify-visual-image-presentation` before this change so the image requirement set converges in one direction (see proposal ordering note).

## Open Questions

_None blocking._ Deferred candidates already recorded as non-goals: language completion dropdown; raw-source toggle for tables without proven cell editors (one-column/ambiguous); ordinary code-block read-mode collapse.
