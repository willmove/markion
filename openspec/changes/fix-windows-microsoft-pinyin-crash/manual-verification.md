# Manual Microsoft Pinyin verification

Automated tests cover the GPUI routing backport and Markion's UTF-16 range, preedit, commit, cancellation, geometry, and undo behavior. The final native smoke check requires an interactive Windows session with Microsoft Pinyin enabled.

## Automated verification completed

- A default parallel `cargo build -p markion` passes against both repository-local patches after rebuilding cc-rs and the affected native dependencies; the reproduced C1056 failure no longer occurs.
- `cargo test -p markion ime -- --nocapture` passes all 18 matching tests.
- The two Visual Edit preedit/highlight regressions pass, including the paragraph shape from the reported document and an IME range overlapping bold text.
- `cargo test --workspace --quiet` passes with no failures.
- `openspec validate fix-windows-microsoft-pinyin-crash` and `git diff --check` pass.
- `cargo fmt --all --check` was run. The task's changed lines are formatted; the current toolchain still reports pre-existing drift in unrelated lines of `src/app/mod.rs`, `src/app/preview.rs`, `src/app/preview_image.rs`, `src/lib.rs`, `src/parse.rs`, and `src/visual.rs`, which were intentionally left unchanged.

## Native verification completed

- The failing build launched at 17:13:52 reproduced the original edit location and captured the exact DirectWrite UTF-8 boundary panic before Windows terminated the Win32 callback with `0xc0000409`.
- The corrected build launched at 17:46:22 with the same saved document/session, remained responsive during the native smoke-check window, and closed normally at 18:16:16.
- Redirected stderr contained no DirectWrite panic (only two menu-close diagnostics), and Windows Application events after launch contained no `markion.exe` Application Error or Windows Error Reporting event.

## Original regression

The verification can be repeated for release acceptance as follows:

1. Build and launch the patched debug application with `cargo run`.
2. Open the original Markdown document and enter Visual Edit.
3. Place the caret after the `0` indicated in the reported screenshot.
4. Switch to Microsoft Pinyin and type `nihao` without committing immediately.
5. Confirm Markion remains running, marked text is visible, and the candidate window follows the caret.
6. Commit `你好`, then undo and redo once; confirm one Undo removes the whole composition and one Redo restores it.
7. Start another composition, press Escape, and confirm the source and selection remain valid.

## Crash-event check

Record the launch time, complete the smoke check, then query Windows Application events after that time. Acceptance requires no new `markion.exe` application error with exception `0xc0000409` and no matching new Windows Error Reporting `BEX64` event.
