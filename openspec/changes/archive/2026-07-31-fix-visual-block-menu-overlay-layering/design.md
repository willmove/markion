## Context

Visual Edit renders its document through a GPUI virtualized `list`. The focused row adds drag and block-operation chrome, and the open `Turn Into` menu is currently an absolutely positioned child of that row. GPUI paints children in tree order; `.occlude()` changes mouse hit-box behavior but does not create a paint layer. As a result, later virtualized rows and their formatted text or images can paint over the menu.

Markion already renders application menus, context menus, the slash palette, link editing, preferences, and recovery UI as later children of the application root. File-tree and preview context menus also use `anchored()` with a window-space pointer position so GPUI can measure the panel and keep it within the viewport. The block menu can reuse that established pattern without changing any block mutation logic.

The relevant presentation flow becomes:

`button pointer-up -> BlockMenuState { BlockTarget, anchor } -> root contextual overlay -> existing validated block command -> one canonical source mutation`

Opening, positioning, scrolling within, or dismissing the menu remains presentation-only. It does not change `MarkdownDocument.text`, document version, selection history, the shared `Arc<Vec<VisualBlock>>`, memoized syntax highlighting, or the cached text handle.

## Goals / Non-Goals

**Goals:**

- Paint the block-operation menu above every Visual Edit document row and media element.
- Keep the menu anchored near the invoking gutter button and keep all commands reachable near viewport edges.
- Preserve current exact `BlockTarget` validation, command serialization, undo, dirty, autosave, recovery, tab-isolation, and cache behavior.
- Give the menu deterministic dismissal and overlay precedence rules.
- Add regression evidence for the overlapping-content scenario shown in the bug report.

**Non-Goals:**

- Changing which blocks can transform, reorder, duplicate, or delete, or changing the Markdown produced by those commands.
- Adding a parallel rich-text/AST document model or persisting contextual UI state.
- Redesigning the slash palette, link editor, application menus, or all overlays into a new general-purpose framework.
- Adding an external UI, snapshot, or image-processing dependency solely for this fix.

## Decisions

### 1. Render the menu in the application root's contextual-overlay stratum

The focused row will continue to render the gutter button and drag/drop affordances, but it will no longer append the menu panel. When `BlockMenuState` is present, the application root will append one block-menu overlay after the main document content and before modal recovery/preferences content. Opening an application-level menu or modal will close the block menu or otherwise keep the modal above it.

This guarantees that no later virtualized document row can paint over the panel and avoids relying on a numeric z-index that GPUI's `Div` style does not expose. A pane-local overlay host was considered, but it would require threading app state and coordinate conversion through `visual_edit_surface_view` while providing no additional layering benefit for this bounded fix. A native popup window was rejected because it would add platform-specific focus and lifecycle behavior for an ordinary contextual menu.

### 2. Store a transient window-space anchor with the validated target

`BlockMenuState` will retain the existing immutable `BlockTarget` and add the invoking button's pointer position (or equivalent button anchor) in window coordinates. The row's mouse-up handler passes that anchor to `open_visual_block_menu`; the root overlay uses `anchored().position(anchor)` with a small visual offset.

Using a window-space anchor matches existing file-tree and preview context menus and lets GPUI measure, flip, and snap the panel inside the viewport. The menu keeps a bounded maximum height and gains internal vertical scrolling so edge clamping never makes lower commands unreachable. Recomputing anchor geometry from virtual-list rows on every frame was rejected because rows can be recycled and it would couple presentation-only menu state to list measurement.

### 3. Close rather than detach when the document viewport moves

Escape, an outside pointer action, a document-pane scroll, tab or view-mode change, document mutation, undo/redo, opening a conflicting overlay, or stale target validation will dismiss the menu. Scrolling inside the menu's own overflow region remains available and does not scroll the document. Commands continue to revalidate version, block identity, and exact source range at execution.

Closing on document scroll avoids leaving a root-level panel visually detached from a recycled or moved row. Keeping the panel open and continuously tracking the row was considered, but it adds measurement coordination without improving the short-lived menu interaction.

### 4. Preserve mouse occlusion separately from paint ordering

The root-level panel keeps an opaque themed background, border, shadow, and `.occlude()` behavior so pointer events do not fall through to document content. Paint correctness comes from the overlay's position in the root tree; `.occlude()` remains only the hit-testing safeguard. Application modals retain higher precedence, while document rows, images, the visual input bridge, and overlay scrollbars remain below the block menu.

### 5. Verify both presentation structure and command behavior

Rendered GPUI coverage will use a multi-block document whose first-row menu geometrically overlaps following headings, formatted prose, and media. Tests will assert that the menu is emitted through the root overlay, remains within viewport bounds, exposes every command through scrolling, blocks underlying pointer targets, and still dispatches the existing exact command with one undo entry. The existing command test remains the semantic guard.

Because the current GPUI test API exposes rendered debug bounds and pointer simulation but not cross-platform pixel snapshots, the implementation will also repeat the reported runtime fixture as a focused visual smoke check. The structural overlay assertion prevents the original row-local paint order from being reintroduced without requiring a new image-test dependency.

## Risks / Trade-offs

- **[A stored anchor can become detached after scrolling or layout changes]** → Close the menu on document scroll, tab/mode change, and relevant layout-invalidating state changes.
- **[Root overlays can conflict with application menus or modals]** → Define one contextual menu at a time, clear the block menu when a conflicting overlay opens, and render modal content after contextual overlays.
- **[A tall command list can still exceed a small window]** → Bound panel height, use `anchored()` for flipping/snapping, and enable internal vertical scrolling.
- **[A behavior-only test can pass while paint order regresses]** → Assert the root overlay composition and overlapping bounds in addition to command dispatch, then retain the reported runtime fixture as visual verification.
- **[Presentation events could accidentally invalidate document caches]** → Keep anchor/menu state exclusively on `MarkionApp`; tests assert unchanged text, version, history, and derived `Arc` identity across open, reposition, scroll-within-menu, and dismiss paths.

## Migration Plan

No persisted data or Markdown migration is required. Implement the state/anchor addition, move the panel builder to the root contextual-overlay path, add dismissal and overflow behavior, and then add the rendered regression coverage. Rollback restores the row-local panel and removes the transient anchor field; saved documents and settings remain compatible.

## Open Questions

None. A root contextual overlay with close-on-document-scroll behavior is the selected bounded solution.

