## Why

Typora users expect three in-document navigation habits that Markion still lacks: a live `[TOC]` block, hovering a footnote marker to read its definition, and clicking `[text](#heading)` (or a heading `{#id}`) to jump. The sidebar outline already jumps, but that is not the same muscle memory as writing `[TOC]` and hash links in the Markdown.

## What Changes

- A standalone paragraph whose trimmed text is `[TOC]` (case-insensitive) becomes an in-document table of contents. Source stays `[TOC]`. Read, Split Preview, and unfocused Visual Edit render the current outline as clickable headings; a click jumps like the outline panel. Focusing the Visual Edit block reveals the `[TOC]` token so it can be edited or deleted.
- Hovering a resolved footnote reference shows a presentation-only tooltip with the definition text. Click-to-jump icons remain. Missing definitions show no tooltip.
- Heading anchors prefer an authored `{#id}` when present, otherwise a Unicode-preserving slug of the title, uniquified like GitHub (`hello`, `hello-1`). Bare `#anchor` links in Read/Split/Visual Edit jump to that heading instead of calling the platform URL opener.

### Non-goals

Expanding `[TOC]` into a persisted Markdown list, export `--toc` behavior, copying heading permalinks, back-links from footnote definitions, `[toc]:` reference definitions, HTML `<a href="#…">` inside HTML blocks, cross-file `other.md#heading` jumps, or `[TOC]` nested in list items.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: In-document `[TOC]` widget; footnote-reference hover preview; `#anchor` / `{#id}` heading jumps from links and Visual Edit navigation icons.
- `tables-outline`: Outline heading anchors are unique, honor authored heading ids, and are the same ids in-document hash links resolve against.

## Impact

- **Code:** `src/parse.rs` (slugify, unique anchors, `[TOC]` marker, `#fragment` parse); `src/lib.rs` (heading id + uniquify, absorb TOC paragraphs); `src/model.rs` (`PreviewBlock::TableOfContents`, `VisualBlockKind::TableOfContents`, `VisualNavigationTarget::Heading`, optional footnote label on `InlineSpan`); `src/visual.rs` (TOC visual block, hash-link navigation targets); `src/app/preview.rs` / `src/app/editing.rs` (TOC UI, footnote tooltip, hash-link activate); `src/source_mapped.rs` / `src/export.rs` / `src/document_memory.rs` (new preview variant); `docs/visual-editing-quality.md`; README limitation bullets.
- **Invariants:** TOC, footnote tooltip, and hash-link jumps MUST NOT bump document version or invalidate per-version `Arc` caches. `MarkdownDocument.text` stays `[TOC]` until the user edits that token. `crates/*` stay GPUI-free.
- **Tests:** Pure helpers for slugs/TOC/`#fragment`; derivation that `[TOC]` is not a paragraph; GPUI tests that TOC/`#anchor` clicks jump without mutating source and footnote hover is presentation-only.
