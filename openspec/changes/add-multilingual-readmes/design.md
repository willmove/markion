# Design: add-multilingual-readmes

## Context

`README.md` is a combined document: English edition on top (anchored `#english`), Simplified Chinese edition below (anchored `#simplified-chinese`). `README.zh-CN.md` duplicates the Chinese edition as a standalone entry point and adds a "合并版" nav link back to the combined document. The app localizes its UI into seven languages via `src/i18n.rs` (`Language`: En, ZhHans, ZhHant, Ja, Fr, De, Es); each language's display name in the preferences UI is its native name (简体中文, 繁體中文, 日本語, Français, Deutsch, Español). See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**
- Five new standalone editions that are structurally identical to the existing ones (same sections, same order, same linked docs) so future content updates can be applied section-by-section across all seven files.
- A single, predictable navigation convention a reader can rely on from any edition.

**Non-Goals:**
- Translating `docs/*.md` (the editions keep linking to the English docs, as the zh-CN edition already does).
- Translating code blocks, configuration keys, file names, or command lines — these stay verbatim in every edition.
- Restructuring the combined root README into separate files.

## Decisions

- **File naming: `README.zh-TW.md`, `README.ja.md`, `README.fr.md`, `README.de.md`, `README.es.md`.** The existing `README.zh-CN.md` uses a BCP-47-ish region code, so Traditional Chinese pairs with it as `zh-TW` rather than the i18n persistence code `zh-hant`. The other four use the plain ISO 639-1 code. GitHub renders these as plain files; there is no registry to satisfy, so consistency with the existing name wins over matching `Language::code()`.
- **Navigation line: all seven languages in native names, current one in bold, plus the existing "合并版" extra on `README.zh-CN.md` only.** In `README.md`, English and 简体中文 link to the in-document anchors (`#english`, `#simplified-chinese`) and the other five link to their files. In standalone files, English links to `README.md#english`, 简体中文 links to `README.zh-CN.md` (the canonical standalone), and the rest link to their files. The "合并版" affordance stays exclusive to `README.zh-CN.md` because only Chinese exists inside the combined document; extending that concept to five more languages has no target to point at.
- **Terminology source: reuse the app's own translations.** UI element names (menu paths, preference labels, view-mode names) in each edition follow the corresponding `src/i18n.rs` tables, so a reader who switches the app to that language sees the same terms. Prose is translated naturally; product names (Markion, MarkNice, PicGo, GPUI, pulldown-cmark), key bindings, and the TOML config sample stay untranslated, with the config comments translated as the zh-CN edition already does.
- **Spec delta uses REMOVED + ADDED instead of RENAMED + MODIFIED.** An archived precedent (`2026-08-20-commit-visual-edit-to-wysiwyg`) combined RENAMED with MODIFIED under the old header and the rename did not take effect in the main spec — the old requirement name survived. REMOVED (with Reason/Migration) + ADDED makes the name change to "Multilingual project overview" explicit and archive-safe.
- **Capability Purpose follow-up.** Delta Purpose sections are ignored for existing capabilities, so the main spec's "Covers current bilingual project documentation…" Purpose line is updated to "multilingual" wording as an explicit task during the sync/archive step, not by hand-editing specs mid-change.

## Risks / Trade-offs

- [Five translated editions drift from the English source as features change] → Mitigation: keep section structure and ordering identical across editions so diffs stay localizable; the spec's "README claims are checked against the project" scenario now covers every edition, so staleness is a spec violation and surfaces during documentation-touching changes.
- [Machine-grade translation errors in technical prose] → Mitigation: constrain UI terminology to `src/i18n.rs` strings and keep code/config untranslated; each edition is reviewed against the English source section by section during implementation.
- [Longer navigation line wraps awkwardly on narrow views] → Acceptable: it is a single centered paragraph of short native names; GitHub wraps it gracefully.

## Migration Plan

No deployment or rollback concerns: the change adds five files and edits two navigation lines per existing file. Reverting is a file deletion plus reverting those lines.

## Open Questions

None.
