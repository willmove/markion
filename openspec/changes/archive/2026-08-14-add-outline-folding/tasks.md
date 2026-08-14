## 1. Outline Tree Projection

- [x] 1.1 Add internal structural node keys and a linear projection helper that derives parent/descendant boundaries, disclosure state, visible rows, and the visible active representative from the cached flat heading vector.
- [x] 1.2 Add pure-helper coverage for default-expanded outlines, skipped heading levels, nested collapses, preserved nested fold state, leaf rows, duplicate titles, and active headings hidden beneath collapsed ancestors.
- [x] 1.3 Reconcile collapsed structural keys against each changed heading hierarchy so body-only offset shifts retain folds while removed, renamed, or otherwise obsolete identities cannot hide unrelated headings; cover hierarchy-edit and duplicate-sibling cases.

## 2. Per-Document Folding State

- [x] 2.1 Add session-only collapsed-outline state to `DocumentTabState`, initialize new/opened documents fully expanded, and provide application helpers that toggle only the active document's projected node.
- [x] 2.2 Verify folding state survives document-tab switches, remains isolated between documents, is absent for image tabs, and never enters document snapshots, undo/redo, preferences, or persistence paths.
- [x] 2.3 Add state-level regression coverage proving collapse/expand leaves Markdown text, version, dirty state, selection, history, and the existing cached derived-state identities unchanged.

## 3. Disclosure UI and Interaction Routing

- [x] 3.1 Add themed collapsed/expanded chevron assets to the existing embedded UI icon set and verify both assets resolve through `IconAssets`.
- [x] 3.2 Render projected visible outline rows with a fixed-width disclosure slot, actionable chevrons only for headings with descendants, aligned leaf labels, existing hierarchy indentation, compact row metrics, hover/active styling, and the existing scroll container.
- [x] 3.3 Give disclosure controls and heading labels separate pointer handlers: disclosure clicks stop propagation and toggle folding, while label clicks retain the existing context-aware navigation path without changing folding state.
- [x] 3.4 Apply active styling to the nearest visible collapsed ancestor when the canonical active heading is hidden, then restore exact-heading styling when its ancestors are expanded.

## 4. Interaction Verification

- [x] 4.1 Add GPUI tests showing a disclosure click collapses and re-expands the correct subtree, preserves independently collapsed nested sections, and does not move the canonical cursor or preview list.
- [x] 4.2 Add GPUI regression tests showing heading-label clicks do not toggle folds and still navigate correctly in Edit, Visual Edit, Split Preview, and Read modes.
- [x] 4.3 Add UI coverage for per-document tab isolation, leaf-row alignment, compact row height, active-hidden feedback, and scrolling a long partially folded outline.

## 5. Validation

- [x] 5.1 Run `cargo fmt --check` and `cargo test --workspace`.
- [x] 5.2 Run `openspec validate add-outline-folding` and resolve every reported issue.
