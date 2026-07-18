## 1. Affinity-Aware Projection Mapping

- [x] 1.1 Add GPUI-independent projection-boundary candidate and caret-affinity types, plus per-tab ephemeral affinity state that is cleared or revalidated on unambiguous movement and document mutation
- [x] 1.2 Consolidate display-to-source and source-to-display boundary mapping behind one `VisualProjection` API used by pure model code, pointer hit testing, caret painting, and revealed-delimiter Left/Right movement
- [x] 1.3 Add focused tests for opening/closing marker boundaries, nested exact groups, pointer-side resolution, subsequent insertion side, UTF-8/grapheme safety, and unchanged document versions/cached `Arc<VisualBlock>` values during affinity-only changes

## 2. Layout-Aware Visual Navigation

- [x] 2.1 Record compact per-visible-row navigation snapshots from painted wrapped layouts, keyed by document version and projection inputs, without storing layout state in `MarkdownDocument` or notifying on every paint
- [x] 2.2 Add per-tab preferred-x and bounded pending-navigation state, including cancellation on newer input/mutation and one-shot reveal completion for virtualized adjacent rows
- [x] 2.3 Route Visual Edit Up/Down, Select Up/Down, Home, and End through painted-line targets while preserving existing source-line behavior in Edit/Split and explicit Visual Edit source islands
- [x] 2.4 Add pure and GPUI tests for wrapped paragraphs, different line lengths, preferred-x retention, cross-block movement, virtualized off-screen targets, selection extension, marker reveal reflow, and single-caret painting

## 3. Visual IME Composition Fidelity

- [x] 3.1 Pass the active marked range into `build_visual_projection`, reveal an exact containing syntax group when needed, and retain conservative source-island fallback for ambiguous composition mappings
- [x] 3.2 Paint the projected marked range with composition feedback and return requested projected range geometry from `bounds_for_range`, retaining the Visual Edit surface rectangle only as a pre-layout fallback
- [x] 3.3 Implement a per-tab IME capture lifecycle that records one pre-composition undo state, reuses it across intermediate marked-text replacements, and closes or cancels it safely on commit, unmark, focus/mode change, or another command
- [x] 3.4 Add rendered GPUI coverage for CJK text, emoji and combining characters, multiple UTF-16 composition updates, local marker reveal, candidate bounds, commit/cancel, one-step undo/redo, and Read-mode non-mutation

## 4. Semantic Undo Grouping

- [x] 4.1 Add undo capture metadata for contiguous typing, IME composition, and atomic commands while retaining the existing full-plus-diff history representation and history limit
- [x] 4.2 Coalesce temporally and positionally contiguous ordinary input, and terminate groups on caret/selection discontinuity, paste, formatting, structural edits, table commands, mode/tab changes, save/open replacement, and explicit undo/redo
- [x] 4.3 Add history tests for insertion and deletion groups, selection replacement boundaries, timeout boundaries, structural commands, redo fidelity, IME interaction, history limits, UTF-8 edits, and isolation across tabs

## 5. Integration and Verification

- [x] 5.1 Add end-to-end Visual Edit regressions that combine affinity, wrapped navigation, local marker reveal, IME composition, semantic undo, dirty/autosave behavior, and exact canonical Markdown mutations
- [x] 5.2 Verify cursor, affinity, layout, marked-range geometry, and pending-navigation changes do not increment document versions or invalidate preview, outline, stats, highlighting, cached text handles, or visual-block caches
- [x] 5.3 Run `cargo fmt --all -- --check`, focused Visual Edit tests, `cargo test`, and `cargo test --workspace`
- [x] 5.4 Run `openspec validate harden-visual-edit-interaction-core` and resolve all validation errors
