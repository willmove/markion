## Context

See proposal.md. The sidebar outline already lists headings and `navigate_to_outline_heading` jumps without mutating source. Preview links always call `cx.open_url`. Visual Edit navigation icons only handle `Url` and `Footnote`. `[TOC]` is an ordinary paragraph. Footnote references are superscript with a jump icon but no hover preview. `Heading.anchor` is `slugify(title)` with ASCII-only letters, so CJK titles get empty ids, and pulldown `ENABLE_HEADING_ATTRIBUTES` ids are ignored.

Cached-per-version Markdown must stay intact: TOC rendering reads `document.outline()`; hover and jumps are presentation-only.

## Goals / Non-Goals

**Goals:**

- Live in-document TOC from a `[TOC]` paragraph token.
- Footnote hover tooltip from existing definition text.
- In-document `#anchor` jumps using unique, Unicode-preserving heading ids (authored `{#id}` wins).

**Non-Goals:**

- Persisting generated TOC lists into source.
- Changing export `--toc` flags.
- Copy-permalink heading chrome.
- List-item `[TOC]`, HTML-block anchors, or cross-file fragments.

## Decisions

### `[TOC]` is a preview/visual block, not expanded Markdown

After `derive_preview_and_outline` sorts blocks, a Paragraph whose trimmed plaintext matches `[toc]` case-insensitively (and is exactly that token) becomes `PreviewBlock::TableOfContents { source_range }`. Recurse into quote children. List items stay paragraphs.

Read/Split paint a nested heading list from the per-version outline cache. Clicks call `navigate_to_outline_heading`. Visual Edit paints the same list while the caret is outside the block; when the block owns the caret, the `[TOC]` source token is revealed above the list so Delete / typing still target canonical bytes. Showing or clicking the widget MUST NOT increment document version.

Alternative considered: leave a Paragraph and special-case in the view. Rejected: export, memory, and coverage matches would still treat it as prose.

### Heading ids: authored, then Unicode slug, then uniquify

Use pulldown’s `Tag::Heading { id, .. }` when `Some`. Otherwise `slugify(title)` keeping Unicode alphanumeric characters (not only ASCII) and collapsing other runs to `-`. Empty results become `section`. Duplicates get GitHub-style suffixes (`hello`, `hello-1`). `MarkdownDocument::heading_offset_for_anchor` returns the first matching outline offset.

Bare destinations that parse as `#fragment` (optional percent-decoding) become `VisualNavigationTarget::Heading`. Other URLs stay `Url`. Preview `SelectablePreviewText` and Visual Edit icons share `activate_document_or_external_link`.

### Footnote hover is presentation-only

`InlineSpan` carries `footnote: Option<String>` for `FootnoteReference` events so `^1^` superscript is not mistaken for a note. Hover shows `document.footnotes()` text for that label in a tooltip (same overlay family as tab/search tooltips). No tooltip when the definition is missing. Click-to-jump icons stay.

## Risks / Trade-offs

- **`[TOC]` is Typora-specific** → Other CommonMark tools show the token as a paragraph; Markion documents remain valid Markdown.
- **Unicode slugs differ from the previous ASCII-only `anchor` field** → Outline tests that hardcode ASCII slugs stay valid; CJK headings become jumpable, which is the point.
- **Focused Visual Edit TOC reveals source** → Matches other widgets (images, HTML); the generated list is still shown so navigation remains available.

## Migration

None. Existing documents without `[TOC]` or hash links are unchanged. Duplicate ASCII slugs that previously collided now uniquify; the first heading keeps the unsuffixed id.
