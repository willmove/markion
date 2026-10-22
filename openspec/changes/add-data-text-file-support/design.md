# Design: add-data-text-file-support

## Context

Today every text document — Markdown and curated plain text alike — is opened through `MarkdownDocument::open` (raw UTF-8, no extension gate) and runs the full Markdown pipeline: pulldown-cmark parse into preview blocks, outline, stats, Visual Edit derivation. The syntect/two-face grammar registry (`src/highlight.rs`, `crates/markdown::highlight::LanguageRegistry`) already resolves languages by name *and by extension*, but is only used for fenced code blocks. The open gates disagree: the file tree and session restore admit curated text, while external drag-and-drop (`classify_external_drop_path`, `src/app/workspace.rs`) and the CLI startup intent (`StartupOpenIntent::from_path`, `src/app/state.rs`) accept Markdown only. The source editor element (`src/app/editor_element.rs`) paints the document as one plain `TextRun`, already splitting extra styled runs for the IME marked range; its wrap-measure cache keys on `(document version, width, font size, line height, font family)`.

## Goals / Non-Goals

**Goals:**
- One shared "openable text document" predicate so every open gate (tree click, drop, CLI, session, workspace switch, git panel) can never drift again.
- A minimal document-kind concept that skips Markdown derivation for data documents without touching the Markdown pipeline's code paths.
- Whole-file highlighting for data documents that reuses the existing registry, scope classifier, and theme token classes, memoized per document version.

**Non-Goals (design-level):**
- No general "code editor" mode, no line-number gutter for data documents, no symbol outline.
- No changes to how Markdown or curated plain-text documents render or parse.
- No new persisted formats; `session.toml` records paths already, so nothing migrates.

## Decisions

### D1: One classifier, one gate predicate, in the storage layer

`src/storage/file_tree.rs` gains:

- `DATA_EXTENSIONS: &[&str] = &["json", "yaml", "yml", "toml"]` + `is_data_path(path)` (same case-insensitive shape as `is_markdown_path` / `is_text_path`).
- `FileTreeFileKind::Data` as a fourth classification branch in the scan.
- `pub enum DocumentKind { Markdown, Data }` with `document_kind_for_path(path) -> DocumentKind` (untitled/no extension → `Markdown`). Curated plain text stays in the `Markdown` kind deliberately: its pipeline is unchanged, and a third kind would buy nothing.
- `pub fn is_openable_text_path(path) -> bool` = `is_markdown_path || is_text_path || is_data_path`.

Every gate is rewired to the shared predicate: `classify_external_drop_path`, `StartupOpenIntent::from_path`, `open_session_path` and the workspace-switch filter in `src/app/application.rs`, the git-panel path gates in `src/app/git_panel.rs`, and the tree-click open path. *Alternative rejected:* adding extensions to `TEXT_EXTENSIONS` and branching on `is_data_path` at each call site — that keeps N call sites deciding kinds independently, which is exactly how the current Markdown-only drift happened.

### D2: Document kind lives on the tab, derived from the path

`DocumentTabState` exposes `kind()` as an on-demand accessor computed by `document_kind_for_path(self.document.path())` — implemented during apply as a getter rather than the originally sketched stored-field-plus-chokepoints, because the path already flows through every mutation point (open, Save As commit, rename/reload remap), so deriving on demand cannot go stale and needs no recompute plumbing. Nothing inside `MarkdownDocument` changes — it remains a UTF-8 text buffer with versioning; the kind is presentation policy. *Alternative rejected:* a field on `MarkdownDocument` — it would leak presentation policy into the model crate and get stale under in-place reloads.

### D3: Data documents skip derivation by never invoking it

The derived-state call sites (preview blocks, outline, stats, Visual Edit blocks) already run after version bumps through a per-version `Arc` cache. For `kind == Data` tabs those calls are skipped entirely, so the caches stay empty and the per-version invariants hold trivially — nothing is recomputed per keystroke. Status-bar line/word counts keep using the existing plain-text computation (`MarkdownDocument::line_count` and the word counter), which is text-derived, not Markdown-derived.

View mode is a single app-level field today (`view_mode: ViewMode`, not per-tab). Data documents **clamp** it at the surface: when the active tab is `Data`, the layout renders the Edit surface regardless of the stored mode, and `set_view_mode` no-ops for non-Edit targets. The stored `view_mode` is never mutated by a Data tab, so switching back to a Markdown tab resumes the previous mode for free — matching the spec scenario without new state.

### D4: Whole-file highlighting reuses the registry, memoized per version

`src/highlight.rs` gains one entry point:

```
highlight_data_document(text: &str, extension: &str) -> Option<Vec<HighlightedSpan>>
```

It resolves the grammar via the existing `LanguageRegistry` extension lookup (JSON/YAML in syntect defaults, TOML in the two-face set), parses the whole text, and maps scope stacks through the existing `classify_scopes` into `HighlightKind` — same classification code path as code blocks, so theme colors come from the same token-class mapping. `None` (no grammar / parse failure) means plain text, per the spec.

Memoization: a per-tab cell keyed by document version (`(version, Arc<Vec<HighlightedSpan>>)` beside `measured_height_cache`), recomputed only on version change — mirroring the code-block memoization contract. Large-file behavior is unchanged from the Markdown parse path (whole-document parse per version; config-size files are the target workload).

### D5: Editor element paints spans as styled runs, code font for data tabs

`editor_element.rs` already builds multiple `TextRun`s (IME marked range splits runs). The run builder gains an input: the optional highlighted spans for the active tab. Pure-function shape:

```
build_text_runs(text, spans: Option<&[HighlightedSpan]>, marked_range: Option<Range>) -> Vec<TextRun>
```

- no spans → today's behavior (plain + marked-range split);
- spans → split at span boundaries, color each run via the theme's `HighlightKind` color mapping used by preview code blocks, then overlay the marked-range split (IME composition must keep its distinct styling).

The wrap-measure path keeps measuring with a single plain run (the existing comment already observes plain and styled runs measure identically); its cache key must switch to the *effective* family. The effective family itself: the editor surface wrapper selects `resolved_font_families.code` when the active tab is `Data`, else `.editor` — one decision point, so line height, measurement, and paint all agree.

### D6: File tree and icons

The scan classifies `Data` after `Text` (disjoint extension lists). Tree-row icons dispatch through the existing extension-based `icon_for` (`IconKind::Json` for `.json`, `IconKind::Config` for `.toml`/`.yaml`), which already exists and is tested — no new assets.

### D7: i18n and Save/Save As stay as-is

The open flows reuse existing localized statuses (the "unsupported file" and open statuses already exist), so the target is **zero new strings**; if implementation surfaces a genuinely new user-visible message, it goes through `src/i18n.rs` for every supported language. Save As keeps the current document's extension in the suggested name; the filter profile stays the shared text-document profile — identical to how curated `.txt` Save As behaves today, so data files add no new inconsistency.

## Risks / Trade-offs

- [IME marked-range runs interact badly with span-colored runs] → `build_text_runs` is a pure function; unit-test span+marked-range compositions directly (the repo's established pure-fn test style).
- [A gate is missed and drift returns] → the shared predicate is the only public gate API; tests assert each call site routes through it, plus the existing model-level open tests grow JSON/YAML/TOML cases.
- [Highlight colors diverge from code-block colors] → both paths consume `classify_scopes` + the same theme token-class mapping; a test asserts the shared mapping is used (no second color table).
- [Very large data files pay a full syntect parse per version] → same cost model as the existing full-document Markdown parse per version; acceptable and consistent; no new size limit is introduced.
- [`DocumentKind` goes stale after an in-place path change] → kind recomputation is funneled into the few tab-(re)construction chokepoints, each covered by a test (open, Save As, rename remap).

## Migration Plan

No persisted-format changes: `session.toml` already records every tab path; data files simply become valid entries. Deploy is code-only; rollback is reverting the commits. A session recorded with data-file tabs under the new build opens those paths as plain untitled-ignored entries under an old build (missing-path skip handles it gracefully in the other direction).

## Open Questions

None blocking. Line-number gutter for data documents was considered and left out; it can be added later without touching this design's surfaces.
