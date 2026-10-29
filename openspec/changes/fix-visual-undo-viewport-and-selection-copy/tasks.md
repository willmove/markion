## 1. Reproduce and diagnose

- [x] 1.1 Add and run a GPUI regression reproducing Visual Edit viewport loss on Undo.
- [x] 1.2 Identify and document the cause, covering both full snapshots and compact history records.

## 2. Implement

- [x] 2.1 Repair undo/redo viewport preservation while retaining canonical mutation and per-version caching invariants.
- [x] 2.2 Add localized context-menu Copy for non-empty Visual Edit selections, independent of formatting eligibility.

## 3. Validate

- [x] 3.1 Verify rendered viewport/caret behavior for undo, redo and structural edits, including compact history.
- [x] 3.2 Verify pointer/keyboard Copy, cross-block and empty selections, clipboard contents and non-mutating behavior.
- [x] 3.3 Run relevant Rust tests, formatting and OpenSpec validation; record results.
