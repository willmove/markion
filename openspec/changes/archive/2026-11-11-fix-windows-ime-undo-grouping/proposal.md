## Why

Real Windows input in Visual Edit exposed intermediate Pinyin returning on Undo after candidate-selection pauses, visible-table edits jumping the viewport, and a Copy button moving when its language chip hides. Sustained OS input also exposed long pre-render waits behind Windows paint dispatch. Immediate application-callback replay had passed, leaving both the candidate pause and native message scheduling untested.

## What Changes

- Keep an active, validated IME composition in one undo capture across candidate-selection pauses.
- Preserve ordinary typing timeouts, atomic command boundaries, document/version validation, and exact Unicode source/selection restoration.
- Add deterministic callback regressions for delayed composition updates and commits, then repeat actual Windows keyboard/candidate-window and large-document interaction tests.
- Preserve the viewport while a visible table is remeasured after a cell edit or its undo; keep the code Copy button on the right when its language chip is hidden.
- Request a coalesced Windows frame after text/composition input and handled keyboard actions so low-priority paint dispatch cannot be the sole path to displaying an accepted edit.

Non-goals: replacing the Windows input method, changing security permissions, or redesigning undo storage and Markdown derivation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: group IME by its composition lifecycle, preserve visible-table edit geometry, and specify bounded native input-frame requests.

## Impact

Touches the root application's semantic undo capture, IME input regressions, opt-in diagnostics, Visual Edit viewport/header rendering, and vendored GPUI Windows frame dispatch. Undo snapshots remain cache-free, derived state stays shared per document version, and workspace library crates remain GPUI-free. No new dependencies or user-visible strings are expected.
