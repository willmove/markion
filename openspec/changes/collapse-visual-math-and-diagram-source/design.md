## Context

Visual Edit already has direct payload editors for exactly ranged block math (`visual_math_editor`) and registered diagram fences (`visual_diagram_editor`). Both always render “presentation above, monospaced payload below.” That dual pane is correct for editing but noisy when the user is only reading. Obsidian-style collapse—render-only by default, hover `</>` to expand source, click outside to collapse—matches the product direction after image-field simplification and the WYSIWYG framing work.

Constraints to preserve:

- `MarkdownDocument.text` remains canonical; expand/collapse is presentation-only.
- Math and diagram `Arc` derivation caches stay version-keyed and must not invalidate on toggle.
- Math/diagram render caches stay content-keyed presentation data.
- Payload edits remain one atomic source replacement with existing IME/undo paths.

```text
pointer hover / </> click / outside click
        │
        ▼
EditorTab.expanded_visual_source_blocks: HashSet<VisualBlockId>
        │  (cleared or pruned when VisualBlockId set changes with document version)
        ▼
visual_math_editor / visual_diagram_editor
   collapsed → render (or pending/error chrome) + hover </>
   expanded  → render + payload editor  (today’s dual pane)
   forced    → expanded when pending/error (no collapse until ready+valid)
```

## Goals / Non-Goals

**Goals:**

- Default Visual Edit presentation for exact block math and registered diagrams is render-only.
- Hover shows a top-right source-toggle control; only that control expands source.
- Expanded state shows the existing dual pane; click outside the block collapses it.
- Pending/invalid results keep the payload editor available.
- Math and diagrams share one expand/collapse state machine.

**Non-Goals:**

- Inline math toggle UI; ordinary (non-diagram) code-block collapse; Split Preview/Read changes; formula/diagram tree editing; persisting expand state across sessions.

## Decisions

### 1. Presentation state lives on `EditorTab`, keyed by `VisualBlockId`

Store `expanded_visual_source_blocks: HashSet<VisualBlockId>` (name flexible) on the tab. Toggle adds/removes the block id. On visual-list sync, drop ids that no longer exist in the current `visual_list_blocks`. Do not key by document version alone—stable ids already invalidate when the block’s identity changes.

Rejected: storing expand flags inside derived `VisualBlock` / `Arc` caches (would force re-derivation or break sharing). Rejected: persisting to session/preferences (ephemeral UX is enough).

### 2. One shared chrome helper for math and diagrams

Extract a small wrapper (e.g. `visual_collapsible_source_block`) that owns:

- bordered container;
- hover-visible `</>` control (top-right);
- collapsed vs expanded child slots;
- click handling that distinguishes toggle control vs outside.

`visual_math_editor` and `visual_diagram_editor` supply the presentation child and the payload-editor child. Keeps interaction identical and avoids two hover implementations.

### 3. Expand only via `</>`; collapse via outside click

- Clicking the rendered formula/diagram (or its scroll area) does not expand source. Existing selection/hit-testing behavior for math atoms and diagram images stays as today where applicable.
- Clicking `</>` toggles expanded for a ready+valid block (or expands when collapsed).
- A primary click whose hit target is outside the block’s bounds collapses that block’s expanded entry, unless the click is on the `</>` of another block (that other block expands; previous collapses as “outside”).
- While the source caret owns the payload field, keep expanded even if hover ends; an outside click still collapses and should move focus/selection per existing visual click routing (do not invent a second focus model).

Rejected: second click on `</>` as the only collapse path (user chose outside click). Rejected: clicking the formula to expand (user chose `</>` only).

### 4. Forced expand for pending and error

When the math/diagram cache entry is `Pending` or `Error`, or block-math validation reports invalid LaTeX, present the payload editor regardless of the HashSet (treat as forced expanded). The `</>` control MAY remain visible but MUST NOT hide the payload until the result is ready and valid. This preserves the current “invalid remains editable” guarantee.

### 5. i18n and affordance glyph

Use a compact `</>` (or equivalent) glyph with a localized tooltip/aria label via `src/i18n.rs` (e.g. “Edit source” / 「编辑源码」). Prefer a text glyph over a new icon asset unless an existing icon set is already wired for Visual Edit.

## Risks / Trade-offs

- **[Outside-click races with payload focus]** → Collapse only when the mouse-down/up target is outside the block hitbox; if the caret still maps into the payload after the click, keep expanded (defensive). Cover with an integration test: type in payload, click elsewhere, assert collapsed and caret left the payload.
- **[Hover flicker on child boundaries]** → Hover the outer chrome container, not only the formula image; keep the toggle in the same hovered ancestor.
- **[List virtualization drops hover state]** → Expanded set is on the tab, not the row widget; recycling rows cannot lose expand intent. Hover-only chrome may disappear when scrolled off—acceptable.
- **[Stable id churn collapses unexpectedly after edit]** → Expected: payload edit that changes block identity clears expand; forced-expand path covers invalid intermediate states.
- **[Mermaid + math slightly different chrome today]** → Shared wrapper may nudge diagram border/padding; keep visual parity with current dual-pane chrome as the expanded look.

## Migration Plan

No document or preference migration. After ship, Visual Edit simply defaults to collapsed; users discover `</>` on hover. Rollback is reverting the change; no persisted expand flags.

## Open Questions

_None blocking._ Optional polish after v1: keyboard shortcut to toggle source on the focused block; match Obsidian’s exact iconography.
