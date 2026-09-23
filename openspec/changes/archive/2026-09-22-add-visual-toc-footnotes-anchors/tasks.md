## 1. Heading anchors

- [x] 1.1 Keep Unicode letters/digits in `slugify`, uniquify duplicate ids (`hello`, `hello-1`), prefer pulldown `{#id}`, and add `heading_offset_for_anchor`
- [x] 1.2 Verify unit tests for CJK titles, authored ids, duplicate titles, and empty titles using `section`

## 2. Hash-link navigation

- [x] 2.1 Parse bare `#fragment` destinations (with percent-decoding) into `VisualNavigationTarget::Heading` and jump via `navigate_to_outline_heading`
- [x] 2.2 Intercept Read/Split link activation so `#anchor` does not call `open_url` when the heading exists
- [x] 2.3 Verify a GPUI `#hello` click jumps without bumping document version

## 3. In-document TOC

- [x] 3.1 Absorb a standalone `[TOC]` / `[toc]` paragraph (including quote children) into `PreviewBlock::TableOfContents` and map it to `VisualBlockKind::TableOfContents`
- [x] 3.2 Render a nested clickable outline in Read/Split/unfocused Visual Edit; reveal the `[TOC]` token when the Visual Edit block owns the caret
- [x] 3.3 Verify derivation (no leftover paragraph, list-item `[TOC]` unchanged) and a GPUI TOC click that jumps without mutating source

## 4. Footnote hover

- [x] 4.1 Attach a footnote label on `FootnoteReference` spans/runs and show a presentation-only tooltip with the definition text
- [x] 4.2 Verify hover wiring does not bump document version and unresolved labels have no tooltip

## 5. Coverage of new preview variant

- [x] 5.1 Update `source_mapped`, export, and `document_memory` matches for `TableOfContents`
- [x] 5.2 Update `docs/visual-editing-quality.md` and README limitation bullets

## 6. Verification

- [x] 6.1 Run `cargo fmt --all -- --check`, `openspec validate add-visual-toc-footnotes-anchors`, and targeted tests (fix regressions in TOC/footnote/anchor tests only)
