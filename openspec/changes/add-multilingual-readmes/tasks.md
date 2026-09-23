# Tasks: add-multilingual-readmes

## 1. New standalone editions

Translate the full README content (all sections: install, editing modes, documents and workspace, Markdown editing and preview, themes/languages/preferences, export, Word import, MarkNice publishing, performance, current limitations, development, license) following design.md: UI terminology taken from the corresponding `src/i18n.rs` tables; code, commands, config keys, and product names stay verbatim; config-sample comments are translated. Each new file carries the seven-language navigation line with its own language in bold (English → `README.md#english`, 简体中文 → `README.zh-CN.md`, the rest → their files) and links to `docs/visual-editing-quality.md`.

- [x] 1.1 Create `README.zh-TW.md` (繁體中文) and verify it has the same section headings in the same order as the English edition of `README.md` (compare with `grep '^## ' README.md | head -n -1` against `grep '^## ' README.zh-TW.md`)
- [x] 1.2 Create `README.ja.md` (日本語) and verify section-heading parity with the English edition as in 1.1
- [x] 1.3 Create `README.fr.md` (Français) and verify section-heading parity with the English edition as in 1.1
- [x] 1.4 Create `README.de.md` (Deutsch) and verify section-heading parity with the English edition as in 1.1
- [x] 1.5 Create `README.es.md` (Español) and verify section-heading parity with the English edition as in 1.1

## 2. Navigation updates in existing files

- [x] 2.1 Extend the navigation line of the English edition in `README.md` to list all seven languages (English bold; 简体中文 → `#simplified-chinese`; 繁體中文/日本語/Français/Deutsch/Español → the new files) and verify every linked file exists
- [x] 2.2 Extend the navigation line of the Simplified Chinese edition in `README.md` the same way (English → `#english`; 简体中文 bold) and verify every linked file exists
- [x] 2.3 Extend the navigation line of `README.zh-CN.md` to all seven languages (English → `README.md#english`; 简体中文 bold; keep the trailing 合并版 → `README.md#simplified-chinese` link) and verify every linked file exists

## 3. Consistency and validation

- [x] 3.1 Cross-check every edition against the English source: identical table rows, identical link targets (docs/, LICENSE, releases URL), identical code blocks except translated comments, identical section count — verify by diffing `grep -c '^- '`, `grep -c '^##'`, and code-fence counts per file
- [x] 3.2 Verify UI terms in each edition match `src/i18n.rs` for that language (spot-check menu paths 文件/檔案/ファイル/Fichier/Datei/Archivo and the four view-mode names per language)
- [x] 3.3 Run `openspec validate add-multilingual-readmes --strict` and verify it passes
- [ ] 3.4 During the sync/archive step, update the `project-documentation` capability Purpose in the main spec from "bilingual" wording to "multilingual" wording (delta Purpose is ignored for existing capabilities), and verify the synced main spec shows the "Multilingual project overview" requirement with no remaining "Bilingual" requirement
