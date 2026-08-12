## 1. Data model & scan classification (`src/storage/file_tree.rs`)

- [x] 1.1 Add `FileTreeFileKind { Markdown, Text }` enum, add `TEXT_EXTENSIONS: &[&str] = &["txt", "text", "log", "csv", "tsv", "org", "rst", "adoc", "asciidoc"]`, and a case-insensitive `is_text_path(&Path) -> bool` helper (ASCII-lowercased, mirroring `is_markdown_path`). Keep `MARKDOWN_EXTENSIONS` and `is_markdown_path` unchanged (drag-drop still uses them).
- [x] 1.2 Replace `FileTreeEntry.is_markdown: bool` with `file_kind: FileTreeFileKind` (meaningful only when `kind == File`); update the struct construction site in `collect_file_tree_entries`.
- [x] 1.3 In `collect_file_tree_entries`, classify each regular file once at scan time into `Markdown` / `Text` / dropped: keep skipping non-text files, but admit the curated plain-text types as `FileTreeFileKind::Text`. No per-frame reclassification.
- [x] 1.4 Remove the empty-folder prune — stop calling `entries.truncate(row_index)` when a directory has no Markdown/Text descendant, so empty and asset-only folders survive. Leave `should_skip_file_tree_path` (directory blacklist + hidden dirs) exactly as-is.

## 2. Migrate `is_markdown` call sites to `file_kind`

- [x] 2.1 In `src/app/root_view.rs`, update the `clickable` predicate (~line 1580) to `kind == File` (both `Markdown` and `Text` are clickable) and update the row-icon lookup to branch on `file_kind`.
- [x] 2.2 In `src/ui/icon.rs` `file_tree_icon`, route `FileTreeFileKind::Text` to a neutral plain-text icon; ensure the asset exists under `assets/icons/` (add one if missing). Markdown keeps its current icon.
- [x] 2.3 Grep for remaining `.is_markdown` field reads and migrate them to `file_kind` matches; confirm no compile errors.

## 3. Opening path (no new document type)

- [x] 3.1 Verify `open_tree_file` → `MarkdownDocument::open` opens a curated plain-text file as UTF-8 without rejecting it; relax or remove any Markdown-only assertion if one exists. Per-document-version derived caches must continue to invalidate on version bump as for any document (no new caching logic).
- [x] 3.2 Leave `handle_external_drop` (`src/app/workspace.rs`) unchanged (Markdown/image-only gate) — add an assertion test that a dropped `.txt` is still rejected by the drop path.

## 4. Localization (`src/i18n.rs`)

- [x] 4.1 Update the empty-state message (`FileTreeEmptyState`, "Open a Markdown file to see it listed here.") and any "Markdown-only" wording to reflect Markdown + plain text, across all locales (En, ZhHans, ZhHant, Ja, Fr, De, Es).
- [x] 4.2 If a distinct status string is needed for opening a plain-text file, add it through `Msg` with translations; otherwise reuse the existing open-status string.

## 5. Tests

- [x] 5.1 Update `scan_lists_only_markdown_files` → assert Markdown **and** curated plain-text files are listed and non-text files (`.png`, `.rs`) remain hidden.
- [x] 5.2 Replace `scan_prunes_folders_without_markdown` with `scan_keeps_empty_folders`: assert empty folders and asset-only (non-text) folders now appear, while blacklisted/hidden directories stay excluded.
- [x] 5.3 Add `is_text_path_recognises_curated_extensions_case_insensitively` unit test next to the existing `is_markdown_path_…` test.
- [x] 5.4 Add an integration test in `src/lib.rs`: clicking/opening a `.txt`/`.log`/`.csv` tree row opens it in a tab (or focuses an existing tab for the same path), and the drag-drop path still rejects non-Markdown text.
- [x] 5.5 Update any `src/app/tests.rs` references to the old `is_markdown` field so the suite compiles and the Markdown-row icon/click behavior is preserved.

## 6. Build, validate, archive

- [x] 6.1 Run `cargo test --workspace` until green.
- [x] 6.2 Run `openspec validate file-tree-text-files-and-empty-folders`; fix any deltas, then archive (`/openspec:archive`) when implementation is complete.
