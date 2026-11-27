## Why

On macOS, clicking a Visual Edit block-leading grip intermittently fails to open the block quick-settings menu because GPUI's hardcoded 2 px drag threshold consumes small trackpad/Magic Mouse click jitter, eager mouse-down selection causes intermediate-frame reflows that shift the 22 px grip off the pointer before mouse-up, zero-delta/micro-delta scroll events immediately dismiss newly opened menus, and pre-dispatch IME commits invalidate the captured block target version or identity. Additionally, `MathBlock` rows return early before `visual_block_chrome`, omitting their block-leading grip altogether.

## What Changes

- Add an optional per-element `drag_threshold` on vendored GPUI `Interactivity` / `StatefulInteractiveElement` (defaulting to the existing `2.0 px`), and configure a click-tolerant drag threshold (`6.0 px`) on dual click-and-drag Visual Edit grips (`visual-block-drag-grip` and `visual_table_grip`) while keeping resize handles unchanged.
- Introduce explicit transient grip-press state in `MarkionApp` so unmodified grip `MouseDown` defers single-block selection until click completion (`MouseUp`) or drag start (`on_drag`), preventing mid-click layout collapse from cancelling clicks and resolving completed clicks cleanly even if an external frame update shifts row geometry.
- Ignore zero-delta scroll-phase events (`MayBegin`/`Ended`) and accumulate a small pixel threshold (`4.0 px`) before dismissing an open Visual Edit block or table menu on `ScrollWheelEvent`.
- Recover block-grip clicks when macOS `NSTextInputContext` commits active IME composition immediately before `MouseDown`, unambiguously remapping the clicked block across the single IME change set while continuing to reject genuinely stale or external targets.
- Ignore macOS `modifiers.function` (`NSFunctionKeyMask`, which Fn/Globe and navigation keys can set) when distinguishing ordinary grip clicks from Shift/Cmd/Alt/Ctrl-modified clicks, and respect exclusive block-selection ends in `visual_collapsible_source_block` so selecting a block via its grip does not expand a following collapsible HTML/math/diagram/front-matter source pane.
- Remove the early `return` on `VisualBlockKind::MathBlock` in `visual_block_content_view` so display math blocks receive `visual_block_chrome` and expose their block-leading grip and format symbol.
- Add GPUI regression tests covering jitter clicks, mid-click intermediate-frame layout collapse, zero/micro scroll events, IME pre-dispatch commits, and `MathBlock` grips, plus macOS native verification.

Non-goals: changing the global GPUI drag threshold for resize dividers, modifying Read or Source editing modes, altering canonical Markdown serialization or block-reordering semantics, or introducing a second document representation.

## Capabilities

### Modified Capabilities

- `markdown-editing`: Refine `Block grips SHALL provide optional compact quick settings` so completed ordinary grip clicks reliably open the quick menu despite small pointer jitter, mid-click layout reflow, zero/micro scroll-wheel phase events, or synchronous macOS IME commits, and ensure display math blocks expose the same block-leading grip chrome.

## Impact

- Vendored GPUI element interactivity (`vendor/zed/crates/gpui/src/elements/div.rs`).
- Visual Edit grip rendering, gesture lifecycle, scroll dismissal, and IME commit bookkeeping in the root app crate (`src/app/mod.rs`, `src/app/application.rs`, `src/app/editor_element.rs`, `src/app/editing.rs`, `src/app/structured_editing.rs`, `src/app/preview.rs`, `src/app/root_view.rs`).
- Preserves all architecture invariants: no reparsing or derived Markdown cache invalidation during pointer/menu presentation, versioned `BlockTarget` validation remains authoritative for edits and stale rejections, and all GPUI types stay in the root crate.
