## Verification

Date: 2026-09-09

Source revision before the working-tree change: `9ac844be0249718203762095c22fc08dd48ef89c`

Toolchain: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, `cargo 1.97.1 (c980f4866 2026-06-30)`, host `x86_64-pc-windows-msvc`.

### Deterministic presented-frame evidence

The regression tests observe the caret produced by `VisualEditableText::paint`, including its frame generation, document instance/version, source selection head, owning block stable ID, caret bounds, and whether the caret quad was emitted. Each action is followed only by GPUI effect/frame draining; the assertions do not send a second input.

Before the implementation, this command was run after adding the initial five presented-frame tests:

```text
cargo test --bin markion deferred_caret_tests -- --nocapture
```

Result: 1 passed, 4 failed. The single-Enter control passed, while the painted caret lagged the canonical caret in every navigation case:

- cached cross-block Down: painted source cursor 3, canonical cursor 5;
- cached Select Down: painted selection head 3, canonical selection head 5;
- second action across the authored blank row: painted source cursor 2, canonical cursor 4;
- virtualized cross-block Down: painted source cursor 1301, canonical cursor 1314.

This reproduces the reported cadence deterministically: navigation state changed during paint, but the scene still contained the old caret and needed unrelated later invalidation.

After moving cached completion into the action and unmeasured completion into a revalidated post-paint callback, the expanded suite was run:

```text
cargo test --bin markion deferred_caret_tests -- --nocapture
```

Result: 11 passed, 0 failed. Coverage includes paragraph/tail Enter, same-block Down, cached adjacent-block Up/Down, Select Up/Down, authored whitespace stops, virtualized reveal/deferred completion, an otherwise idle window, stale request replacement/pointer/document cancellation, and bounded caret-follow/typewriter frames.

The whitespace control also confirms unchanged text, document version, dirty state, undo/redo lengths, preferred X, preview/visual `Arc` identities, memoized highlight identity, and cached text-handle identity across presentation-only navigation.

### Focused compatibility suites

```text
cargo test --bin markion visual_edit_ -- --nocapture
cargo test --bin markion visual_typewriter_small_window -- --nocapture
cargo test --bin markion typewriter_content_tests -- --nocapture
```

Results:

- Visual Edit suite: 49 passed, 0 failed;
- small-window typewriter suite: 3 passed, 0 failed;
- typewriter content-coverage suite: 4 passed, 0 failed.

These include the existing terminal Enter, blank-line Up/Down, virtualized adjacent-block navigation, off-screen reveal/manual-scroll, typewriter centering, IME/wrapping, and content-coverage controls.

### Repository checks

```text
cargo fmt --check
cargo test
cargo test --workspace
git diff --check
openspec validate fix-visual-edit-deferred-caret-repaint --strict
```

All commands passed. `cargo test` completed the root library and application suites with no failures (559 library tests passed with 1 ignored; 555 application tests passed with 2 ignored). `cargo test --workspace` completed all workspace member, integration, and documentation tests with no failures. Cargo reported the existing future-incompatibility warning for `proc-macro-error2 v2.0.1`.

### Built artifact and native matrix

```text
cargo build --bin markion
```

Windows debug artifact:

- path: `target/debug/markion.exe`;
- size: 69,579,264 bytes;
- SHA-256: `38067DC2B5F3BFD3F6322C4C276CE517BA1ED78A2C04CDCD8E41C57991C1DD72`;
- build host: Windows 11 Pro 10.0.26200, x86-64.

Native Linux compositor verification remains explicitly pending because this workspace exposes only the Windows host:

| Backend | Scale | Typewriter off | Typewriter on | Status |
| --- | --- | --- | --- | --- |
| COSMIC Wayland on Pop!_OS 24.04 | unavailable | not run | not run | Pending: no COSMIC/Wayland host |
| Other Wayland compositor | unavailable | not run | not run | Pending: no Wayland host |
| XWayland | unavailable | not run | not run | Pending: no XWayland host |

The native matrix must be completed on Linux using the minimal `AB`, `# A\nB`, `A\n\nB`, and long virtualized fixtures, recording the first presented frame after Enter, Up/Down, and Select Up/Down.
