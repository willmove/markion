## Context

See `proposal.md` for motivation. The current status bar is a single 28px row rendered in `src/app/root_view.rs` with one formatted string containing the document title, dirty/save state, and `MarkionApp::status`. `MarkdownDocument` already exposes version-cached `DocumentStats` with character and whitespace-delimited word counts, while each `EditorTab` owns its selection direction and exposes the active caret offset. `text_util::line_column_at` already converts a byte offset to one-based UTF-8-safe line and column values.

The new branch item introduces filesystem-derived state that can change independently of the Markdown document. Repository I/O must therefore be isolated from rendering, text input, and the document's per-version derived caches. The implementation must remain cross-platform and must not add GPUI dependencies to workspace member crates.

## Goals / Non-Goals

**Goals:**

- Build one testable status-context snapshot for the active tab and render it beside, not instead of, existing feedback.
- Reuse cached document metrics and the tab's active caret semantics.
- Discover ordinary repositories, nested repositories, submodules, and linked worktrees without requiring a `git` executable.
- Refresh branch state asynchronously and reject stale results after context changes.
- Keep the status bar single-line, compact, localized, and stable across view modes.

**Non-Goals:**

- Maintaining an in-memory Git index, watching working-tree dirtiness, or exposing Git actions.
- Introducing incremental word segmentation or locale-specific word-breaking rules.
- Persisting status-bar configuration or adding interactive status items.
- Moving cursor or Git state into `MarkdownDocument` or any GPUI-free workspace crate.

## Decisions

### 1. Project a dedicated status-context snapshot from existing active state

Add a small presentation model, such as `StatusBarContext`, containing document counts, optional caret line/column, and an optional branch label. A pure helper will build it from the active document, view mode, active caret offset, and cached Git result. Rendering will format each field through `t`/`tf` message keys rather than assemble hard-coded English.

The data flow is:

```text
document edit -> document version -> cached DocumentStats ----+
caret/selection/view-mode change -> active caret -> line/col --+-> StatusBarContext -> status row
document/workspace path change -> background Git lookup -------+
periodic HEAD refresh -> generation-checked cached branch ------+
```

The snapshot is presentation-only and is rebuilt cheaply when GPUI renders. It does not become another persisted or canonical document model.

Alternative considered: formatting every item directly inside the GPUI element tree. Rejected because a pure snapshot provides a single place to test view-mode, selection-direction, and missing-branch behavior without depending on layout internals.

### 2. Reuse document-version metrics and compute caret location independently

Character and word values come from `MarkdownDocument::stats()`, preserving the existing definition and version cache. The status-bar helper must call it once per rendered snapshot and must not independently scan the full document. Caret location comes from `EditorTab::cursor_offset()` and the existing one-based line/column helper. It is included for Edit, Visual Edit, and Split Preview, and omitted for Read.

Metrics remain keyed only by the document text version. Cursor movement, selection reversal, focus, view-mode changes, and Git refreshes do not invalidate them. Git state likewise stays outside undo snapshots and all Markdown-derived caches.

Alternative considered: add a second count cache to application state. Rejected because it would duplicate `DocumentStats`, require another invalidation path, and risk disagreement with existing statistics.

### 3. Resolve symbolic Git HEAD directly on a background executor

Use a root-crate helper that starts from the active saved document's parent, falling back to an explicitly established workspace for an unsaved document. It walks ancestors for `.git`, supports both a `.git` directory and a `.git` indirection file (`gitdir: ...`), then reads the resolved `HEAD`. A value beginning with `ref: refs/heads/` yields the complete branch name after that prefix; detached HEAD, malformed metadata, permissions failures, and no repository all yield no branch.

All walking and reads run on GPUI's background executor. App state stores the requested context path, the cached optional branch, the resolved HEAD path when available, and a monotonically increasing generation. A landing result is accepted only when its generation and requested context still match. Changing the active tab's repository context or the workspace schedules a new lookup. A low-frequency background refresh of the resolved HEAD detects branch switches made while Markion remains open; it updates UI state only when the value changes.

Alternative considered: shell out to `git branch --show-current`. Rejected because it makes the feature depend on an installed executable and subprocess latency. Alternative considered: add `git2` or another Git library. Rejected for this read-only symbolic-HEAD need because it increases the cross-platform build and packaging surface; direct metadata parsing covers the required working-tree and linked-worktree cases.

### 4. Use two status-bar regions with bounded branch text

Keep the existing 28px single-row container. The left region remains flexible and contains the existing document/save/transient message, clipped to available width. The right region contains non-wrapping context items separated by consistent spacing or separators. The branch item gets a maximum width and clips long names so it cannot displace all feedback; counts and caret use compact localized templates. Missing optional items are not replaced by placeholders.

This preserves the transient status channel while preventing persistent metrics from being concatenated into an untestable monolithic string. It also avoids increasing window chrome height.

Alternative considered: replace transient feedback with metrics whenever the app is idle. Rejected because operation failures and confirmations must remain visible and should not flicker as status context changes.

### 5. Extend the exhaustive localization catalog

Add compact message keys for branch, characters, words, and line/column formatting in every supported language. The branch name and numeric values are interpolation arguments; only labels and punctuation conventions are localized. Existing catalog-completeness tests remain the enforcement mechanism.

## Risks / Trade-offs

- [Risk] Calling the complete document statistics API from a now-always-visible surface can expose its full-version computation cost on very large files. → Reuse the existing version cache, call it only once per snapshot, add a regression test for repeated same-version reads, and profile before considering a separate incremental metric in a later change.
- [Risk] Repository metadata can change or disappear between discovery and read. → Treat every lookup as fallible, return no branch, and retry on the next context or periodic refresh without replacing user-facing operation feedback.
- [Risk] A slow lookup can land after a tab or workspace switch. → Tag every request with a generation and context path and discard mismatched results.
- [Risk] Direct Git metadata parsing does not cover unusual repository extensions that do not expose a symbolic `HEAD` through standard `.git` metadata. → Fail closed by omitting the optional branch item; keep the resolver isolated so a Git library can replace it later without changing the status-bar contract.
- [Risk] Long localized labels or branch names can crowd transient feedback. → Keep compact templates, bound and clip branch text, prevent wrapping, and give the existing feedback region flexible remaining width.

## Migration Plan

No persisted data or user migration is required. Introduce the pure projection and resolver tests first, then app-level async cache state, localization entries, and the two-region renderer. Rollback consists of removing the new app state and renderer items; document data, preferences, and Git metadata are never modified.
