## 1. Windows Help action

- [x] 1.1 Route the bundled index through the Windows file manager launcher, preserve native paths and Explorer quoting, and surface launch errors with the existing translations. Verify formatting and compile the Windows app.

## 2. Verification

- [x] 2.1 Invoke the Help action on Windows with resources in a path containing spaces and non-ASCII characters. Confirm that Explorer selects `README.md` and the current document remains open.
- [x] 2.2 Run the workspace tests, bundled example verification, and strict OpenSpec validation.

## Verification record

- Reproduced the original Windows menu behavior in an isolated profile. Shell path parsing accepted the native path and rejected the mixed-separator path with `0x80070057` even though the file existed.
- Built the fixed Windows app and invoked Help → Examples and Templates with an unsaved edit. Explorer selected `README.md` at a path containing spaces, Chinese characters, and a comma. The scratch workspace and dirty document stayed open, and Undo restored the original text.
- Forced the Explorer process launch to fail in an isolated application fixture. The Help menu dismissed and Markion displayed the localized failure in the status bar.
- `cargo fmt --all -- --check` passed. The example verifier checked 18 Markdown files and their relative links and matched all 19 files in the staged resource payload.
- `cargo test --workspace --locked` passed with 2,391 passing tests and five ignored. Strict OpenSpec validation and `git diff --check` passed.
