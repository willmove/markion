## Context

See `proposal.md - Why` for motivation. The relevant current state (all in `src/storage/file_tree.rs` unless noted):

- `MARKDOWN_EXTENSIONS = ["md", "markdown", "mdown"]` (line 18) is the single source of the file-type gate. `collect_file_tree_entries` (lines 303-308) drops every non-Markdown regular file.
- Empty folders are pruned: a directory row is pushed tentatively, recursion runs, and if no Markdown descendant is found the row is `entries.truncate(row_index)`-removed (lines 282-300).
- `should_skip_file_tree_path` (lines 323-357) keeps a hard-coded directory blacklist (VCS / build / cache / IDE dirs) plus any hidden directory (`name.starts_with('.')`).
- `FileTreeEntry` (lines 32-45) carries `kind: {Directory, File}` and `is_markdown: bool`, classified once at scan time.
- Opening: `open_tree_file` → `MarkdownDocument::open` reads raw UTF-8 bytes and does **not** re-check the extension, so the pipeline can already ingest non-`.md` files.
- Rendering: `clickable = kind == File && is_markdown` (`src/app/root_view.rs:~1580`); rows capped at 300/frame; icons via `file_tree_icon` (`src/ui/icon.rs:162`).
- Caching invariants: `FileTree` is rebuilt on demand by a background scan and is independent of the per-document-version derived Markdown caches (preview blocks / outline / stats, shared via `Arc`). Tree membership never touches those caches.

## Goals / Non-Goals

**Goals:**
- A single, scan-time classification of regular files into Markdown / PlainText / Hidden, driven by a curated extension allowlist.
- Empty folders survive in the tree while the directory blacklist stays fully in effect.
- Plain-text files are clickable and open through the existing document pipeline with no new document type.
- Zero impact on the per-document-version caching invariants and the 300-row render cap.

**Non-Goals** (design-level, on top of the proposal's non-goals):
- No new "plain-text document mode", monospace renderer, or per-type syntax highlighting. Text files render through the existing CommonMark preview pipeline.
- No content sniffing — classification is purely by file extension.
- No `.gitignore` engine and no Preferences toggle.

## Decisions

### D1. Curated extension allowlist, classified once at scan time
Introduce a constant alongside `MARKDOWN_EXTENSIONS`:

```
TEXT_EXTENSIONS (non-Markdown plain text): txt, text, log, csv, tsv, org, rst, adoc, asciidoc
```

`MARKDOWN_EXTENSIONS` stays unchanged (it's still used by the drag-and-drop gate `is_markdown_path` and to mark a file as Markdown). At scan time each regular file is classified exactly once into one of `Markdown`, `PlainText`, or dropped. Store the result on the entry so the renderer never re-derives it per frame (perf invariant).

- *Why not content sniffing:* extension-based gating is what the codebase already does; sniffing adds I/O and ambiguity (a `.log` that happens to contain `#` is still a log).
- *Why not a user-configurable list now:* deferred to a future change (proposal non-goal); a hard-coded curated list ships value immediately and keeps scope tight.

### D2. Data model — replace `is_markdown: bool` with a small file-kind enum
Replace `FileTreeEntry.is_markdown: bool` with `file_kind: FileTreeFileKind` where `FileTreeFileKind { Markdown, Text }` (only applied when `kind == File`; directories ignore it). This is richer than two booleans with no extra storage cost, and it makes the icon + clickability decisions exhaustive.

- `clickable` becomes `kind == File` (both `Markdown` and `Text` are clickable) — simpler than the old `&& is_markdown`.
- Icon: `Markdown` → existing Markdown icon; `Text` → new neutral plain-text icon.
- *Alternative considered:* keep `is_markdown` and add `is_text`. Rejected as more error-prone (two bools invite a "neither" state that should never exist for a listed file).
- *Migration:* `is_markdown` is read in ~3 call sites (`root_view.rs`, tests). Update them to match on `file_kind`.

### D3. Stop pruning empty folders; keep the directory blacklist
Remove the "recurse, then `truncate` if no Markdown found" branch. A directory row stays once pushed. `should_skip_file_tree_path` is untouched, so `.git`/`target`/`node_modules`/etc. and hidden directories remain excluded.

- A folder containing only non-text files (e.g. only `.png`) is still skipped because those files never produce entries and the folder is... now **kept** (it exists on disk). This is the intended new behavior — the tree mirrors real disk structure for non-ignored paths.
- *Trade-off:* a project with many asset-only folders will see more folder rows. Mitigation: the 300-row cap and one-level expand still bound the view; users collapse what they don't need. Acceptable for a text editor's workspace.

### D4. Open text files through the existing pipeline, no new document type
`open_tree_file` already calls `MarkdownDocument::open`, which reads UTF-8 bytes regardless of extension. A `.txt`/`.log`/`.csv` therefore opens as a document whose source is the raw text; the preview pipeline renders it as CommonMark (plain text is a benign subset — a CSV just shows as lines).

- For `.csv`/`.tsv` this means no native table rendering unless the file already uses Markdown pipe tables; that's acceptable and explicitly a non-goal here.
- `handle_external_drop` (`src/app/workspace.rs:~343`) is **not** broadened — drag-drop keeps its Markdown/image-only gate (proposal non-goal). Only the in-tree click path opens text files.
- *Risk:* a `.log` line beginning with `#` renders as an H1 in preview. Acceptable for v1; a future plain-text mode could address it.

### D5. Data flow & caching impact (per the design rule)
```
background scan
  └─ collect_file_tree_entries
       ├─ directory?  → push row (always; no truncate), recurse
       ├─ file?       → classify by extension → Markdown | Text | drop
       └─ push row with file_kind
  → FileTree { entries } stored in app state (rebuilt on refresh/create/delete)
       └─ renderer: filtered_visible_file_tree_entries (300-row cap) → rows
                                                    ↓ click (Markdown or Text)
       └─ open_tree_file → MarkdownDocument::open → versioned doc
                                                    ↓ per-version
       └─ derived MD caches (preview/outline/stats) via Arc  ← UNCHANGED
```
The per-document-version derived caches are keyed by document identity and version, not by tree membership, so admitting text files into the tree does not perturb them. Opening a text file follows the same version-bump → cache-invalidation path as any Markdown file. The only classification work added is at scan time (already a background task), keeping the per-frame render path O(visible rows).

## Risks / Trade-offs

- **Noisier tree in asset-heavy projects** (D3) → mitigated by the 300-row cap, one-level expand, and the unchanged blacklist; users collapse unwanted folders.
- **MD preview renders plain text slightly transformed** (D4, e.g. `#` lines, smart-quote punctuation) → acceptable for v1; future plain-text mode is explicitly deferred.
- **Curated list may miss a user's preferred extension** (D1) → ship a sensible default; a future Preferences toggle can make it user-configurable without redesign.
- **`is_markdown` callers must all migrate** (D2) → small, mechanical; covered by unit tests that assert Markdown rows keep their icon/click behavior.

## Migration Plan

1. Land `FileTreeFileKind` + classification + empty-folder change behind the existing scan path; update the ~3 `is_markdown` call sites.
2. Add the neutral text icon; broaden `clickable`.
3. Update i18n empty-state + any "Markdown-only" wording.
4. Update/extend tests (see tasks.md).
5. No on-disk format or config migration — this is purely a scan/render behavior change. Rollback is `git revert`; no persistent state to repair.

## Open Questions

- Whether `.csv`/`.tsv` belong in the v1 curated list or should wait for a future native-table view. (Does not change the approach; only the contents of `TEXT_EXTENSIONS`. Default: include them — they are plain text.)
