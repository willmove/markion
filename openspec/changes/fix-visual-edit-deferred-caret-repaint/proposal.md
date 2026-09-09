## Why

Visual Edit can update the canonical caret during a paint pass without reliably scheduling a follow-up draw. On compositors whose frame loop does not produce an incidental invalidation, especially native Wayland, the first Enter or vertical-navigation action can take effect in the model while the painted caret remains stale until the next key press or text input.

## What Changes

- Make every accepted Visual Edit caret move visible in the next eligible rendered frame without requiring a second user action.
- Resolve cross-block vertical navigation immediately when current geometry is already cached; when a virtualized target must first be measured, publish geometry during paint and complete the move after the paint pass through a version-checked deferred handoff.
- Schedule bounded caret-follow refinement through an explicit post-paint frame request instead of relying on entity notification issued while GPUI is drawing.
- Add first-presented-frame regressions for Enter, Up/Down, selection navigation, blank-line stops, virtualized targets, and typewriter on/off behavior.

Non-goals: changing Markdown newline bytes, skipping first-class blank-line navigation stops, changing source/Edit/Split behavior, adding scroll animation, or globally changing GPUI invalidation semantics.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Require a single Visual Edit caret-moving input to update both canonical selection and painted caret without another input, including deferred navigation into newly measured virtualized rows.

## Impact

The root GPUI application paths for Visual Edit navigation, caret geometry publication, follow-frame scheduling, and rendering tests are affected, primarily `src/app/editing.rs`, `src/app/preview.rs`, `src/app/state.rs`, and `src/app/tests.rs`. The change must preserve `MarkdownDocument.text` as the canonical model, per-version shared Markdown/visual `Arc` caches, memoized highlighting, cached text handles, undo history, dirty state, and per-tab viewport state. No dependency, persisted-format, public API, or workspace-member change is intended.
