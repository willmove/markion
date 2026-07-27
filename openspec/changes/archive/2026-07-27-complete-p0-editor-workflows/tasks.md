## 1. Persistence foundation

- [x] 1.1 Add GPUI-free atomic same-directory write/replace helpers with cleanup and replacement tests.
- [x] 1.2 Add content-confirmed disk identities to `MarkdownDocument`, capture them on open/save, and return typed external-change conflicts without clearing dirty/path state.
- [x] 1.3 Route explicit Save, Save As, export-to-Markdown, session writes, and recovery writes through the appropriate atomic helper.
- [x] 1.4 Add per-tab external-change state, periodic checks, clean-tab reload, and dirty/deleted-file conflict actions for Reload, Overwrite, and Save a Copy.

## 2. Recovery and session safety

- [x] 2.1 Introduce stable atomic recovery-v2 snapshots carrying original path and disk identity while retaining legacy v1 loading.
- [x] 2.2 Write recovery before named-document autosave, retire it only after successful durable save/discard, and prevent autosave across a conflict.
- [x] 2.3 Restore recovery alongside the persisted session without replacing diverged/newer disk content, with focused startup/session tests.

## 3. Image resources

- [x] 3.1 Add image-format recognition, safe asset-directory/name/link generation, content reuse/collision handling, and filesystem unit tests.
- [x] 3.2 Route clipboard image entries and dropped image files into resource import and one undoable Markdown insertion; keep Markdown-file drop opening intact.
- [x] 3.3 Add exact inline-image source parsing and one-command image replacement plus alt/title/width/alignment transformations.
- [x] 3.4 Render width/alignment metadata and an explicit missing-resource placeholder in Preview and Visual Edit, with edit/replace affordances for exact images.

## 4. Contextual formatting and links

- [x] 4.1 Add pure exact inline-link parsing/serialization and atomic label/URL/title mutation tests, including escaping and UTF-8 ranges.
- [x] 4.2 Add Visual Edit selection-contextual Bold, Italic, Inline Code, and Link controls without changing document version on presentation-only interaction.
- [x] 4.3 Add a focused visual link editor overlay for new and existing exact inline links, with confirm/cancel, keyboard, selection, undo, IME, and ambiguity regressions.

## 5. Product integration and verification

- [x] 5.1 Add every new string to all supported localization catalogs and validate catalog completeness.
- [x] 5.2 Update the maintained Visual Edit support/evidence documentation for image and link mutation ownership.
- [x] 5.3 Run focused model/storage/GPUI tests, `cargo fmt --check`, `cargo test --workspace`, the repository quality script, and strict `openspec validate complete-p0-editor-workflows`; resolve all failures.
