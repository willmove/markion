# Proposal: fix-visual-edit-mixed-prose-line-breaks

## Why

A CommonMark paragraph written as consecutive lines (soft breaks, no blank separator) renders as multiple lines in Read and Split Preview, but Visual Edit collapses those lines into one whenever the paragraph also contains a link, footnote, inline math, or inline HTML image. That mixed path is the only Visual Edit prose layout that fragments text into a wrapping flex row so navigation icons and atoms can sit as sibling elements; a `\n` fragment there wraps like a space instead of starting a new line. The reported fixture is a heading plus a three-line paragraph that begins with a Markdown link.

## What Changes

- Visual Edit SHALL preserve authored soft and hard line breaks in mixed prose rows (paragraphs, headings, list items, and quoted leaves that already take the fragment layout), matching Read / Split Preview.
- Link / footnote navigation icons, inline math atoms, and inline HTML image atoms SHALL stay on the logical line that owns their construct.
- Regression tests SHALL pin the reported no-blank-line fixture (link + following prose lines + inline code) by asserting distinct layout rows, and SHALL keep ordinary single-line mixed prose wrapping as today.
- The Visual Edit support matrix SHALL name this line-break behavior for mixed fragment layout.

### Non-goals

- No change to the source-mapped projection: SoftBreak / HardBreak already become `"\n"` runs; this is a view-layout fix.
- No change to Read / Split Preview (plain `StyledText` already honors `\n`; math-only preview flex-wrap is out of scope).
- No change to canonical source, parser options, or treating single newlines as paragraph splits.
- No new caret/IME mapping model for the newline byte itself.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: extend *Source-backed Visual Edit mode* so mixed-fragment prose rows honor authored line breaks instead of joining them on one flex line.

## Impact

- **Code**: `src/app/preview.rs` (`visual_text_with_math_element` mixed fragment layout). Tests in `src/app/tests.rs`.
- **Docs**: `docs/visual-editing-quality.md` support-matrix wording for mixed inline constructs.
- **Invariants**: derived Visual Edit blocks remain cached per document version and shared via `Arc`; layout grouping is per-frame presentation only and MUST NOT invalidate those caches or mutate source. No `gpui` dependency in `crates/*`.
- **Compatibility**: presentation-only Visual Edit fix — no file format, settings, or API migration.
