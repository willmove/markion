# Verification

## Before the fix

`cargo test --bin markion content_ -- --nocapture` exercised the root test host with the patched GPUI dependency. Eight tests passed and the new existing-document regression failed on the first painted typing frame:

```text
测试 frame 0: preceding text must paint at 92px;
anchor=ListOffset { item_ix: 2, offset_in_item: -126px },
paints=Some([(2, Bounds { origin: Point { x: 10px, y: 164px },
                         size: Size { 605px × 48px } })])
```

The caret-presence and <= 1 logical-pixel centering assertions had already passed. The paint observation runs after the actual Visual Edit text paint call and intersects its bounds with the current content mask. It is opt-in and compiled only in tests; clearing the observation does not clear navigation state or modify layout.

The replay starts empty, commits `测试`, presses Enter three times, commits `测试`, presses Enter, then continues text/single-Enter cycles. This confirms the earlier runtime diagnosis through an agent-runnable test rather than through a debugger state substitution.

The smaller fixture `测试\n\n测试\n`, with the caret at its end, fails after a single `测试` commit: expected preceding text at y=116, anchor `(2, -126)`, only block 2 painted. No setup edit history is required. Removing the paragraph separator yields the earlier passing single-paragraph case.

`cargo test --bin markion typewriter_content_tests -- --nocapture` also caught the generic list omission with a cold cache: row 0 missing, anchor `(3, -100)`, only row 3 painted. A control assertion initially inspected the public bottom-aligned list offset, which represents the end sentinel rather than the effective layout offset; it was corrected to check the actual first and last painted positions. That assertion failure was a test-harness mistake, not an additional application defect.

## After list normalization

The observed action replay, seeded two-paragraph input, and generic list coverage now paint the missing predecessors. The generic controls pass for padding, cold/warm measurements, zero-height rows, top clamping, downward reveal, and short-content bottom alignment.

The first post-fix run passed three test functions and reached the typography stage of the fourth, where it failed with `typography change frame 0: 26.428589px`. This exposed setters invalidating Visual Edit measurements without requesting recentering after the earlier request had expired. The rendered-font-size and paragraph-spacing setters now request recentering when Visual Edit is active. Rendered typography changes do not introduce source-pane recentering.

## Automated results

- `cargo test --bin markion typewriter_content_tests -- --nocapture`: 4 passed. The input-history and seeded-document tests check current text painting and caret centering on every inspected frame. The seeded case includes repeated single Enter and IME composition/commit cycles, 640 x 400/default typography and 480 x 300/font size 15/spacing 13, resize, typography changes, and long wrapping text. Text that legitimately leaves the viewport is allowed to stop painting.
- Generic list integration coverage includes cold and warm measurements, zero-height and variable-height predecessors, padding, document-start clamping, downward reveal, and short-content bottom alignment. These tests execute through the root GPUI test host using the locally patched dependency; GPUI's private unit suite was not separately run.
- Presentation-only frames preserve text, dirty state, version, undo length, visual and preview Arc identity, cached text pointer, and highlight-cache entry identity/count. The paint probe remains opt-in test support with no production code or state cost.
- `cargo test --bin markion visual_typewriter_small_window -- --nocapture`: 3 passed on the final guarded typography-setter implementation.
- `cargo test`: 559 library tests passed (1 existing ignored), 543 app tests passed (2 existing ignored), and doc tests passed with no failures. This includes source/Split typewriter, ordinary reveal, manual scrolling, first/final rows, focus mode, and the four new tests on the final implementation.
- Scoped `rustfmt --edition 2024 --check --config skip_children=true` for affected app Rust files, `rustfmt --edition 2024 --check vendor/zed/crates/gpui/src/elements/list.rs`, and scoped `git diff --check`: passed.
- `openspec validate fix-visual-typewriter-content-visibility --strict`: passed. The new added coverage requirement is distinct from the previous typewriter delta and preserves the existing focus/typewriter requirement.

Raw root-test and focused-test output is available locally at `target/typewriter-root-tests.log` and `target/typewriter-small-window-tests.log`. The build emitted existing linker informational warnings and the dependency future-compatibility notice for `proc-macro-error2`; no test failed.

## Native acceptance

Pending. The available computer-use surface cannot operate native Windows windows. Automated GPUI painting is not a native replay of the user's original document or its exact every-other-Enter cadence. Task 4.3 remains unchecked until the rebuilt `target/debug/markion.exe` is verified in a small native window with the original line-then-single-Enter operation. A representative minimal document is `测试\n\n测试\n`, with the caret at the end before committing more text.

## Debug build

`cargo build` succeeded after waiting for a concurrent workspace test process to release the shared build directory. The native acceptance request asks the user to close the previous process and relaunch this executable before using the original document.

- Path: `D:\Coding\EditorProjects\markion\target\debug\markion.exe`
- Built: 2026-09-08 15:15:43 +08:00 (07:15:43 UTC)
- Size: 69,878,784 bytes
- SHA-256: `56CB1A053E02F464E0B431A9BF81AB817A8A6BB82EB7A211E5BC746764DBCE14`
- Build output: `target/typewriter-build.log`

The available implementation, automated verification, build, formatting, and OpenSpec checks are complete. Native original-document acceptance is the remaining task; this change has not been archived or represented as fully verified on the user's desktop.
