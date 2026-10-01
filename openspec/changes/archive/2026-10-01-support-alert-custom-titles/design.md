## Context

GFM alert detection is pulldown-cmark's (`BlockQuoteKind`), which requires `[!TYPE]` followed by end of line. A titled marker therefore arrives as a plain quote whose first child paragraph starts with the literal `[!NOTE] Title` line, joined to the next line by a soft break. Visual Edit derives the callout title row from the uncovered gap that starts an alert group. The PDF, DOCX, and LaTeX exporters read preview blocks. HTML export runs `push_html` over the parser events.

## Decisions

### 1. Detect after the quote is built
At `End(BlockQuote)` for a top-level quote that pulldown left untyped, `split_titled_alert` checks the authored first line of the first child paragraph with a strict marker parser: `[!`, one of the five kinds (ASCII case-insensitive), `]`, at least one whitespace character, and non-empty text. It then sets `alert` and `alert_title`, and removes the first rendered line from the paragraph, i.e. the spans up to and including the first `\n`. The paragraph's source range is advanced to the next line's content start, skipping the `>` markers and their spacing. If nothing remains, the paragraph is dropped. The title is the rendered first line with the marker removed, so inline Markdown markers do not appear.

*Alternative:* pre-rewrite the source before parsing. Rejected: it breaks byte-exact ranges.

### 2. Visual Edit needs no new ownership logic
Because the paragraph now starts on the second line, the marker line falls into the leading gap, and the existing alert-group path turns that gap into the `CalloutTitle` row. `CalloutTitle` gains `title: Option<String>` (from `BlockQuote.alert_title` through `VisualLeaf`), which the app renders in place of the kind label. The reveal behavior is unchanged.

### 3. Exports
- PDF IR `Block::Alert` gains `title: Option<String>`. The layout and raster labels use it, falling back to `alert_label(kind)`.
- DOCX: `render_docx_alert` takes the title.
- LaTeX: alerts emit `\textbf{title}\par` first inside `quote`.
- HTML: a pass over the offset events finds `Start(BlockQuote)`. For a pulldown-typed alert, it inserts the title paragraph with the kind label. For an untyped quote whose first paragraph starts with a valid titled marker (checked on the authored source), it retypes the quote, drops the events of the marker line (up to the first soft or hard break, or the paragraph end along with an emptied paragraph), and inserts the title paragraph with the escaped title from that line's text events. Classes come from rewriting the opening tag to `<blockquote class="markdown-alert markdown-alert-<kind>">`.

### 4. Caches
Everything happens inside the existing single parse pass (preview cache) or the export call. No per-frame work is added.

## Risks / Trade-offs

- [The title loses inline formatting] → This is a stated non-goal: plain text keeps the title row simple and the same across exports.
- [Lazy continuation: the title line's paragraph continues without `>`] → The next line start is computed from the source. Without a `>` marker, the skip only consumes leading spaces.
