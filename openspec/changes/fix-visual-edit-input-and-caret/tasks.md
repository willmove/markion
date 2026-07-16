## 1. Visual Source Coverage

- [x] 1.1 Add a whitespace visual-block variant and preserve whitespace-only gaps and trailing source ranges without turning them into permanently visible unsupported-source cards
- [x] 1.2 Add focused model tests for whitespace range coverage, trailing caret focus, and unchanged per-version visual cache reuse

## 2. Platform Input and Caret Geometry

- [x] 2.1 Add one non-hit-testing Visual Edit input bridge that registers the existing `EntityInputHandler` once per frame for empty and non-empty documents while leaving Read mode without an editing input target
- [x] 2.2 Capture the focused visual caret rectangle and make `EntityInputHandler::bounds_for_range` use mode-appropriate source or visual geometry with a conservative Visual Edit fallback
- [x] 2.3 Verify text insertion, selection replacement, IME marked-text updates, dirty state, undo/redo, autosave scheduling, and derived-cache invalidation continue through the existing document mutation path

## 3. Visual Cursor Following

- [x] 3.1 Add per-tab one-shot visual reveal state and a source-offset-to-visual-block lookup that covers prose, source islands, whitespace, boundaries, and EOF
- [x] 3.2 Request and consume visual-row reveal after mode entry, cursor/selection navigation, document mutation, search navigation, and outline jumps without overriding unrelated manual scrolling

## 4. Regression Verification

- [x] 4.1 Enable GPUI test support for dev tests and add rendered-window regression tests for normal input in populated and empty Visual Edit documents plus Read-mode non-mutation
- [x] 4.2 Add helper tests for caret bounds/reveal behavior and confirm cursor-only interaction leaves document versions and cached visual blocks unchanged
- [x] 4.3 Run `cargo fmt --check`, `cargo test`, and `cargo test --workspace`
- [x] 4.4 Run `openspec validate fix-visual-edit-input-and-caret` and resolve all validation errors
