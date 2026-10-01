# Visual Edit focus mode verification

## Diagnosis

Source editing applied `focus_mode` while building text runs in `editor_element.rs`. The Visual Edit surface called `visual_block_view`, whose rich-content and conservative-source branches did not consume the preference at all. Existing caret ownership was already available and was not the cause.

The minimal real-renderer regression was run against an isolated checkout of `c294c7c` before the fix:

```text
cargo test --bin markion visual_focus_mode_dims_non_current_rendered_paragraph -- --nocapture
focus mode must dim the non-current rendered paragraph
test result: FAILED. 0 passed; 1 failed
```

The test itself completed in 0.01 seconds; a second direct invocation reproduced the same assertion. Isolation was needed because concurrent Git onboarding edits initially referenced functions not yet defined in the working tree.

## Fix and regression coverage

The common Visual Edit row factory now applies opacity after constructing content, including early-return source islands. It computes caret ownership once and passes that value into the content renderer. No persistent focus state, derived-cache invalidation, or additional Markdown parsing was introduced.

All four new GPUI regressions pass in the isolated checkout with the fix (0.02 seconds):

- The original non-current-paragraph failure.
- Light/dark themes, rich prose, quote/list leaves, code, front matter, unsupported source islands, tables, HTML, formulas, images, and blank/empty documents.
- Selection direction, EOF, tab/view changes, disabled focus, and unchanged document version, dirty state, undo/redo history, cached source text, and shared visual/preview caches.
- Real window pointer placement and keyboard navigation across paragraph boundaries.

The structured fixture deliberately uses existing caret ownership for formula closing boundaries: that offset can still belong to the preceding formula. Focus presentation follows caret painting rather than introducing a competing boundary rule.

Tests inspect styles on production GPUI row elements and exercise headless window input; no native screenshot comparison was performed.

## Current-workspace validation

After the concurrent Git onboarding implementation became compilable, `cargo test` was run in the original working tree with this fix and the existing uncommitted changes:

```text
Library:     559 passed; 0 failed; 1 ignored
Application: 538 passed; 0 failed; 2 ignored
Doc-tests:     0 passed; 0 failed
```

All four new focus-mode regressions passed in that application run. Total: 1097 passed, 3 ignored, no failures. Source/Visual Edit typewriter, caret ownership, pointer editing, and cache tests passed alongside the new coverage.

`rustfmt --edition 2024 --check --config skip_children=true src/app/preview.rs src/app/mod.rs src/app/focus_mode_tests.rs`, the scoped `git diff --check`, and `openspec validate fix-visual-edit-focus-mode` pass. No diagnostic instrumentation remains. The temporary isolated checkout was removed after validation; the change remains active and unarchived.
