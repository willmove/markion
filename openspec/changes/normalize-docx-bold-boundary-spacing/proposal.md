## Why

The DOCX importer currently serializes adjacent Word runs independently, so a bold run followed immediately by ordinary text can become `**粗体**文字`. Some widely used Markdown editors do not recognize or render that boundary consistently, while `**粗体** 文字` is interoperable and preserves the intended bold scope.

## What Changes

- Insert exactly one ASCII space after an import-generated closing strong-emphasis delimiter when the next source-visible content begins with a word-like Unicode character and the source boundary contains no whitespace.
- Leave the output unchanged when the following content already begins with whitespace, punctuation, a line break, or the end of the paragraph, and do not split consecutive content that remains bold.
- Make the decision from DOCX run semantics before Markdown escaping and delimiter emission, so authored literal asterisks and unrelated Markdown are not rewritten.
- Add focused converter tests for Chinese, Latin, and numeric text; ASCII and CJK punctuation; existing whitespace; line breaks; adjacent style combinations; and parser-visible bold boundaries.

**Non-goals:** Reformatting existing Markdown documents, changing editor bold commands, inserting spaces around italic/strike/code delimiters, or normalizing general DOCX typography and punctuation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `docx-import`: Add an interoperable strong-emphasis boundary rule to the capability currently planned by `add-builtin-docx-import`.

## Impact

- Affects the GUI-free DOCX serializer in `crates/docx-import/src/render.rs` and its semantic tests in `crates/docx-import/src/tests.rs`.
- Does not add a runtime dependency, user-facing setting, diagnostic type, persistence migration, or localization string.
- Keeps the root editor and its per-version derived `Arc` caches, syntax memoization, cached text handles, undo history, and typing path unchanged.
- Depends on `add-builtin-docx-import` establishing the `docx-import` capability; archive that change first or reconcile both deltas before archival.
