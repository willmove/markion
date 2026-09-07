## 1. Reproducible XIM Dependency Patch

- [x] 1.1 Vendor the published `zed-xim 0.4.0-zed` package under `vendor/zed-xim`, retain its upstream license and metadata, add it to the root workspace exclusion needed for focused manifest tests, and document the source plus Markion-only edits in `MARKION_PATCH.md`.
- [x] 1.2 Add the root `[patch.crates-io]` override for `zed-xim`, pin the vendored package to `xim-ctext 0.4.1`, regenerate `Cargo.lock`, and verify the Linux dependency graph resolves the local XIM package and the intended decoder version.

## 2. Panic-Safe Compound-Text Decoding

- [x] 2.1 Add red request-boundary tests in the vendored XIM package for GB2312 and KS C 5601 preedit/commit/reset payloads, mixed CJK/ASCII compound text, and an invalid payload that currently panics or bypasses typed error handling.
- [x] 2.2 Add a compound-text decode variant to `ClientError` and replace the reset, character-commit, and preedit `expect` calls with typed error propagation that never records payload bytes or user text.
- [x] 2.3 Run the vendored package's focused client tests and confirm valid Chinese/Korean fixtures reach the handler as exact UTF-8 while invalid input returns `ClientError` without unwinding.

## 3. Linux X11 Failure Cleanup

- [x] 3.1 Add a failing GPUI/Markion regression that models an active non-ASCII preedit followed by XIM failure cleanup and asserts the last accepted text and selection remain valid, the marked range and IME undo capture finish, and later direct input succeeds.
- [x] 3.2 Extend GPUI's existing `filter_event` error branch to clear the composing flag and unmark the owning window before dropping the failed XIM connection, while retaining its existing direct-key fallback and content-free error log.
- [x] 3.3 Run the focused Linux XIM and Markion IME suites and verify cleanup uses only the existing canonical mutation/undo path without adding derived-state work or bypassing document-version cache invalidation.

## 4. Validation and Platform Evidence

- [x] 4.1 Run `cargo fmt --check`, the focused vendored-XIM tests, `cargo test -p markion ime -- --nocapture`, and `cargo test --workspace`.
- [x] 4.2 Compile and test the Linux X11 path in CI or a Linux environment, including `zed-xim` with its `x11rb-client` features, and confirm the resolved build contains the local patch rather than the stale registry source.
- [ ] 4.3 Manually verify IBus and Fcitx on Linux X11 where available: Chinese and Korean preedit updates, commit, cancellation, undo/redo, malformed-input safe degradation where reproducible, continued direct typing, and absence of a main-thread panic; record the environment and results in `manual-verification.md`.
- [x] 4.4 Reconcile this delta with the completed `fix-windows-microsoft-pinyin-crash` IME delta before either change is synced or archived, then run `openspec validate fix-linux-xim-compound-text-crash`.
