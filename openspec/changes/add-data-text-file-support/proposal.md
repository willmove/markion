# Proposal: add-data-text-file-support

## Why

Markion's file tree admits a curated plain-text set (`.txt`, `.log`, `.csv`, …) alongside Markdown, but the data/config formats a writing workspace inevitably contains — JSON, YAML, TOML — are rejected everywhere, even though the app configures itself with TOML and Markdown front matter is YAML. Two problems compound this: the open gates disagree (curated text opens from the tree but not via external drag-and-drop or the CLI open intent), and every text document is currently parsed through the Markdown pipeline, so naively admitting data files would misrender them (a TOML `#` comment previews as a heading).

## What Changes

- Introduce a **data text document kind** for `.json`, `.yaml`/`.yml`, and `.toml` files: opens and edits as UTF-8 text in its own tab, but never runs the Markdown pipeline — no pulldown-cmark parse, no preview blocks / outline / Markdown-derived stats, no Visual Edit or Read modes (source editing only), rendered with the code font slot.
- Add **whole-file syntax highlighting** for data documents, resolved by extension through the existing syntect/two-face grammar registry, classified into the existing `HighlightKind` theme-mapped token classes, memoized per document version under the same rules as code-block highlighting.
- **Align the open gates**: the file tree lists data files with a `Data` kind; tree click, external drag-and-drop, the CLI startup open intent, session restore, workspace switching, and the recent-files list all treat curated text and data text files as openable documents (drag-and-drop and the CLI intent also gain the existing curated text extensions, closing today's drift).
- File-tree icons for data files reuse the existing `IconKind::Json` / `IconKind::Config` classification; any new user-facing labels or status strings go through `src/i18n.rs` in every supported language.

## Capabilities

### New Capabilities

- `data-text-editing`: Covers opening, editing, and persisting JSON/YAML/TOML documents as a distinct data-text document kind — source-only editing surface, code font, whole-file grammar-based highlighting, and the same tab isolation, autosave/recovery, and per-version caching rules as other text documents.

### Modified Capabilities

- `workspace`: The file-tree listing requirement gains the data extensions (`.json`, `.yaml`, `.yml`, `.toml`) as a third curated file class; curated plain-text and data files become openable via external drag-and-drop and the CLI open intent (today both are Markdown-only); the session-restore requirement's wording is reconciled to reopen curated text and data files (code already restores curated text, ahead of the spec text).
- `chrome-platform`: The Open Recent requirement's "recent Markdown files" wording becomes "recent documents" — the recent store already records curated text paths opened through File → Open, and data text files join the same flow.

## Impact

- `src/storage/file_tree.rs`: new `DATA_EXTENSIONS` constant, `is_data_path`, `FileTreeFileKind::Data`.
- `src/app/workspace.rs` (`classify_external_drop_path`), `src/app/state.rs` (`StartupOpenIntent::from_path`), `src/app/application.rs` (session-restore and workspace-switch path filters), `src/app/documents.rs` (open flow), `src/app/git_panel.rs` (conflict-view path gate), `src/ui/icon.rs` (tree icon dispatch).
- Editor pipeline: tab state gains a document-kind field; derived Markdown state (preview blocks, outline, stats), Visual Edit, and Read surfaces must not be produced for data documents. The per-version `Arc` cache, memoized-highlighting, and cached-text-handle invariants are preserved — data documents skip derivation entirely rather than recomputing per keystroke; whole-file highlighting is memoized per version.
- `src/highlight.rs` / `crates/markdown`: a whole-file highlight entry point over the existing `LanguageRegistry` (extension-resolved); no new grammar dependencies.
- `src/i18n.rs`: new strings for file-type labels / statuses across all supported languages.
- No changes to storage formats, `session.toml`/`config.toml` schemas, or export pipelines.

**Non-goals:** XML/INI/MDX and general source-code files; data-format validation, formatting, or linting (JSON linter, YAML front-matter editing); a symbol outline for data documents; per-document-kind Format-menu gating changes; any Markdown behavior changes.
