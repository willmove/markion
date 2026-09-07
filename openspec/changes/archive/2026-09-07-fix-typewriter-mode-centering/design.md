## Context

The typewriter preference, command, status feedback, and persistence are already wired. The failure is in presentation geometry. `DocumentTabState::scroll_editor_typewriter_to_offset` counts hard newlines, multiplies by one line height, and subtracts ten lines. That calculation neither represents a soft-wrapped caret row nor the actual viewport center. It is also invoked before an edited document has produced current-version wrapped layout and updated scroll extents. In Visual Edit, the same call changes the hidden source-editor `ScrollHandle`; the visible `ListState` continues to use ordinary minimum-distance caret reveal.

Markion now has the geometry needed to fix this without reparsing Markdown. Source layout already publishes versioned wrapped lines, `line_tops`, per-line heights, and `source_content_y_for_offset`. The source `ScrollHandle` exposes viewport bounds and maximum offset. Visual Edit already publishes screen-space caret bounds, performs coarse reveal for unmeasured virtual rows, follows measured caret geometry for bounded frames, and includes a presentation-only trailing spacer. The current source surface has no equivalent leading boundary space, and the current Visual Edit flow can still run a coarse reveal during typing even when the active row was already measured; both behaviors can visibly pull the caret toward the viewport top before centering refinement.

The change crosses input, render, and virtual-list state, but remains presentation-only. It must not invalidate or duplicate the per-document-version Markdown caches, syntax highlighting cache, shared text handle, or undo history.

## Goals / Non-Goals

**Goals:**

- Center the active measured caret row in the viewport after typing, caret navigation, enabling typewriter mode, entering an editable view, changing tabs, and geometry-affecting reflow.
- Use the visible surface's native geometry: wrapped source-layout coordinates for Edit/Split and measured caret bounds plus `ListState` for Visual Edit.
- Make deferred work explicit so centering waits for current document/layout/scroll geometry and cannot apply a stale request to another tab or document version.
- Allow the first and last editable rows to remain centered through presentation-only leading and trailing space.
- Keep consecutive typing visually stable by bypassing coarse reveal when current measured caret geometry can be refined directly.
- Preserve ordinary caret reveal, manual scrolling, preference persistence, and Split Preview sync behavior outside a pending typewriter recenter.

**Non-Goals:**

- Animated or smoothed scrolling, configurable center bands, or changes to F8/preferences UI.
- Typewriter behavior in Read mode or changes to focus mode.
- Replacing the source editor, Visual Edit virtualization, or sync-scroll mapping.
- Mutating Markdown text or changing derived-state/cache ownership.

## Decisions

### 1. Represent centering as a versioned per-tab presentation request

Caret-changing actions and typewriter-relevant transitions will enqueue a small per-tab request instead of immediately writing a guessed scroll offset. A request identifies the document instance/version, active caret offset, and target editing surface. Render reconciliation consumes it only when that same target still owns current geometry. A changed document, tab, view, or newer caret request supersedes stale work.

The request remains pending when geometry is unavailable or scroll extents normalize during the first layout, and schedules a bounded follow-up frame. It is cleared when the target is within a small pixel tolerance, the current scroll bounds have clamped the target, typewriter mode is disabled, or the target is no longer editable. User wheel/scrollbar input alone does not create a request, so manual inspection away from the caret is not immediately undone; the next typing or caret-navigation action recenters.

Alternative considered: keep calling `set_offset` synchronously from every action. This retains the current race with stale wrapped layout and pre-edit maximum offsets, so it cannot reliably handle reflow or newly appended content.

### 2. Center source editing from measured wrapped-layout coordinates

For Edit and the source pane of Split Preview, reconciliation obtains the caret row's content-space Y coordinate from current `SourceLayoutKey`/wrapped-line data and obtains the viewport height and maximum scroll from `ScrollHandle`. The target scroll is:

`caret_row_center_y - viewport_height / 2`

clamped to the handle's valid scroll range. The calculation uses the caret's actual wrapped-row position and current line height, so hard lines, soft wraps, font changes, and pane-width changes share one path. The old hard-newline/fixed-ten-line helper is removed or reduced to the new geometry helper.

Source boundary scroll room is derived from the measured viewport and applied only as editor presentation padding while typewriter mode is active. Equal leading and trailing space surrounds the measured document content, and source offset-to-Y/Y-to-offset mapping accounts for the leading inset without including it in the wrapped-text cache key or document data. The padding is large enough for the first and final caret rows to reach the center; normal scroll extents return when the mode is disabled.

Alternative considered: retain a constant ten-line band. It is not a vertical center, changes apparent position with window size, and cannot map soft-wrapped rows correctly.

### 3. Extend Visual Edit's existing two-phase reveal instead of adding a second scroll system

Visual Edit keeps its existing coarse block reveal only when the caret's virtual row has not been measured. Consecutive typing on an already measured row preserves the existing list anchor and proceeds directly to measured centering refinement, preventing the coarse reveal from temporarily pinning that row to the viewport top. Once `visual_caret_bounds` is available, typewriter reconciliation computes the delta between the caret bounds' vertical center and `visual_list.viewport_bounds()` center, then applies that delta relative to the list's current logical top. The existing bounded caret-follow frames provide the post-layout refinement needed after a splice or reflow.

Presentation-only leading and trailing virtual spacers provide document-boundary scroll room. Typewriter mode may size/refine them from the current viewport, but the spacers stay outside `visual_list_blocks` and the derived Markdown cache, and block-index mapping accounts for the leading spacer. When typewriter mode is off, the current inset-based, minimum-distance reveal function remains unchanged.

Alternative considered: map the Visual Edit source offset into the hidden source editor and copy its pixel scroll. Source and rendered block heights differ and virtual rows may be unmeasured, so that mapping would be visibly inaccurate and would bypass the established list refinement path.

### 4. Recenter on geometry transitions and coordinate with sync scroll

Besides text and navigation actions, a pending request is created when typewriter mode becomes active, the active document/view changes into Edit, Split, or Visual Edit, or editor width/font metrics invalidate the relevant layout. This covers first render, tab switching, window/pane resize, view changes, and typography reflow without continuously forcing scroll on unrelated frames.

A source centering write marks the editor as the Split Preview sync driver exactly once for that request. Existing sync-scroll reconciliation then maps the newly centered source viewport into the preview; preview follower normalization must not be interpreted as a new typewriter request. Visual Edit remains outside split sync as today.

### 5. Test geometry separately from GPUI lifecycle integration

Pure helpers will cover center-delta/target clamping, invalid/zero geometry, and boundary behavior. GPUI tests will cover current-version deferral, soft-wrapped source rows, caret navigation and typing, source end padding, Visual Edit coarse-to-measured centering, mode-off behavior, tab/view/reflow transitions, and Split sync coordination. Tests will assert that document text/version/dirty state, undo history, and `Arc` identities of already-derived state do not change when padding or centering is applied.

## Data Flow and Cache Boundaries

1. Input, toggle, tab/view transition, or reflow invalidation records a per-tab typewriter request; document mutation continues through the existing checked mutation boundary.
2. Source `EditorElement` or Visual Edit rows measure and publish presentation geometry for the current document/version.
3. The visible surface reconciles the newest request against its viewport and native scroll state, performs a clamped scroll, and requests at most the bounded refinement frames needed for normalized geometry.
4. Split sync observes a source scroll as its existing editor driver; no Markdown parse, preview-block rebuild, syntax-highlight recomputation, text-handle replacement, or undo snapshot is caused by the centering step itself.

## Risks / Trade-offs

- **[Risk] Render-time reconciliation can create notify/scroll feedback loops.** → Clear satisfied or clamped requests, compare targets with a pixel tolerance, and cap normalization retries/follow frames.
- **[Risk] A stale request can move the wrong tab after an asynchronous render.** → Bind requests to document identity/version, caret offset, and surface; replace rather than queue requests.
- **[Risk] Typewriter padding changes scrollbar extent, thumb size, or source/list index mapping.** → Apply it only while the mode is active, derive it from the live viewport, and keep explicit presentation-to-document coordinate/index transforms; cover enable/disable and resize in integration tests.
- **[Risk] Visual virtual rows may not be measured when the caret jumps far away.** → Retain coarse item pin/reveal first, then center from the measured caret in bounded follow-up frames.
- **[Risk] Split sync may fight the typewriter scroll.** → Treat the editor as the sole driver for the recenter and reuse existing expected-follower suppression.
- **[Trade-off] Centering the first row requires visible blank space above the document.** → Accept that space only while typewriter mode is active because stable, always-centered caret placement is the mode's defining behavior.

## Migration Plan

No data migration is required. Implement the presentation-state and geometry changes behind the existing boolean preference, run the focused GPUI tests and `cargo test --workspace`, then ship normally. Rollback consists of reverting the code change; persisted preferences and documents remain compatible because their schemas do not change.

## Open Questions

None. The existing typewriter preference is interpreted consistently across every editable surface, with boundary clamping and no animation.
