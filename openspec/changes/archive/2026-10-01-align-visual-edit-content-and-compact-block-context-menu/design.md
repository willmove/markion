## Context

Visual Edit renders a virtualized `list` of `VisualBlock` rows. For every block accepted by `block_can_transform_at`, `visual_block_chrome` currently wraps the rendered block in a horizontal flex row whose leading child reserves 48 pixels when reordering is available and 28 pixels otherwise. The controls inside that child are conditional on caret ownership, but the reserved width is unconditional. Transformable prose therefore starts and wraps farther to the right than both Read mode and non-transformable Visual Edit media.

The completed block-menu layering change already moved the menu panel into the application root, stores an ephemeral `BlockTarget` plus a window-space anchor, constrains the menu to the viewport, and centralizes presentation-only dismissal. This change should reuse that overlay seam and every existing exact transform/reorder command rather than replace them.

The relevant interaction flow becomes:

`row right-click or keyboard context action -> BlockMenuState { BlockTarget, anchor, submenu } -> compact root overlay -> existing validated command -> one canonical source mutation`

The hover/focus drag flow remains:

`flow-neutral absolute grip -> existing DraggedVisualBlock -> existing validated reorder command -> one canonical source mutation`

Opening, navigating, or dismissing the menu and showing or hiding the drag grip remain ephemeral presentation state. They do not alter `MarkdownDocument.text`, document version, selection, history, dirty state, per-version derived `Arc` caches, memoized highlighting, or cached text handles.

## Goals / Non-Goals

**Goals:**

- Give equivalent top-level Visual Edit and Read-mode content a common document axis and available width.
- Keep block chrome out of normal flow so hover, focus, and menu availability never change content measurement or wrapping.
- Make block operations available from a compact pointer context menu and an operable keyboard context-menu path.
- Preserve exact `BlockTarget` validation, one-edit/one-undo semantics, drag reordering, overlay precedence, localization, and cache invariants.
- Define deterministic selection and specialized-child event precedence and add rendered regression evidence.

**Non-Goals:**

- Changing block transformation, serialization, duplicate, delete, or reorder semantics.
- Adding a rich-text/AST document model or persisting contextual UI state.
- Redesigning the selection-formatting toolbar, link editor, table toolbar, or Read/Split preview context menu.
- Changing the Visual Edit support class or fallback behavior of any Markdown construct.
- Introducing touch-specific long-press interaction or a new UI dependency.

## Decisions

### 1. Compose block chrome as an out-of-flow sibling

Each eligible Visual Edit row remains a relative, full-width container whose content child receives the complete document-column width. The drag grip is rendered as an absolute sibling in the existing leading pane/document padding and appears only while that block is hovered, focused, or actively dragged. The menu ellipsis button and the fixed 28/48-pixel flex spacer are removed.

The grip's hitbox may be clamped within the Visual Edit surface on narrow windows, but it MUST NOT participate in the content line box, reduce the content width, or change row height. Lists, blockquotes, source islands, tables, image fields, and other constructs keep their intentional semantic or component-internal indentation.

Keeping the existing flex gutter with zero-opacity controls was rejected because it preserves the layout defect. Removing drag reordering entirely was rejected because Move Up/Down is a slower substitute and the stable spec already requires a drag path.

### 2. Resolve the block target at the invocation source

A right-button event on an eligible row constructs the same immutable `BlockTarget` used by the current ellipsis button and passes the pointer's window-space position to `open_visual_block_menu`. The event does not move or collapse the current text selection merely to open the menu. A more specific child interaction may consume the event first; otherwise it bubbles to the generic row-level block menu. Read/Split preview continues using `PreviewContextMenu` and is not merged with the editable block menu.

A new Visual Edit context-menu action provides the keyboard path. It resolves the caret-owning transformable block and anchors the overlay near the latest painted `visual_caret_bounds`; if exact caret geometry is unavailable, it uses the Visual Edit input/surface bounds as a bounded fallback. Unsupported, ambiguous, or stale ownership does not open an actionable menu.

Moving the caret on every right-click was rejected because it destroys a non-empty selection before the user has chosen an operation. Reusing the preview context-menu state was rejected because preview actions are non-editable selection/copy operations with different targeting and lifecycle rules.

### 3. Use a compact root menu with two shallow transform submenus

The root block menu uses the existing root contextual-overlay stratum, opaque chrome, hit-test occlusion, viewport anchoring, and local overflow scrolling. Its root contains:

1. a localized Text and Headings submenu containing Text and Heading 1 through Heading 6;
2. a localized Lists submenu containing Bulleted, Numbered, and Task List;
3. direct Quote, Code Block, Divider, and Table transforms;
4. a separator followed by Duplicate, Move Up, and Move Down;
5. a final separator followed by a visually destructive Delete action.

The current block type is checked or otherwise identified in the transform choices. Unavailable moves are disabled or omitted without changing command semantics. Submenus open beside the parent panel, use the same viewport clamping, and remain no deeper than one submenu level. Keyboard navigation supports Up/Down between enabled items, Right or Enter to open/confirm, Left to return from a submenu, and Escape to close.

Retaining all transforms and operations as one vertical list was rejected because it creates an unnecessarily tall menu and forces scrolling in ordinary windows. A dense unlabeled icon grid was rejected because heading/list distinctions and destructive actions need accessible localized labels.

### 4. Preserve selection and contextual-overlay precedence

Right-click inside or outside an existing Visual Edit selection leaves the exact canonical selection unchanged until the user invokes a command. While the block menu is open, its root overlay takes presentation precedence over the selection toolbar; closing the block menu allows normal selection-derived controls to appear again. Existing link, table, media, source-island, and drag handlers may stop propagation when they own a more specific interaction, preventing a second menu from replacing the intended UI.

Opening another application menu, modal, link editor, slash palette, tab, or view mode keeps the existing one-contextual-overlay dismissal rules. Document scrolling still closes the block menu, while scrolling within a menu or submenu does not move the document.

Combining selection-format commands into this change was rejected because it would broaden the work from block presentation and targeting into a separate inline-formatting redesign.

### 5. Verify geometry, targeting, keyboard behavior, and invariants

Rendered GPUI tests will compare debug bounds for equivalent top-level Visual Edit content with the shared document column and with non-transformable media, then prove that hover/focus chrome does not change content bounds, row height, or wrapped-line layout. Interaction fixtures will open the menu on a non-caret block by right-click, open it for the caret-owning block by keyboard, navigate both submenus, apply an exact transform and reorder, and verify one-step undo.

Lifecycle tests will assert that right-click, submenu navigation, outside dismissal, Escape, and keyboard opening preserve source, version, selection, history, dirty state, and derived `Arc` identity until an operation is confirmed. Existing overlay-overlap and viewport-edge coverage remains the paint-order guard.

## Risks / Trade-offs

- **[Right-click is less discoverable than an always-visible ellipsis]** → Preserve slash commands, Format-menu/shortcut paths, and a hover/focus drag affordance; provide the standard keyboard context-menu action.
- **[Absolute chrome can be clipped or overlap content in narrow layouts]** → Position it within existing leading surface padding, clamp its hitbox, and test constrained-width bounds without adding flow width.
- **[Row-level right-click can conflict with selection, links, tables, or media controls]** → Define child-first event consumption, preserve selection on open, and add propagation tests for generic and specialized targets.
- **[Submenu state can become detached from a recycled virtual row]** → Keep the immutable target and window anchor in root state and retain close-on-document-scroll, mutation, tab, and mode changes.
- **[Keyboard anchor geometry may not yet be painted]** → Prefer `visual_caret_bounds`, fall back to bounded Visual Edit surface geometry, and reject stale/unsupported block ownership.
- **[A compact menu can hide infrequent transforms]** → Keep every current transform reachable in no more than one submenu level and support keyboard navigation and menu-local scrolling.

## Migration Plan

No persisted data or Markdown migration is required. Implement flow-neutral row composition first, add right-click and keyboard invocation using the existing overlay state, replace the flat menu renderer with grouped submenus, then add rendered and lifecycle tests. Rollback restores the fixed gutter and flat panel without affecting saved documents or settings; the root overlay layering fix remains independently valid.

## Open Questions

None. The selected interaction is right-click/keyboard for the compact block menu plus a hover/focus-only out-of-flow drag grip.
