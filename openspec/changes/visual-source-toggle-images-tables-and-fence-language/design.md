# Design — visual-source-toggle-images-tables-and-fence-language

## Context

Visual Edit already has a collapsible-source family: `visual_collapsible_source_block` (bordered chrome, hover `</>`, outside-click collapse) hosts block math, registered diagrams, and raw-HTML blocks; expand state lives on the tab as `expanded_visual_source_blocks: HashSet<VisualBlockId>` and is pruned when block ids disappear. Payload editors are `VisualEditorField`s carried on `VisualBlock.editor`; every payload edit is one atomic canonical source replacement validated against `document_version` (checked mutation boundary), and Tab traversal / caret routing walk `VisualBlockEditor::fields()`.

Current gaps this change closes (see proposal.md — Why): standalone images have no source affordance (`simplify-visual-image-presentation` removed the three-field editor and left "switch mode" as the only path), and fence info strings cannot be edited (`fenced_payload_ranges` already yields `info_range` but nothing edits it). GFM tables are intentionally untouched here — the always-editable grid stays; an earlier dual-editor attempt inside this change was reverted after user testing.

Data flow (all existing, extended in two places):

```text
document version N
   └─ visual_blocks_shared() — Arc, cached per version          [src/visual.rs]
        VisualBlock {
          kind: Image | CodeBlock,
          editor: Some(Image { payload })                       ← NEW (only when span proven)
                   | Code  { opening_fence, payload, info, closing_fence },  ← info NEW
        }
view layer                                                    [src/app/preview.rs]
   Image arm  → visual_image_source_editor → visual_collapsible_source_block
                 (payload_first = true: source above the image)
                 forced = preview-image entry for url is Failed/Unavailable
   code/diagram header → language chip (VisualEditableText on the info range,
                 revealed while the fence is hovered or owns the caret)
tab state (presentation-only)                                  [src/app/state.rs]
   expanded_visual_source_blocks / hovered_visual_source_block /
   hovered_visual_code_block
edit path                                                      [src/app/editing.rs, src/lib.rs]
   VisualBlockEdit { document_version, field, range, replacement }
   → checked mutation → one source replacement → version N+1 → re-derive
```

Toggles and hover never touch document text, dirty state, undo, or derived caches; image render caches stay content-keyed.

## Goals / Non-Goals

**Goals:**

- Images and fence info strings join the existing collapsible/payload-editor family with zero new interaction paradigms — one shared chrome, one expand state, one edit path.
- Keep the default presentation of images exactly as today (no new visual noise when unfocused), and keep GFM tables exactly as they are.
- All new payload fields flow through the existing version-checked mutation boundary, IME, undo, and dirty-state paths.

**Non-Goals:** (beyond the proposal's)

- No completion dropdown for the language field; no re-introduction of structured image field controls; no collapse mode for ordinary code blocks; no GFM table changes (reverted after user testing — the always-editable grid stays).
- No persistence of expand state across sessions (family behavior).

## Decisions

### 1. Payload fields live on `VisualBlockEditor`, not synthesized in the view

`visual_editor_tab_target`, caret routing, and mutation validation all enumerate `block.editor.fields()`. Synthesizing payload fields in the view layer would fork that machinery and silently break Tab traversal. Concretely:

- `VisualBlockEditor::Image { payload }` — one field (`ImageSource` kind) covering the complete authored span. Name revives the removed variant but holds a single whole-span field, not three structured ones; the dormant `ImageAlt`/`ImageDestination`/`ImageTitle` field kinds stay untouched and unused.
- `VisualBlockEditor::Code` gains `info: VisualEditorField` (`CodeInfo` kind) whose range is the first info-string token, or an **empty range at `opening_fence.end`** when no info string is authored (an empty field range is the insertion point; replacing it inserts the token directly after the fence with no added space).

Rejected: view-layer field synthesis (breaks traversal/validation).

### 2. Image span proof is a local byte-scan inside `visual_block_editor`, never a re-parse

`inline_image_at` builds a full pulldown-cmark `Parser` — running it per image block per version derivation would re-parse the document per image (quadratic on image-heavy files). Instead, `visual_block_editor` proves the span from data already in `PreviewBlock::Image` plus cheap local checks on the authored slice: slice starts with `![`, ends with an unescaped `)`, no unescaped `)` inside a non-angle destination, label/destination bounds resolvable with the existing unescaped-delimiter scan helpers (`authored_image_destination_range`-style logic). If any check fails (reference-style, multiline, malformed), the editor stays `None` and the block keeps today's island/rendered behavior — no guessed ranges.

### 3. Image source renders above the image via a `payload_first` slot on the shared chrome

User testing found the source-under-image layout awkward: after clicking `</>` the authored span should sit directly under the toggle, above the presentation. `visual_collapsible_source_block` gains a `payload_first: bool` (image = true; math/diagram/HTML = false) that flips the child order and keeps the toggle at the top-right. The image payload reserves a top-padding strip so the toggle never covers source text. Rejected: a separate chrome variant for images (diverges interaction behavior); moving the toggle to the bottom (overlaps the caret-owner image controls).

### 4. Image forced-expand rides the preview-image cache state

`forced` for the image chrome = the preview-image entry for that destination (resolved against `document_dir`) is in its failed/unavailable state — exactly the math/diagram pattern (`forced = !matches!(entry, Ready)`). A permanently unloadable remote image keeps the source visible; that mirrors the family's "invalid stays editable" contract and is the accepted trade-off (risk below).

### 5. Language field: hover/caret-revealed chip, not a persistent editable header label

User testing found the caret-gated header label hard to notice and click. The label is now a distinct chip (border, min width, I-beam cursor) that is hidden while the pointer is outside the fence and the fence does not own the caret; hovering the fence or placing the caret in it reveals the chip over the first info token. A bare fence shows the localized `CodeLanguage` placeholder as the chip's content and the whole chip moves the caret into the empty insertion range on click; a non-empty token renders a `VisualEditableText` bound to the `CodeInfo` range so clicking places the caret precisely. Hover state lives in `tab.hovered_visual_code_block` (set by the fence's `on_hover`, pruned/cleared with the other hover state). Editing stays live with the existing keystroke sanitizer (whitespace, backticks, newlines rejected); Enter in the field commits instead of inserting a newline. Dispatch after a retyped token needs no special case: version bump → re-derivation → the block presents as ordinary code / diagram / math per the new first token. Read mode and Split Preview keep the static label; diagram fences mount the same chip whenever their payload editor is visible. Rejected: an autocomplete dropdown (non-goal); a persistent always-editable header label (noise + missed clicks).

### 6. Row re-measurement and payload-editor plumbing reuse the family mechanism

Mounting/unmounting a payload child changes the row's height under the virtualized list; math/diagram rows already re-measure through the existing id/`height_signature` machinery shipped with the collapse change. Image rows join the same path; no new height invalidation scheme is introduced (covered by a verification task, not new code paths). The `Code` editor's extra info field must not disturb caret movement: `visual_editor_edge_target` filters `Code` editors to their payload field only, so stepping off the payload's closing edge still hands to the next block instead of the fence header.

### 7. i18n

Reuse the existing source-toggle tooltip. Add: `Msg::CodeLanguage` placeholder label (all seven languages) in `src/i18n.rs`. No other new strings.

## Risks / Trade-offs

- **[Forced-expand noise for permanently unloadable images]** → accepted, mirrors math/diagram errors; the payload is exactly where the user fixes the URL.
- **[Live token editing temporarily breaks dispatch (e.g. mid-way through typing `merm`)]** → ordinary re-parse handles it (`merm` is just an unregistered language → lexer fallback); no fence is ever malformed because the field cannot emit whitespace/backticks/newlines.
- **[Image span proof drifts from the parser's image grammar]** → proof uses the same conservative delimiter-scan helpers as `inline_image_at`; any unprovable form falls back rather than guessing. Property-style tests over escaped labels/titles/angle destinations keep the two in sync.
- **[CRLF documents]** → payload ranges carry `\r\n` verbatim; multi-line payloads already flow through the HTML payload editor path, and single-line fields (image, language token) never contain line endings.
- **[Chip reveal churn while the pointer crosses the fence boundary]** → reveal is hover-set membership on the tab, like the existing table/source hover state; leaving the fence clears it, and a caret-owned fence keeps the chip stable while typing.

## Migration Plan

No document, preference, or session migration — everything is presentation-only or additive derived state. Rollback is reverting the change. Archive `simplify-visual-image-presentation` before this change so the image requirement set converges in one direction (see proposal ordering note). GFM tables were intentionally left untouched (an earlier dual-editor attempt in this change was reverted after user testing).

## Open Questions

_None blocking._ Deferred candidates already recorded as non-goals: language completion dropdown; a GFM table raw-source toggle (reverted after testing — the always-editable grid stays); ordinary code-block read-mode collapse.
