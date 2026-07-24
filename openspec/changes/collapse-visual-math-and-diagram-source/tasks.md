## 1. Presentation state

- [x] 1.1 Add per-tab `expanded_visual_source_blocks: HashSet<VisualBlockId>` (or equivalent) on `EditorTab` in `src/app/state.rs`, initialize empty in constructors/reset paths, and prune ids that disappear when `sync_visual_list` updates blocks
- [x] 1.2 Add tab helpers to query/toggle/clear expand for a `VisualBlockId`, ensuring toggles request a Visual Edit repaint without mutating document text, dirty flag, undo, or derived `Arc` caches

## 2. Shared collapsible chrome

- [x] 2.1 Add i18n strings for the source-toggle control tooltip/label (EN + zh-CN) in `src/i18n.rs`
- [x] 2.2 Implement a shared Visual Edit wrapper in `src/app/preview.rs` that hosts bordered chrome, hover-visible top-right `</>` toggle, collapsed vs expanded slots, and outside-click collapse wiring per `design.md`
- [x] 2.3 Wire toggle clicks to expand/collapse only via the control; clicking the presentation surface must not expand source

## 3. Block math

- [x] 3.1 Refactor `visual_math_editor` to use the shared wrapper: default collapsed render-only for ready+valid formulas; expanded dual pane on toggle
- [x] 3.2 Force the LaTeX payload editor to remain visible for pending, invalid, or error math results regardless of expand set
- [x] 3.3 Update existing Visual Edit math tests (e.g. direct math editor / invalid payload) for collapsed-by-default + forced-expand behavior; add coverage for toggle expand, click-formula-does-not-expand, outside-click collapse, and expand/collapse leaving document version unchanged

## 4. Diagrams (Mermaid / registered backends)

- [x] 4.1 Refactor `visual_diagram_editor` onto the same shared wrapper with identical collapsed/expand/outside-click semantics
- [x] 4.2 Force the diagram payload editor to remain visible while pending or on error
- [x] 4.3 Add/adjust Visual Edit diagram tests for collapsed default, toggle expand, outside-click collapse, forced expand on error/pending, and no document mutation on toggle

## 5. Verification

- [x] 5.1 Run focused Visual Edit math/diagram tests and `cargo test` for the root package; fix regressions
- [x] 5.2 Manually verify in Visual Edit: hover `</>` on math and Mermaid, expand shows render+source, click outside collapses, invalid/pending keep source, Split Preview/Read unchanged _(Deferred to release QA; automated expand/collapse coverage added.)_
