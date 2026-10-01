## 1. Document and tab dormancy primitives

- [x] 1.1 Add `MarkdownDocument::evict_derived_caches` that clears all derived `RefCell`/`Cell` caches and resets pending source edits to `Full` without changing text, path, dirty, or `text_version`.
- [x] 1.2 Add `EditorTab::enter_dormant` that calls document eviction, clears shaped-line / layout caches, resets preview/visual list mirrors to empty, clears ephemeral visual navigation/expand state, and releases preview-image claims if that API exists.
- [x] 1.3 Add document unit tests: after populating all derived caches, eviction reports them unpopulated, version and text unchanged, and accessors can repopulate afterward.

## 2. Wire into tab switching

- [x] 2.1 Call `enter_dormant` on the tab being deactivated inside `switch_active_tab` (after finishing undo capture / clearing drag flags).
- [x] 2.2 Ensure `replace_active_tab` and close-tab paths still fully drop state (no regression); dormancy is not required for removed tabs.
- [x] 2.3 Add a GPUI test: two Visual Edit tabs, switch away and back — selection and undo survive; inactive tab's memory sites for visual/preview/shaped_lines are zero while inactive.

## 3. Harness and docs

- [x] 3.1 Extend the memory harness with a dormancy scenario (2× `plain_long`, switch, assert accounted derived drop / restore).
- [x] 3.2 Update `docs/memory-retention.md` with the dormancy policy and a before/after attribution note.
- [x] 3.3 Run `cargo fmt --check`, `cargo test --workspace`, and `openspec validate evict-inactive-tab-caches`.
