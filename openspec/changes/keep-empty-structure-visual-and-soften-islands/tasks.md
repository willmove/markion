## 1. Keep empty ATX headings in the derived model

- [x] 1.1 Add failing pure tests that `##`, `###`, `###     `, `#`, and `######` (with and without a trailing newline) derive as `PreviewBlock::Heading` / `VisualBlockKind::Heading` with a source range covering the line, `source_island.is_none()`, and a heading `block_prefix`. Cover the Format/slash serialization `## ` as well.
- [x] 1.2 Stop dropping empty headings in `push_nonempty_block` (`src/parse.rs`) and any matching full-document flush in `src/lib.rs` / incremental path in `src/source_mapped.rs`. Continue dropping empty paragraphs. Make the tests from 1.1 pass without changing Visual Edit view chrome yet.

## 2. Keep empty list items in the derived model

- [x] 2.1 Add failing pure tests that `- `, `* `, `1. `, and `1)` empty items derive as list-item visual blocks with no Unsupported island, plus an empty task item `- [ ] ` that already survives derivation and must not be an island.
- [x] 2.2 Stop dropping empty unordered/ordered items in `flush_list_item` and the matching full-document list flush. Confirm Enter-on-empty-list-item still exits the list (`is_empty_list_marker` / structural Enter tests).

## 3. Prefix reveal for empty rows and prefix-end caret

- [x] 3.1 In `build_visual_projection`, treat heading/list `block_prefix` as active when the caret or a selection endpoint is inside the prefix **or at `prefix.end`**, and always reveal the prefix when the block owns the caret and has no visible content runs.
- [x] 3.2 Add projection tests: empty `###` with caret on the line reveals `###`/`### `; caret at the first title character of `# Hello` reveals `# `; caret inside `Hello` keeps the prefix hidden.

## 4. Stop islanding rendered empty rows

- [x] 4.1 Tighten `focused_conservative` in `visual_block_view` so empty `editable_runs` alone does not route Heading, ListItem, or other rendered kinds to `visual_source_island_view`. Keep `always_source` for FrontMatter / Code / residual Unsupported.
- [x] 4.2 Confirm empty heading and empty list render arms paint through `VisualEditableText` with the revealed prefix so the caret and IME bounds exist without island chrome. Add a GPUI test: caret on `###     ` / `- ` is not a source-island box; typing inserts after the prefix; document version/caches unchanged until text changes.

## 5. Unfocused placeholder height in Visual Edit, Read, and Split Preview

- [x] 5.1 Paint empty headings with heading metrics and empty list items with marker + empty content in Read and Split Preview (stop skipping empty `text` in those arms). Visual Edit unfocused rows keep the same reserved height.
- [x] 5.2 Add tests that an empty ATX heading and an empty list item occupy a heading/list row in preview/read derivation (not omitted), and that outline still exposes the empty heading as a jump target.

## 6. Lightweight chrome for remaining islands

- [x] 6.1 Restyle `visual_source_island_view`: tight padding, left accent or hairline, faint background, code-slot font, no large `p_3` bordered card. Apply to FrontMatter, unclosed/indented code islands, and residual Unsupported that is not empty structure.
- [x] 6.2 Add a regression test that YAML front matter and an unclosed fence still map to source-island kinds and remain source-editable. Manual visual check of chrome is listed in 8.x.

## 7. Coverage matrix and structural regressions

- [x] 7.1 Update `docs/visual-editing-quality.md`: empty ATX headings and empty list items are rendered + prefix-reveal; remove empty list items from the roadmap; note remaining islands use lightweight chrome.
- [x] 7.2 Keep existing heading Enter (new paragraph, prefix not copied) and empty-list Enter (exit list) tests passing. Add a slash/block-transform test that turning an empty row into Heading 2 stays a visual heading.

## 8. Validation

- [x] 8.1 Run `cargo test --workspace` (preserve per-version cache invariants; no reparse on caret/prefix reveal).
- [x] 8.2 Run `openspec validate keep-empty-structure-visual-and-soften-islands` and resolve any reported inconsistencies.

## 9. Manual verification (GUI — defer to release QA if needed)

- [x] 9.1 In Visual Edit, type `##` then space on a new line: heading-sized row, `## ` visible, no gray card; type a title and the hashes hide once the caret is in the title. _(Deferred to release QA: covered by unit/GPUI tests; visual chrome check tracked by the release process.)_
- [x] 9.2 Delete a heading’s title down to `###     `: row stays a heading placeholder, not an island. Repeat with `- ` for a list item. _(Deferred to release QA.)_
- [x] 9.3 Switch to Read / Split Preview on a document that contains empty `##` and `- ` lines: rows still occupy heading/list height. _(Deferred to release QA: derivation now keeps these rows; preview arms paint heading/list metrics.)_
- [x] 9.4 Confirm YAML front matter and an unclosed fence still look like source (monospace, distinct) but without the old padded bordered poster. _(Deferred to release QA.)_
