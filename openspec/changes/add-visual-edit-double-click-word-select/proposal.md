# Proposal: add-visual-edit-double-click-word-select

## Why

Double-clicking the left mouse button in Visual Edit mode does nothing today: the second click is handled exactly like the first, collapsing the caret. Every mainstream Markdown editor (Typora, VS Code, Obsidian) double-click-selects the word under the pointer so the user can immediately format, replace, or copy it. Markion's Visual Edit mode is missing this basic text-selection affordance.

## What Changes

- Add double-click (and further same-spot clicks) word selection to the Visual Edit editable surface (`VisualEditableText`): when a left-button `MouseDownEvent` reports `click_count >= 2`, the maximal same-class character run around the hit display offset (word characters, CJK run, punctuation run, or whitespace run) is selected instead of collapsing the caret.
- Define the word range in **display text** (what the user sees), then map it to a canonical source range through `VisualProjection` segments using innermost edge resolution: hidden Markdown syntax at the selection edges is excluded (so typing over a selected word inside `**bold**` keeps the bold markers), while hidden syntax inside the word (e.g. `bo**ld**` rendering as "bold") stays within the contiguous source selection.
- Double-clicking inside a rendered atom's display glyph (inline math, inline HTML images) selects the atom's authored source range.
- Add a pure character-run helper (`char_run_range`) to `src/text_util.rs` and a pure display→source word-range resolver on `VisualProjection` in `src/visual.rs`, both unit-testable without GPUI.
- Shift+click, drag-select, single-click placement, IME composition handling, and viewport preservation semantics remain unchanged.

**Non-goals:** source-mode (plain text editor) double-click parity; read-only preview pane (Split/Read) double-click selection; triple-click line/paragraph selection; word-wise drag extension after the double-click; dictionary-based CJK word segmentation (a contiguous CJK run between whitespace/punctuation is the selected unit); double-click behavior inside multi-field editors (code/math/HTML/table cell inputs), which are separate input surfaces.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `markdown-editing`: adds a new requirement covering Visual Edit double-click word selection — display-run semantics (including CJK runs), source-accurate mapping with edge-excluded hidden syntax, rendered-atom source selection, and the invariant that pointer selection does not reparse or mutate document state.

## Impact

- **Code:** `src/app/preview.rs` (`VisualEditableText::paint` MouseDown handler gains a `click_count` branch), `src/visual.rs` (new `VisualProjection` word-range resolver), `src/text_util.rs` (new `char_run_range` helper), `src/app/editing.rs` (selection applied via the existing `move_to_visual_editor_target` pathway or a thin sibling).
- **Specs:** `openspec/specs/markdown-editing/spec.md` gains one requirement (delta in this change).
- **Dependencies:** none new; uses existing std char classification and the existing projection/segment machinery.
- **Invariants touched:** pointer-only interaction must not reparse, bump `MarkdownDocument.version()`, or invalidate derived caches (existing requirement); selection state stays in the existing per-tab `selected_range` with affinity cleared; the viewport-preservation rules for in-viewport pointer placement continue to apply unchanged.
