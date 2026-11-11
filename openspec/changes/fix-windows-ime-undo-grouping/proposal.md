## Why

Real Windows keyboard input in Visual Edit reproduced an IME undo defect: after composing `ni hao`, committing `你好`, and pressing Undo, transient Pinyin reappears instead of restoring the pre-composition document. Immediate application-callback replay had passed, leaving candidate-selection pauses untested.

## What Changes

- Keep an active, validated IME composition in one undo capture across candidate-selection pauses.
- Preserve ordinary typing timeouts, atomic command boundaries, document/version validation, and exact Unicode source/selection restoration.
- Add deterministic callback regressions for delayed composition updates and commits, then repeat actual Windows keyboard/candidate-window and large-document interaction tests.
- Preserve the viewport while a visible table is remeasured after a cell edit or its undo; keep the code Copy button on the right when its language chip is hidden.

Non-goals: replacing the Windows input method, changing security permissions, or redesigning undo storage and Markdown derivation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: clarify that an active IME session is grouped by composition lifecycle rather than the ordinary typing timeout.

## Impact

Touches the root application's semantic undo capture, IME input regressions and Visual Edit viewport/header rendering. Undo snapshots remain cache-free, derived state stays shared per document version, and workspace library crates remain GPUI-free. No new dependencies or user-visible strings are expected.
