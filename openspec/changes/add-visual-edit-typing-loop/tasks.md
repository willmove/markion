## 1. Auto-pair preference

- [x] 1.1 Add `markdown_auto_pair: bool` (default `true`) to `AppPreferences` / `PreferencesFile`, tolerate missing or non-boolean as `true`, include it in reset, and verify preference load/save/reset tests cover the new key
- [x] 1.2 Expose a General-tab toggle wired through i18n in all seven languages, and verify toggling does not increment document version while `cargo test` preference/i18n cases pass

## 2. Markdown auto-pair

- [x] 2.1 Implement opener/wrap/skip-closer/Backspace-unwrap for `*`, `_`, `` ` ``, `$`, `(`, `[`, `{`, `"`, `'` at the shared text-input boundary, skipping IME marked ranges, backslash-escaped characters, and fenced-code / math / diagram payload editors, and verify pure tests for pair, wrap, skip, unwrap, and suppression
- [x] 2.2 Honor `markdown_auto_pair` on the source editor, Visual Edit prose, and Visual Edit table cells, and verify GPUI or document tests that a typed `*` with the preference off inserts only `*` and that one Undo reverses a pair

## 3. Hide block markers after Enter

- [x] 3.1 Assert unfocused heading/list/task/quote rows hide structural prefixes after Enter or a caret move, without rewriting source, and verify Visual Edit tests for `## Title` + Enter, list continue, and click-away (version-stable hide)
- [x] 3.2 If a just-typed ATX/list/task/quote line still paints marker characters as prose when unfocused, fix classification/projection so the next derivation shows the rendered row, and verify the typed-heading scenario in the markdown-editing delta

## 4. Clickable task-list checkboxes

- [x] 4.1 Hit-test the Visual Edit task glyph column and toggle `[ ]` ↔ `[x]` (`[X]` off → `[ ]`) as one checked source mutation when the prefix is hidden, and verify model/GPUI tests for check, uncheck, undo, and revealed-prefix click-through
- [x] 4.2 Leave Read and Split Preview checkboxes inert, and verify pointer tests on those surfaces do not dirty the document
- [x] 4.3 Close roadmap gap 7 in `docs/visual-editing-quality.md` (clickable checkbox = rendered WYSIWYG) and verify the matrix no longer lists task-list checkbox click as an open gap

## 5. Emoji `:` completer

- [x] 5.1 Expose a GPUI-free queryable shortcode table from the existing `emoji_for_shortcode` set (modest common-name expansion allowed), and verify prefix-filter unit tests including empty/no-match and charset rules
- [x] 5.2 Add per-tab emoji-query state and a slash-palette-like overlay (i18n title/empty copy) that opens on word-boundary `:`, ignores `12:30` / `://`, yields to an open slash palette, and dismisses on Escape without a version bump, verified by GPUI/state tests
- [x] 5.3 Confirm (Enter/Tab/click) replaces `:query` with `:name:` as one undoable edit so preview/Visual Edit render the glyph, and verify undo plus IME-composition suppression tests

## 6. Verification

- [ ] 6.1 Run `cargo fmt --check` and `cargo test --workspace`, fixing pairing, checkbox, emoji, preference, or i18n regressions while leaving per-version `Arc` derived caches untouched on interaction-only paths
- [ ] 6.2 Run `openspec validate add-visual-edit-typing-loop` and reconcile tasks with the proposal, design, and delta specs
