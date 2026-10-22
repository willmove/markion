# Tasks: add-data-text-file-support

## 1. Classifier and file tree

- [x] 1.1 In `src/storage/file_tree.rs`: add `DATA_EXTENSIONS` (`json`, `yaml`, `yml`, `toml`), `is_data_path`, `DocumentKind { Markdown, Data }` with `document_kind_for_path`, `is_openable_text_path`, and the `FileTreeFileKind::Data` scan branch (classified after `Text`; lists stay disjoint). Unit-test case-insensitive classification, disjointness from Markdown/text lists, and that a scan lists `.json`/`.yaml`/`.yml`/`.toml` rows as `Data`. Verify with `cargo test file_tree`.
- [x] 1.2 Route file-tree row icons for `Data` entries through the existing extension-based `icon_for` (`.json` → `IconKind::Json`, `.toml`/`.yaml` → `IconKind::Config`) without adding icon assets. Extend the icon classification test with `.yaml`. Verify with `cargo test icon`.

## 2. Open-gate alignment

- [x] 2.1 Rework `classify_external_drop_path` (`src/app/workspace.rs`) to open documents for every `is_openable_text_path` file (Markdown, curated text, data). Update `drop_filter_opens_only_real_markdown_files` to assert the mixed drop opens `.md`/`.txt`/`.json` and still skips other extensions and directories. Verify with `cargo test drop_filter`.
- [x] 2.2 Extend `StartupOpenIntent::from_path` (`src/app/state.rs`) to accept `is_openable_text_path` files as `File` intents. Unit-test that a `.toml` CLI path resolves to a `File` intent, an unsupported extension still reports `UnsupportedFile`. Verify with `cargo test startup`.
- [x] 2.3 Switch the session-restore open path and the workspace-switch surviving-path filter (`src/app/application.rs`) to `is_openable_text_path`. Add a model-level test that `open_session_path` opens a `.json`/`.txt` path and still rejects an unsupported one. Verify with `cargo test session`.
- [x] 2.4 Switch the git-panel path gates (`src/app/git_panel.rs`) to `is_openable_text_path` so data files join Markdown/text in conflict viewing. Verify with `cargo test git_panel`.

## 3. Document kind on tabs

- [x] 3.1 Add `kind: DocumentKind` to the tab state, recomputed at the tab (re)construction chokepoints (open, Save As commit, rename/reload remap, recovery restore). Unit-test that a tab built for a `.json` path is `Data`, an untitled tab is `Markdown`, and a Save As to a `.json`/`.md` name reclassifies the tab in place. Verify with `cargo test document_kind`.
- [x] 3.2 Clamp view modes for `Data` tabs: `set_view_mode` no-ops for non-Edit targets while a `Data` tab is active, and the surface layout renders the Edit pane regardless of the stored mode, without mutating the app-level `view_mode`. Unit-test the clamp decision as a pure function. Verify with `cargo test view_mode`.

## 4. Markdown pipeline skip

- [x] 4.1 Skip preview-block, outline, stats, and Visual Edit derivation for `Data` tabs so their per-version `Arc` caches stay empty (never recomputed per keystroke). Model-level test: a `Data` tab's derived caches are empty and the status-bar counts still come from plain text. Verify with `cargo test derived` plus the existing derivation tests staying green.

## 5. Whole-file highlighting and rendering

- [x] 5.1 Add `highlight_data_document(text, extension)` in `src/highlight.rs`: grammar resolved by extension through the existing `LanguageRegistry`, scopes mapped by the existing `classify_scopes` into `HighlightKind`; `None` on no-grammar/parse-failure. Unit-test non-empty classified spans for JSON and TOML samples and `None` for an unresolvable extension. Verify with `cargo test highlight`.
- [x] 5.2 Add the memoized per-version span cache to the tab state (`(version, Arc<Vec<HighlightedSpan>>)` beside `measured_height_cache`) and unit-test that renders across one version reuse the memoized spans and an edit recomputes once for the new version. Verify with `cargo test highlight_memo`.
- [x] 5.3 Implement `build_text_runs(text, spans, marked_range)` as a pure function in `src/app/editor_element.rs`: span-split colored runs overlaid with the IME marked-range split; no spans keeps today's runs. Unit-test span+marked-range compositions. Verify with `cargo test text_runs`.
- [x] 5.4 Wire rendering: the editor element consumes the tab's memoized spans via `build_text_runs` using the existing theme `HighlightKind` color mapping (no second color table), the surface wrapper selects `resolved_font_families.code` for `Data` tabs, and the wrap-measure cache key uses the effective family. Verify with `cargo test editor_element` and a manual smoke pass.

## 6. Integration and verification

- [ ] 6.1 Manual GUI smoke: open `.json`/`.yaml`/`.toml` from the tree, via external drop, and via a CLI path; confirm source-only surface with code font and whole-file colors, view-mode shortcuts stay on the source surface, outline/preview absent, edits mark dirty and autosave, Save As suggests the current extension, and switching to a Markdown tab restores its previous view mode. Record the pass in this change's notes.
- [x] 6.2 Run the full workspace suite (`cargo test --workspace`) and confirm every crate is green before archiving.
