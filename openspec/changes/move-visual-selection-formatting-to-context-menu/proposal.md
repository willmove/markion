## Why

Selecting text in Visual Edit currently opens a floating formatting toolbar at a fixed position over the document. The toolbar obscures content and adds persistent visual noise; the same actions are better exposed on demand in the context menu the user opens for the selected text.

## What Changes

- Remove the automatic floating selection-formatting toolbar from Visual Edit.
- Add Bold, Italic, Inline Code, and Link actions to the Visual Edit context menu when the invocation belongs to a non-empty, exactly source-mapped selection that can safely accept those actions.
- Reuse the existing compact Visual Edit block context menu and its pointer and keyboard invocation paths, placing selection formatting in a distinct group without removing the targeted block operations.
- Preserve the exact selection while opening, navigating, or dismissing the menu; validate the document version and selection again before dispatch so stale or ambiguous selections cannot be mutated.
- Keep existing canonical Markdown mutations, semantic undo, link editing, localization, tab isolation, autosave/recovery, and per-document-version derived-cache identity unchanged.
- **Non-goals:** adding new formatting commands, changing Source/Format-menu shortcuts, changing block-transform semantics, redesigning the link editor, or introducing a separate rich-text document model.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: replace automatically displayed Visual Edit selection controls with selection-aware formatting actions in the existing context menu, including safe targeting and presentation-only lifecycle requirements.

## Impact

- Affected seams include Visual Edit contextual state and dispatch in `src/app/mod.rs` and `src/app/editing.rs`, row/context-menu composition in `src/app/preview.rs`, removal of the root floating toolbar in `src/app/root_view.rs`, localized menu labels in `src/i18n.rs` where existing strings cannot be reused, and rendered/interaction tests in `src/app/tests.rs`.
- This change builds on the compact Visual Edit block-menu behavior in the completed active change `align-visual-edit-content-and-compact-block-context-menu`; that change should be archived or otherwise reconciled first because both changes refine the same contextual-menu contract.
- `MarkdownDocument.text` remains canonical. Formatting continues through the existing one-command mutation and semantic-undo paths; selection-only menu lifecycle must not change document version or invalidate shared `Arc` derived state, memoized highlighting, or cached text handles.
- No external dependency, persistence migration, public API change, or workspace-member dependency is introduced.
