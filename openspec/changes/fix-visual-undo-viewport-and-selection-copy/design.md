## Context

History is stored per tab as full snapshots and compact text diffs. Restores use the checked canonical mutation boundary; rendering derives per-version visual blocks and synchronizes a virtualized list. The block context menu already preserves canonical selection and has localized inline formatting actions.

## Goals / Non-Goals

**Goals:** Preserve the user's rendered editing location across history restoration and expose selection Copy without mutations.

**Non-Goals:** Persisting viewport positions in history, changing clipboard formats, or redesigning history grouping.

## Decisions

- Build a GPUI reproduction that types in a scrolled, visible paragraph, invokes the history action, and checks the rendered list anchor and caret. Test full and compact entries plus redo.
- Keep the existing flow: history -> checked mutation -> fresh versioned derivation -> list synchronization -> bounded caret visibility. Repair the point that loses the anchor instead of restoring an arbitrary old scroll snapshot.
- Add Copy independently of selection formatting eligibility so cross-block selections can be copied. Reuse existing localized labels and canonical selected-text clipboard behavior.

## Risks / Trade-offs

- Whole-document history restores may rebuild visual identities -> test list synchronization and source mapping across restoration.
- Row count/height changes can move the restored caret -> retain normal minimal visibility reconciliation and test structural edits.
- Menu indices depend on optional actions -> preserve keyboard initialization and disabled-state handling through the common menu item list.

## Diagnosis

The GPUI reproduction types into Paragraph 4 while the virtual list is anchored at item 2. Undo restores the exact source and cursor but moves the list anchor to item 0. The regression failed twice in 0.03 seconds after initial compilation.

Full snapshots used a whole-document canonical mutation, clearing the source-mapped cache and every visual row identity. Visual list synchronization consequently spliced all rows; the list moves an anchor inside a replaced interval to that interval's start, which was zero. Compact history already used range mutations and could retain unaffected row identities. Restoring full snapshots via the existing UTF-8-safe history compaction and checked range mutation removes this distinction without changing snapshot storage or document instance/version semantics.

Selection Copy captures the canonical non-empty range separately from inline-format eligibility, prepends a localized Copy item to the block menu, and validates the captured range and block version on activation. It calls the same canonical-selection clipboard helper as the Copy shortcut; it does not invoke a document mutation or a caret move.

## Validation

- Before the fix, `cargo test --bin markion visual_edit_undo_redo_preserves_visible_edit_viewport -- --nocapture` failed with list anchor `0` instead of `2`; repeating the compiled regression reproduced the same failure.
- After the fix, 189 Visual Edit tests passed, including full/compact undo and redo, structural newline restoration, manually scrolling away before undo, pointer and keyboard Copy, cross-block and empty selections, stale selection rejection, exact clipboard contents, and unchanged document/selection/cache/history/scroll state.
- `cargo test --config 'profile.test.opt-level=0'` passed all root-package suites: 710 library tests and 752 application tests passed; 3 existing ignored tests remained ignored, with no failures. The test-profile override speeds application test compilation without changing repository configuration.
- `cargo fmt --all -- --check`, `git diff --check`, and strict OpenSpec validation passed. No debug instrumentation or temporary source harnesses remain. No release was packaged or published.
