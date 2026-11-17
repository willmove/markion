## Context

See proposal.md for motivation. Originally, `slash_query_at` found the marker before the caret but extended its range to the physical line end, so suffix spaces and CRLF `\r` rejected queries and confirmation could consume untyped suffix text. The initial implementation corrected those boundaries, gated structured payloads, and compacted the fourteen block commands. The follow-up adds insertion settings, limits slash headings to H1–H5, and tightens spacing further.

## Goals / Non-Goals

**Goals:** Fix the source-range boundary, retain the existing command/selection transaction, and render a small native GPUI palette using the current theme and icon asset system.

**Non-Goals:** No new document model, parsing pipeline, Source-mode completion, or replica of unsupported reference-app commands.

## Decisions

- Bound the query and replacement to marker-through-caret; preserve all following bytes, including CRLF. Continue accepting ASCII and Chinese markers after line start or whitespace. Reject whitespace and URL/path syntax in the query itself. For standalone templates (code, divider, table), separate remaining same-line prose with the document's line ending so it does not attach to a closing delimiter. Table insertion needs a blank line because GFM also accepts body rows without pipes. Retaining the old line-end boundary cannot distinguish a typed query from an existing paragraph.
- Resolve eligibility against the already cached Visual blocks only after finding a lexical query. Allow ordinary editable block kinds and blank lines; reject source islands and structured editors. This avoids interpreting literal slashes in payloads as block commands without introducing a new full parse.
- Keep the root library's block-command catalog, including H6 for other block-transform consumers, unchanged. An application-level palette action type wraps block commands and adds Link, Formula, and Image settings; its catalog omits H6. Up/Down/Enter navigate the same filtered action collection as the pointer UI.
- Render the five heading buttons and Text in the first six-button row, followed by lists, quote, code, and divider in the second row. Reduce button dimensions to 36×30 pixels and grid gaps to 2 pixels, with a narrower popup. Group Table, Link, Formula, and Image as separated labeled insertion actions below the grid.
- Opening insertion settings validates the captured slash query, closes completion, and initializes the existing editor with the query's source range and version. It does not first delete the query or insert a placeholder. The inline formula editor wraps the confirmed LaTeX payload in math delimiters; link and image settings use their existing serialization/resource paths. Image settings also offer the existing local-file chooser against the captured range. Dismiss the unchanged query when settings are canceled so completion does not immediately reopen.
- Suppress slash/emoji completion while a settings editor owns input. Confirmation is one canonical edit replacing the captured marker/query; cancel leaves source, selection, history and cache identities intact. Stale versions are rejected by existing checked mutation boundaries.
- Data flow: canonical source + collapsed selection → caret-bounded query → cached Visual context check → transient palette state → themed overlay → existing version-checked `BlockEdit` → one undoable source mutation. Opening/filtering/hovering/dismissing do not change document version or derived cache identity; mutations use the normal invalidation path.

## Risks / Trade-offs

- [Compact icons are less self-explanatory] → localized hover labels and selected-command caption; retain explicit Table label.
- [Template insertion before existing prose can join Markdown delimiters] → suffix-aware line separation and exact source/undo regressions.
- [Context filtering can reject boundary positions incorrectly] → resolve using the marker byte inside the owning block; test code, math, tables, front matter, prose, lists, headings, and quotes.

## Migration Plan

No data migration. Ship with the normal application build; reverting the scoped code and icon changes restores the old presentation.
