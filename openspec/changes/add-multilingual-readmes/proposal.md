# Proposal: add-multilingual-readmes

## Why

Markion's UI ships in seven languages (English, 简体中文, 繁體中文, 日本語, Français, Deutsch, Español), but the repository README exists only in English and Simplified Chinese. Readers in the other five supported UI languages get no native-language project overview, and the `project-documentation` spec still mandates a bilingual-only structure, so adding editions requires a spec-level change.

## What Changes

- Add five standalone README editions mirroring the current English/Simplified Chinese content: `README.zh-TW.md` (繁體中文), `README.ja.md` (日本語), `README.fr.md` (Français), `README.de.md` (Deutsch), `README.es.md` (Español).
- Extend the in-document language navigation in `README.md` (both the English and the Simplified Chinese edition) and in `README.zh-CN.md` so every edition links to every other edition; each standalone file marks its own language as the active one.
- Keep `README.md` as the combined English + 简体中文 document (unchanged structure); the five new languages are standalone files only, not additional sections of the root README.
- Translate all sections (install, editing modes, workspace, Markdown editing and preview, themes/languages/preferences, export, Word import, MarkNice publishing, performance, limitations, development, license) with terminology consistent with the app's own `src/i18n.rs` strings for each language.
- Update the `project-documentation` spec: the bilingual requirement becomes a multilingual requirement covering seven editions and their cross-linking.

Non-goals: changing any README's factual content beyond translation and navigation; translating `docs/*` or other documentation; changing application code, packaging, or `src/i18n.rs`. This change touches none of the cached-per-version Markdown invariants.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-documentation`: the "Bilingual project overview" requirement expands to a multilingual overview — the combined root README (English + 简体中文), the existing standalone Simplified Chinese README, and five new standalone editions (繁體中文, 日本語, Français, Deutsch, Español), all with equivalent coverage and complete cross-language navigation.

## Impact

- Documentation: `README.md`, `README.zh-CN.md` (navigation lines only), plus new `README.zh-TW.md`, `README.ja.md`, `README.fr.md`, `README.de.md`, `README.es.md`.
- Specs: `openspec/specs/project-documentation/spec.md` updated via this change's delta on archive.
- No Rust code, APIs, dependencies, persisted formats, release packaging, or performance invariants are affected.
