## Why

DOCX export currently offers no user-facing options at all: the save dialog only picks a path, and the app always calls the engine with `ExportOptions::default()`. After the fidelity and engine-pipeline changes (`improve-docx-fallback-fidelity`, `improve-docx-content-fidelity`, `improve-docx-engine-pipeline`), several meaningful choices exist — page size and margins, table of contents, image embedding policy, reference template — but users have no way to reach them. Mainstream tools (Typora, Pandoc front-ends, WPS) expose exactly these choices at export time.

## What Changes

- Extend the DOCX export flow with a small options step: page size (A4/Letter/Legal), table of contents on/off (engine path), and image embedding policy (embed local images vs. text fallback), with sensible defaults matching current post-P0/P1/P2 behavior.
- Persist the last-used DOCX export options in the config file (`[export.docx]` section) so repeat exports keep the user's choices.
- Wire the chosen options through `ExportOptions` to both the pandoc engine path and the built-in fallback (page size/margins already constants in the fallback; image policy toggles the embedding step).
- Harden the DOCX test story: package-structure validation, an engine/fallback consistency smoke check (same fixture, both paths produce openable packages), and updated docs (`docs/faq.md`).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `export`: gains a requirement for user-facing DOCX export options (page size, TOC, image policy) with persistence of last-used choices.

## Impact

- `src/app/documents.rs`: the DOCX export dialog flow gains an options step before/alongside the save-path prompt.
- `src/model.rs`, `src/storage/`: `[export.docx]` config section and persistence of last-used options.
- `src/export.rs`, `crates/export/`: options threading to both backends.
- `src/i18n.rs`: new strings for the options UI (en + zh).
- `docs/faq.md`: document the new options.

Non-goals: a general export-preferences redesign for other formats, font pickers in the UI (fonts remain template-driven via reference-doc/styles.xml), per-export watermark/header/footer editing.
