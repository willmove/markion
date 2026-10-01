## 1. Flow-Neutral Visual Block Geometry

- [x] 1.1 Refactor `visual_block_chrome` so eligible row content owns the complete document-column width, remove the fixed 28/48-pixel spacer and ellipsis trigger, and keep stable debug selectors for rendered geometry tests.
- [x] 1.2 Render the existing drag grip as a hover/focus/active-drag absolute sibling within the leading interaction padding, clamp its hitbox for narrow surfaces, and prove its visibility state does not mutate document or cache state.
- [x] 1.3 Preserve intentional list, quote, source-island, table, image-field, and other component-internal indentation while removing only the block-operation gutter.

## 2. Context Targeting and Lifecycle

- [x] 2.1 Extend the ephemeral block-menu presentation state with compact-menu/submenu and keyboard-active-item state while retaining the immutable exact `BlockTarget` and window-space anchor.
- [x] 2.2 Open the block menu from an eligible row's right-click at the pointer anchor, target the clicked block even when it does not own the caret, preserve the current exact selection, and honor child-first propagation for specialized interactions.
- [x] 2.3 Add a Visual Edit keyboard context-menu action and shortcut routing that targets the caret-owning eligible block and anchors near `visual_caret_bounds` with a bounded surface fallback.
- [x] 2.4 Reconcile the new entry paths with existing Escape, outside action, document scroll, mutation, stale target, tab/mode change, application menu/modal, slash palette, link editor, and selection-toolbar overlay precedence without changing canonical state.

## 3. Compact Localized Block Menu

- [x] 3.1 Define a menu presentation model that identifies the current block type, groups Text with Heading 1–6 and the three List transforms into two shallow submenus, and exposes Quote, Code Block, Divider, Table, Duplicate, Move Up, Move Down, and a separated destructive Delete according to availability.
- [x] 3.2 Replace the flat root menu renderer with compact localized root/submenu panels that reuse opaque root overlay chrome, pointer occlusion, viewport flipping/clamping, menu-local scrolling, current-type indication, disabled states, and destructive styling.
- [x] 3.3 Implement pointer and keyboard navigation across enabled menu items and submenus—Up/Down, Right/Enter, Left, and Escape—while dispatching every confirmation through the existing exact transform, duplicate, reorder, or delete command.
- [x] 3.4 Add or update all user-facing menu, submenu, action, status, and shortcut labels through `src/i18n.rs` for every supported language.

## 4. Rendered Regression Evidence

- [x] 4.1 Add GPUI geometry coverage showing equivalent top-level Visual Edit headings, paragraphs, media, and Read-mode content share the document axis and available width, with semantic indentation explicitly excluded.
- [x] 4.2 Add a long wrapped-prose fixture proving hover, focus, drag-grip visibility, and block-menu availability leave content bounds, row height, and line breaks unchanged.
- [x] 4.3 Add interaction coverage proving right-click targets a non-caret eligible block without collapsing selection, specialized child handlers can consume the event, and unsupported or stale targets do not open an actionable generic menu.
- [x] 4.4 Add keyboard and pointer coverage for both transform submenus, current-type indication, disabled boundary moves, viewport-edge reachability, exact command dispatch, and one-step Undo.
- [x] 4.5 Add drag coverage proving the flow-neutral grip retains the existing before/after reorder result and creates no mutation before drop, plus lifecycle assertions for unchanged source, version, selection, history, dirty state, and derived `Arc` identity while menus are only presented or navigated.
- [x] 4.6 Repeat the reported headings/prose/image/formula scenario on Windows and visually confirm Visual Edit alignment matches the shared document column while the compact context menu remains above content and within the viewport.

## 5. Verification

- [x] 5.1 Run `cargo fmt --check` and the focused Visual Edit geometry, block-context-menu, keyboard, lifecycle, and drag GPUI tests.
- [x] 5.2 Run `cargo test --workspace` and resolve regressions without weakening exact-target, one-mutation/one-undo, overlay, or cached-per-version invariants.
- [x] 5.3 Run `openspec validate align-visual-edit-content-and-compact-block-context-menu` and `pwsh ./scripts/check-quality.ps1`, confirming the complete repository quality gate passes.
