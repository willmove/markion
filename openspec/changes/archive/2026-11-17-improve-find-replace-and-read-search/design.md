## Context

See `proposal.md` — Why. The current overlay keeps `search_query` and `replace_text` as plain strings, renders each value as `"{label}: {value}"`, and redirects platform text input by truncating any trailing IME composition and appending at the end. Only the inline file-name editor has caret and selection state, so search focus does not intercept document navigation, Select All, Enter, or clipboard actions consistently.

Search results are `SearchMatchRange` values over canonical Markdown. Selecting a result writes its source range into the active tab selection and always calls the source-editor scroll path. That works in Edit and Split modes, and partially in Visual Edit through source-backed projection, but Read mode hides the source editor. The preview already exposes virtualized `PreviewBlock` rows, stable per-block text-run identities, selectable plain text for those runs, and list scrolling; it does not consume source search ranges or paint search highlights.

The implementation must preserve the per-document-version `Arc` cache for preview/outline/stats state, the memoized highlighter, the cached source text handle, and virtualized preview rendering. Read-mode search must consume already-derived preview blocks and must not parse Markdown from render or query-edit paths.

Current flow:

```text
platform text input -> append-only String -> source regex scan
                                           |
                                           v
                                source selection + source scroll
                                           |
                         Read mode: hidden and therefore ineffective
```

Target flow:

```text
SearchFieldState -> query/options/domain generation
                           |
              +------------+-------------+
              |                          |
              v                          v
   canonical source matcher    cached preview-run matcher
   Edit/Split/Visual Edit              Read
              |                          |
              +------------+-------------+
                           v
             typed match targets + current identity
                           |
              +------------+-------------+
              v                          v
   editor/visual highlights      preview-run highlights
   + visible-pane reveal         + virtual-list reveal
```

## Goals / Non-Goals

**Goals:**

- Give Find and Replace fields complete single-line caret, selection, clipboard, deletion, keyboard-navigation, pointer, and IME behavior without changing document text while a field is focused.
- Represent source and Read-preview results explicitly so navigation, highlighting, counting, and replacement cannot accidentally use the wrong pane or coordinate system.
- Reuse cached preview blocks and existing preview text-run/rendering machinery while keeping match computation outside render.
- Keep search highlights separate from free-range preview selection so search does not change preview copy semantics.
- Make query, option, document-version, active-tab, and view-mode transitions reset or retain current match state deterministically.

**Non-Goals:**

- Do not turn the search overlay into a general widget framework or retrofit the link editor, file-tree filter, and every redirected input in this change.
- Do not build a flattened cross-block rendered-document index; Read-mode matches remain within one selectable rendered text run, while inline styling inside a rich-text run remains transparent to matching.
- Do not add background indexing, workspace search, search history, whole-word mode, fuzzy matching, or new persistence.
- Do not make rendered formulas or image atoms searchable unless they already expose visible selectable text; link labels are searchable but hidden destinations are not.

## Decisions

### 1. Introduce caret-aware search field state instead of extending plain strings

Each Find and Replace value will use a small `SearchFieldState` containing the buffer, anchor, cursor, and active composition range/length. The state will clamp every endpoint to UTF-8 boundaries and expose selection-aware insert, marked-text replacement, Backspace, Delete, movement, Select All, Cut, Copy, and Paste operations. `SearchField` continues to identify focus; the input bridge routes field-focused key actions before document editing actions.

The rendered field will shape only the buffer, paint its selection and caret, and map pointer x-coordinates to byte positions. An adjacent localized label or icon/tooltip identifies Find versus Replace. Empty fields stay visually empty; guidance is never injected into the buffer or painted as an in-field placeholder.

Rationale: a true field state fixes the interaction model at its source and provides testable Unicode/IME invariants. Reusing the current append-only `active_input_text_mut` path would preserve the exact defects being addressed. Generalizing every redirected input now was considered but rejected because link and file-name workflows have different commit and multi-field rules; a later refactor can extract shared primitives after the search behavior is proven.

### 2. Replace one range type with domain-typed search targets

Search state will distinguish the domain and target coordinate system:

```text
SearchDomain = Source | ReadPreview

SearchTarget =
  Source { range, line, column }
  Preview { block_index, run_id, range_in_run }

SearchResultState =
  Idle | PendingPreview | InvalidPattern(message) | NoMatches | Ready
```

The match collection remains ordered in document order. The current result is owned by the active search generation rather than retained as an unchecked numeric index from a previous query. A generation key includes active tab identity, document version, search domain, query, case sensitivity, and regex mode. Any key change invalidates stale matches and chooses a new current result at or after the source caret or top visible preview row, wrapping when needed.

Rationale: source byte ranges cannot faithfully identify visible Read-mode text, while preview carets cannot drive source replacement. A typed target prevents accidental replacement of preview-only coordinates and makes mode transitions explicit. A single optional source range plus ad-hoc block lookup was rejected because hidden Markdown matches have no exact Read-mode presentation and would recreate the present ambiguity.

### 3. Keep the existing source matcher and add a preview-run matcher over cached blocks

Edit, Split Preview, and Visual Edit retain the existing `SearchOptions` source semantics and replacement engine. Read mode enumerates the selectable runs that are actually rendered for each cached `PreviewBlock`, obtains their visible plain text, and applies the same compiled literal/regex and case-sensitivity semantics independently to each run. Rich text is matched after inline spans are concatenated, so bold, italic, link, and other inline style boundaries do not split a visible phrase. Decorative markers, code line numbers, hidden link targets, Markdown punctuation, image paths, and non-text atoms are excluded.

Code blocks need one canonical search coordinate independent of whether line numbers cause the renderer to split them into line elements. The preview renderer will translate that canonical code-run range into the displayed line fragment when painting, avoiding duplicate results when line numbers are enabled.

Read matching runs only when the preview list reflects the active document version. If asynchronous/debounced preview derivation is temporarily stale, result state becomes `PendingPreview`; stale matches are not presented as current, no synchronous parse is forced, and match computation is retriggered when current preview blocks are installed.

Rationale: the existing preview selection helpers already define which text is visible and selectable, and the preview is virtualized by block. Building a second flattened document string would require separator policy and offset maps, duplicate retained text, permit accidental cross-structure matches, and complicate cache invalidation.

### 4. Paint all matches separately from selection, with one stronger current result

The source editor receives sorted source match ranges and paints subdued background quads in addition to its normal selection. Visual Edit filters source targets to each visible block and maps representable ranges through its existing source-to-display projection; the current source match continues to drive the canonical selection/reveal path so unsupported or temporarily source-revealed constructs remain reachable.

The preview renderer queries search targets by `(block_index, run_id)`, overlays subdued match backgrounds, and paints the current match with a stronger theme-aware color. These ranges are separate from `PreviewSelection`; opening, navigating, and closing Find therefore does not overwrite a user's preview drag selection or change Copy / Copy Markdown / Copy HTML precedence.

The source and preview match vectors stay sorted so a virtualized row can locate only its own ranges without scanning every match in the document. Closing the overlay drops active match vectors and paint state but retains both field buffers and options as today.

Rationale: reusing document or preview selection for every match cannot represent multiple ranges and would break copy/edit semantics. Styling text at derivation time was rejected because search is transient UI state and must not invalidate cached Markdown or syntax-highlight data.

### 5. Centralize current-result initialization, navigation, and replacement transitions

Every successful refresh chooses a valid current target immediately; the UI never displays `0/N` when `N > 0`. Next and previous use one wraparound function for toolbar actions, Enter / Shift+Enter, and F3 shortcuts. Revealing dispatches by target: source results use the visible editor/Visual Edit reveal path, while preview results scroll the preview list directly to the owning block and then rely on its exact range highlight.

Replace current uses the current source target, applies one atomic replacement, refreshes matches for the new document version, and selects the next surviving target at the replacement location. Replace all retains the existing one-snapshot undo behavior and refreshes to an empty or remaining-match state. Invalid, pending, empty-query, and no-match states have no current target and disable replacement actions.

Rationale: retaining an index merely because it is still within the new vector length can associate the UI with an unrelated result after the query or document changes. One transition path prevents toolbar and keyboard behavior from diverging.

### 6. Preserve requested panel form while gating replacement by view mode

Search panel state records whether the user requested Find or Replace independently of whether replacement is currently available. In Read mode the replacement row and mutating actions are unavailable; invoking Replace opens/focuses Find and shows localized guidance outside the field. Entering Read while Replace is open hides the replacement row without discarding its buffer. Returning to an editable mode restores the requested Replace form and recomputes source matches.

Rationale: silently switching view modes or mutating hidden Markdown from a read-only surface violates user intent. Forgetting the requested form would make a temporary mode switch destroy useful UI state.

### 7. Keep validation and control availability in overlay state

The overlay derives summary text, field error treatment, and enabled/disabled actions from `SearchResultState`. Regex compilation errors remain visible in the overlay instead of being overwritten by a later generic no-match summary. Option buttons use explicit active styling plus localized tooltips/accessibility labels; navigation, replacement, and close controls receive equivalent localized labels. Existing hard-coded `No query` and case-sensitivity status strings are removed or routed through `Msg`.

Rationale: the status bar is shared by unrelated operations and is too easy to overwrite. The overlay owns the query and is the stable place to explain why it cannot produce or replace results.

## Risks / Trade-offs

- [Risk] Read-mode counts can temporarily lag while debounced preview blocks are stale. → Represent that interval as localized pending state, suppress stale highlights, and recompute when the current-version block set is installed without forcing a parse.
- [Risk] Large match sets can make per-frame painting expensive. → Keep targets sorted, partition by visible source line or preview `(block, run)`, paint only visible virtualized preview rows, and add a large-match regression test or instrumentation before considering a display cap.
- [Risk] Regex behavior could diverge between source and preview matching. → Compile through one shared matcher helper and test identical literal, case, Unicode, invalid-pattern, and zero-width behavior over both input sources.
- [Risk] Zero-width regular-expression matches have no visible highlight extent and replacement can be surprising. → Preserve the regex engine's ordered matches, render a caret-width current marker where possible, and cover navigation/replacement progress so repeated actions cannot loop forever.
- [Risk] Visual Edit cannot display every source-only match without revealing syntax. → Keep canonical source targets authoritative, use existing source-reveal/projection behavior for the current match, and only promise subdued all-match highlighting where a visible representation exists.
- [Risk] A new search-specific field duplicates primitives from the inline name editor. → Keep UTF-8 boundary helpers small and reusable where safe, but avoid broad input refactoring in the same change.
- [Risk] The overlay can become crowded in narrow windows or verbose languages. → Use a two-row Replace form, compact symbol controls with localized tooltips, responsive wrapping, and external labels that do not consume editable text width.

## Migration Plan

No persisted data or external API migration is required.

1. Introduce and test the new field and typed match state behind the existing actions and shortcuts.
2. Move source search navigation/highlighting to the new state while preserving replacement behavior and undo boundaries.
3. Add Read-preview matching, current-version gating, preview highlighting, and virtual-list reveal.
4. Replace the old field renderer and append-only routing, then add localized overlay states and control availability.
5. Remove obsolete plain-string/current-index paths after behavior tests pass.

Rollback consists of restoring the prior search state and renderer; document and preference formats are unchanged. Replacement mutations remain covered by the existing undo snapshot model throughout rollout.
