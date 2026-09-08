## Why

In Visual Edit with typewriter mode enabled, typing a line and pressing Enter in an existing document can make preceding content disappear and reappear in a small window. Runtime investigation found that an upward centering request can leave the virtual list anchored to a later item with a negative offset, omitting visible predecessors while the caret remains correctly centered; the previous caret-only regression checks therefore miss this failure.

## What Changes

- Preserve all document content that intersects the Visual Edit viewport during typewriter centering, including the first painted frame after typing or Enter.
- Normalize the upward autoscroll result at the GPUI list boundary before its existing prepaint retry, preserving the requested pixel position and measuring only the predecessors needed to resolve it.
- Add a deterministic regression with preceding paragraphs and blank lines, checking actual painted content together with caret position after each input action and subsequent unchanged frames.
- Preserve boundary centering, manual scrolling, ordinary reveal, and the existing bounded refinement behavior.
- Record the distinction between the confirmed omitted-content mechanism and the user's native every-other-Enter timing, which still requires verification with the original document.

Non-goals: changing Markdown newline semantics or visual block ownership, increasing retry counts, scroll animation, new preferences, broad list refactoring, or unrelated workspace changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chrome-platform`: Add an explicit Visual Edit typewriter viewport-coverage requirement so centered caret geometry cannot mask missing visible document content.

## Impact

The locally patched GPUI list implementation in `vendor/zed/crates/gpui/src/elements/list.rs`, Visual Edit's centering integration, and focused rendering tests in the root application. No new dependency, public API, persisted format, or document migration is intended. Scroll-only updates must preserve document text, dirty state, version, undo history, shared per-version Markdown/visual caches, memoized highlighting, and cached text handles. Library workspace members remain GPUI-free.

This is a follow-up to `fix-small-window-typewriter-jitter`, whose changes are already present in the checkout but whose completed checklist does not verify preceding-content visibility. It retains that work and adds a separate requirement rather than duplicating or rewriting its delta specs. Existing focus-mode work must remain intact.
