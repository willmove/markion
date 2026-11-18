## 1. Reproduce on Main

- [x] 1.1 Add regression cases for the overlapping writes, retained draft collision and CRLF Select All. Record each baseline failure before changing its behavior.

Windows baseline at `0bcd212` failed all three cases. The disk contained `old` after the successful manual save (#29), the retained recovery contained the unrelated draft's bytes (#28), and the first CRLF Select All returned `0..47` instead of `16..31` (#32).

## 2. Protect Document Content

- [x] 2.1 Order same-destination saves and reconcile an in-flight autosave at the explicit save boundary. Verify overlapping writes, delayed completions, and failed saves.
- [x] 2.2 Publish new recovery snapshots without overwriting existing files. Verify retained and new drafts, repeated autosaves, and legacy snapshot loading.
- [x] 2.3 Resolve the containing source paragraph with LF and CRLF separators. Verify first and second Select All, exact replacement, and Undo without changing derived cache behavior.

## 3. Verification

- [x] 3.1 Run the targeted regressions, workspace tests, formatting check, and OpenSpec validation.

Windows verification passed all nine new regression tests. The full locked workspace run finished with 2,247 passed, zero failed and five ignored. `cargo fmt --all -- --check` and strict change validation passed.
