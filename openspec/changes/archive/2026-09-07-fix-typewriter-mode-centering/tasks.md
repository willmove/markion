## 1. Centering State and Geometry

- [x] 1.1 Add pure, unit-tested helpers for viewport-center targets/deltas, scroll-range clamping, pixel tolerance, and presentation-only trailing-space sizing.
- [x] 1.2 Add a versioned per-tab typewriter recenter request with bounded retry/refinement state, stale-target validation, and cancellation when the mode or target surface is no longer active.
- [x] 1.3 Route typing, caret navigation, typewriter enablement, tab/view changes, and layout-affecting reflow through request enqueueing while leaving wheel/scrollbar input alone; remove the hard-newline/fixed-ten-line centering path.

## 2. Source Editor Integration

- [x] 2.1 Reconcile Edit and Split source requests after current wrapped layout is published, using `source_content_y_for_offset`, the current line height, typewriter leading inset, and live `ScrollHandle` viewport/max-offset geometry.
- [x] 2.2 Add typewriter-only source leading and trailing presentation space that follows viewport resize and disappears when disabled without entering document text, measured wrapped-text cache identity, or undo state.
- [x] 2.3 Integrate source recentering with the existing Split Preview editor-driver sync path and add GPUI tests for soft wraps, hard-line navigation/typing, stale-layout deferral, document-tail centering, and follower stability.

## 3. Visual Edit Integration

- [x] 3.1 Extend the existing Visual Edit coarse reveal and bounded measured-caret follow flow with viewport-center refinement when a valid typewriter request is pending, providing leading and trailing presentation space and suppressing coarse reveal for an already measured typing target.
- [x] 3.2 Add GPUI tests for far unmeasured caret jumps, consecutive typing, first/end-of-document centering, view/tab/reflow changes, and preservation of ordinary minimum-distance reveal when typewriter mode is disabled.

## 4. Invariants and Verification

- [x] 4.1 Add regression assertions that typewriter scrolling and trailing space do not change document text, dirty state, version, undo/redo history, cached text handle, or `Arc` identities of already-derived Markdown state.
- [x] 4.2 Run `cargo fmt --check`, the focused root-package typewriter/GPUI tests, `cargo test --workspace`, and `openspec validate fix-typewriter-mode-centering`; resolve every failure without weakening the specified behavior.
- [x] 4.3 Reproduce and eliminate per-keystroke caret/scroll jumps to the viewport top in Edit and Visual Edit, asserting that consecutive typed characters settle at the vertical center without an intermediate coarse reveal displacing an already measured caret.
