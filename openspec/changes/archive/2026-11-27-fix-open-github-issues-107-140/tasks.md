## 1. Editing commands (#107–#110)

- [x] 1.1 Fix #107 by dispatching Enter according to the focused search-overlay control, and verify with app tests that focused Replace Current/Replace All perform one undoable replacement while Find/Replace fields still navigate matches.
- [x] 1.2 Fix #108 by accepting tab whitespace after ordered-list delimiters, and verify Source and Visual Edit split `7.\t...` into steps 7/8/9 with correct undo/redo and unchanged space-marker behavior.
- [x] 1.3 Fix #109 by extending emoji completion over the recognized existing suffix, and verify mid-token completion leaves no old letters/colon while an independently authored following colon is preserved.
- [x] 1.4 Fix #110 by retaining exactly delimited display math as an editable semantic block when its payload is invalid, and verify deleting a brace keeps the formula editor/local error with one-step undo and versioned derived caches.

## 2. Parsing, menus, and navigation (#111–#118)

- [x] 2.1 Fix #111 by excluding escaped braces from naive balance validation, and verify `\left\{x\right.` and `\left\lbrace x\right.` render equivalently in Visual Edit, Read, and Split while genuinely invalid TeX still errors.
- [x] 2.2 Fix #112 by using document-level reference definitions for Visual Edit links, and verify paragraph-adjacent pseudo-definitions and continued destinations resolve identically in Visual Edit and Read.
- [x] 2.3 Fix #113 by adding the existing Markdown export action to both Export menu surfaces with localization, and verify an unsaved buffer exports to a new `.md` path without changing the original path, dirty state, or disk bytes.
- [x] 2.4 Fix #114 by remapping the complete selected range after Visual block drag/drop, and verify dragging two paragraphs in either direction preserves their selection so the next keyboard Move acts on both; include LF and CRLF.
- [x] 2.5 Fix #115 by revealing the source caret/selection endpoint after Page Up/Page Down, and verify repeated paging and Shift+paging update both byte selection and source scroll with Typewriter Mode off.
- [x] 2.6 Fix #116 by carrying pending Visual page distance across layout snapshots, and verify eight Page Downs plus reverse/selection paging retain viewport-sized steps in long list and paragraph documents.
- [x] 2.7 Fix #117 by routing Undo/Redo/Copy/Cut to the focused inline name editor, and verify document text, selection, clipboard, and undo stacks stay isolated while filename editing and rename confirmation still work.
- [x] 2.8 Fix #118 by supplying relevant original reference definitions when converting selected visible blocks to HTML, and verify reference links retain label and destination without copying unrelated visible paragraphs.

## 3. Export and preview fidelity (#119–#126)

- [x] 3.1 Fix #119 by embedding local-image alt text as PDF accessibility descriptions, and verify two alt texts produce corresponding structure/alt metadata with unchanged image content and unchanged missing-image fallback.
- [x] 3.2 Fix #120 by preserving significant non-breaking spaces during preview cleanup, and verify standalone NBSP paragraphs and leading NBSP runs affect Read/Split layout as in both HTML exports.
- [x] 3.3 Fix #121 by capturing resize measurements from the dragged image wrapper, and verify dragging the second of two differently sized images matches the single-image control and updates only its width attribute.
- [x] 3.4 Fix #122 by revealing folded code containing either endpoint of the current match and scrolling to its active end, and verify cross-block matches expand only required blocks without source/history mutation.
- [x] 3.5 Fix #123 by double-click selecting the visible word across revealed inline syntax, and verify bold words copy/replace correctly with syntax-hidden both off and on, including formatting inside a visible word.
- [x] 3.6 Fix #124 by exposing definition-list term/definition text to Read Find, and verify plain, case-sensitive, and regex searches visit both rendered rows without changing source or history.
- [x] 3.7 Fix #125 by running the normal current-result reveal when pending Read blocks become ready, and verify switching from Source with Find open immediately reveals the initial match without requiring F3.
- [x] 3.8 Fix #126 by retaining source offsets for fragments in unfocused multiline table cells, and verify clicks on either line insert at the exact Markdown position after visiting another cell while padding clicks retain existing behavior.

## 4. Sessions, history, code copy, and documents (#127–#134)

- [x] 4.1 Fix #127 by snapshotting outgoing workspace tabs before removal and ordering root changes before restored-tab focus persistence, and verify repeated North/South switches retain each root's clean tab list and active path.
- [x] 4.2 Fix #128 by consuming the initial-tab replacement decision independently of newly restored tabs, and verify two paths restore in order with remembered active focus while an unrelated dirty tab remains open and unchanged.
- [x] 4.3 Fix #129 by carrying each history entry's historical path into detail/comparison requests, and verify pure rename and rename-plus-edit fixtures show the original patch and correct current comparison without mutating Git or editor state.
- [x] 4.4 Fix #130 by retaining parsed trailing newlines in preview code payloads used by Copy, and verify Read/Split distinguish zero, one, and two trailing blank lines without altering source or history.
- [x] 4.5 Fix #131 by removing only the opening fence's CommonMark indentation margin from Visual code Copy, and verify indented fences match Read/Split while intentional relative indentation and final newlines survive.
- [x] 4.6 Fix #132 by propagating resolved ordered-list starts into DOCX numbering, and verify numbering XML gives 5/6 for top-level, nested, and prose-separated lists while normal 1/2 lists remain unchanged.
- [x] 4.7 Fix #133 by keeping enclosing ordered groups open across nested bullets, and verify DOCX numbering XML yields top-level 1/2/3 around a nested bullet while genuinely separated lists restart correctly.
- [x] 4.8 Fix #134 by excluding only a leading UTF-8 BOM from Markdown interpretation while preserving it on save, and verify Read/Visual/outline match the unsigned control and interior U+FEFF plus original bytes remain intact.

## 5. Document state, serialization, and PDF shaping (#135–#140)

- [x] 5.1 Fix #135 by making close compare current content with the saved baseline when deciding whether to prompt, and verify edited-back and undo/redo routes to identical bytes behave consistently while genuinely dirty content still prompts.
- [x] 5.2 Fix #136 by replacing the appropriate image-destination range with exactly one angle-bracket pair, and verify URL-only edits to spaced filenames work from bracketed and unbracketed sources with titles/attributes intact.
- [x] 5.3 Fix #137 by escaping entity-opening ampersands in serialized image titles, and verify alt-only and newly typed title edits preserve literal `&copy;` through save/reparse without changing ordinary ampersands or quotes incorrectly.
- [x] 5.4 Fix #138 by establishing standalone block boundaries before slash-inserted delimiters, and verify Divider on an empty line creates a rule rather than Setext H2 and Code Block after prose cannot swallow following content; include LF/CRLF and undo.
- [x] 5.5 Fix #139 by deriving backward word boundaries from whole-line Unicode segmentation, and verify contractions, curly apostrophes, decimals, separate punctuation, and sentence boundaries navigate/select/delete correctly in Source and Visual Edit.
- [x] 5.6 Fix #140 by applying PDF mark placement offsets once at the intended scale, and verify shaped combining grave positions match font references at multiple sizes for `m̀` and `ọ̀` while advances, text extraction, and precomposed controls remain correct.

## 6. Final verification

- [x] 6.1 Review all tasks and diffs in issue order, update any stale comments/documentation touched by the fixes, and verify no unrelated user changes or active OpenSpec tasks were modified.
- [x] 6.2 Run `cargo fmt --all -- --check`, `git diff --check`, `cargo test`, and `cargo test --workspace`; verify all pass and report any platform-only scenarios that could not be exercised on Windows.
