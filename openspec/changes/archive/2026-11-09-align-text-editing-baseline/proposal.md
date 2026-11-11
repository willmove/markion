## Why

Source and Visual Edit do not yet share a complete desktop text-editing baseline. Word navigation, word deletion, line-end selection, page navigation, richer mouse selection, an ordinary editor context menu, structural list indentation, and several context-sensitive input rules are missing or inconsistent, while Redo lacks the standard macOS Shift+Cmd+Z binding.

## What Changes

- Add Unicode-aware previous/next-word movement and deletion, Shift+Home/End, PageUp/PageDown, and progressive Select All from the current text block to the whole document.
- Add Source double-click word selection and triple-click block selection, Visual triple-click block selection, and an ordinary editor context menu for Cut, Copy, Paste, and Select All.
- Make collapsed-list Tab/Shift+Tab use exact source-backed indentation/outdentation where a legal list transition exists, rather than always inserting four spaces.
- Add context-sensitive `* ` unordered-list repair after auto-pairing and URL-to-Markdown-link conversion on a following whitespace input; suppress both in restricted fields and IME composition.
- Accept `/` and `、` slash queries after line start or whitespace, while rejecting non-query and URL-like prefixes.
- Bind Redo to both `secondary-y` and `secondary-shift-z`, updating shortcut surfaces consistently.
- Add unit and rendered-GPUI regressions for exact source, selection, history, UTF-8, restricted contexts, and non-mutating menu lifecycle.

**Non-goals:** no character/block/cell SelectionState redesign, no multi-block drag, no merged table cells, no new clipboard flavors, no overlay positioning changes (covered by `anchor-visual-editing-overlays`), and no second rich-text document model.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: expand common editing, structural list, input-rule, mouse-selection, and editor-context-menu behavior.
- `keyboard-shortcuts`: define the fixed baseline navigation bindings and permit both Redo combinations while retaining override support for menu actions.

## Impact

- `src/text_util.rs`, `src/input_rules.rs`, `src/auto_pair.rs`, `src/block_edit.rs`: GPUI-free navigation and input policies.
- `src/app/mod.rs`, `src/app/bootstrap.rs`, `src/app/editing.rs`, `src/app/root_view.rs`, `src/app/state.rs`: actions, selection, context menu, and rendered integration.
- `src/i18n.rs` and `docs/keyboard-shortcuts.md`: shortcut reference consistency, reusing existing action labels where possible.
- Canonical Markdown, checked mutations, semantic undo, per-version caches, and existing platform input/IME paths remain authoritative.
