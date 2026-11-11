## Why

Slash commands, link editing, and inline token/math editing currently open at fixed window positions rather than near the content they affect. Visual formatting requires opening a context menu, and editing can leave block controls visible during continuous input. These differences add pointer travel and visual interruption even when the underlying mutations are correct.

## What Changes

- Add a shared geometry policy for document overlays that anchors to painted caret, selection-owning block, or Visual input bounds and falls back safely when geometry is unavailable.
- Move the slash palette and document link/token/math editors from fixed window positions to that anchored policy, retaining bounded preference dialogs and theme-aware chrome.
- Add a compact selection-contextual toolbar for Bold, Italic, Inline Code, and Link while preserving the existing context-menu actions and exact revalidation.
- Suppress hover/focus block grips after document input until pointer movement, and keep all overlay visibility changes presentation-only.
- Add rendered-GPUI evidence for anchors, formatting actions, stale targets, palette/dark-theme presentation, and unchanged document/cache state.

**Non-goals:** no SelectionState or EditTransaction redesign, no new formatting commands or clipboard formats, no multi-line rich-text field framework, no OS pointer hiding, and no changes to canonical mutations or source-backed editing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: augment selection-contextual formatting with an anchored automatic toolbar and place slash/document-inline editors near their source interaction.
- `chrome-platform`: define presentation-only anchored-overlay and input-suppressed block-chrome behavior, including theme consistency.

## Impact

- `src/app/mod.rs`, `src/app/application.rs`, `src/app/editing.rs`: overlay visibility, selection revalidation, and input suppression state.
- `src/app/root_view.rs`, `src/app/preview.rs`: anchored slash/link/token/selection-toolbar rendering and block-grip visibility.
- Existing `VisualSelectionFormatTarget`, `BlockTarget`, `LinkEditorState`, and `TokenEditorState` remain the mutation/targeting seams.
- Canonical Markdown, selection, versioned `Arc` caches, undo, and platform input are unchanged.
