## Context

See `proposal.md` for motivation. The GUI-free importer currently walks paragraph children in `render_inline_children`; each `render_run` resolves direct and inherited formatting, escapes its text, and immediately writes opening delimiters, content, and closing delimiters to `ChunkWriter`. Because one run has no knowledge of the next visible run, a bold-to-plain transition can serialize as `**粗体**文字`.

A string replacement over completed Markdown would be unsafe: literal authored asterisks, links, equations, HTML, and already-separated boundaries are indistinguishable from converter-generated strong delimiters at that stage. The decision therefore has to remain inside the DOCX semantic serialization path.

## Goals / Non-Goals

**Goals:**

- Detect a semantic transition from bold content to immediately adjacent non-bold word-like content.
- Insert one U+0020 before serializing the following content, after all closing markers from the prior run.
- Share boundary state across transparent inline containers while resetting it at paragraph and other structural boundaries.
- Keep the implementation linear in the number of inline nodes and GUI-free.

**Non-Goals:**

- A general Markdown formatter or post-import cleanup pass.
- Coalescing every adjacent Word run with equivalent formatting.
- Spacing rules for emphasis, strike, code, links, punctuation, typography, or existing Markdown documents.

## Decisions

### 1. Track typed inline-boundary state during serialization

Introduce paragraph-local boundary state that records whether the last visible text ended with bold formatting and has not yet been separated by another visible boundary. Text-producing render paths provide the state with metadata before writing Markdown:

- whether the current visible text resolves to bold after direct and inherited style resolution;
- the first unescaped source character, if any;
- whether a structural or non-text item separates it from the preceding text.

Before a non-bold item is emitted, insert one ASCII space only when the pending predecessor ended bold and the current first source character satisfies Rust's Unicode-aware `char::is_alphanumeric`. CJK letters, Latin letters, and digits therefore share one rule without a new Unicode dependency. Whitespace, punctuation, combining marks, Markdown delimiters, symbols, and line breaks do not pass the positive word-like test.

The state is passed through recursive `smartTag`, `sdt`, revision, and simple-field traversal so empty or transparent XML wrappers do not hide the adjacency. A visible punctuation run consumes the pending transition without inserting a space, ensuring `**粗体**，文字` does not later become `**粗体**， 文字`. Hyperlink labels participate using their first visible source character even though their serialized Markdown starts with `[`. Images, equations, explicit breaks, and other structural inline output clear the pending adjacency.

Alternative considered: inspect the next sibling from `render_run`. This fails across transparent wrappers, empty runs, fields, and hyperlinks and duplicates traversal rules. Alternative considered: post-process `**` patterns in the final string. This can rewrite authored Markdown and cannot reliably identify formatting provenance, so it is rejected.

### 2. Insert at the start of the following semantic item

The compatibility space is emitted immediately before the following item's opening Markdown/HTML syntax. This guarantees that the prior run has already emitted its full closing sequence. A bold+italic run followed by plain text becomes `***粗斜体*** 文字`; bold combined with safe underline/superscript/subscript HTML likewise closes completely before the space.

Adjacent runs that both resolve bold never trigger the rule because the semantic bold state has not ended. Empty runs and bookmark markers do not trigger or clear it. Paragraph completion discards the state, so no space can cross a paragraph, table-cell, footnote-block, or other block boundary.

Alternative considered: append the space from the bold run after looking ahead. Emitting from the next item keeps output deterministic when intervening empty nodes exist and prevents a trailing space when no following word-like content is ultimately rendered.

### 3. Treat the space as compatibility serialization, not a diagnostic

No per-boundary diagnostic or preference is added. The conversion already normalizes Word's run-oriented representation into Markdown, and the inserted space is the requested portable serialization. Reporting every occurrence would add noise without offering a user decision. Exact-output tests make the normalization observable to maintainers.

### 4. Verify both exact source and parser semantics

Add focused synthetic DOCX fixtures that vary direct/inherited bold state, adjacent empty/wrapper nodes, CJK/Latin/digit starts, ASCII/CJK punctuation, existing spaces, explicit breaks, hyperlinks, and combined styles. Assertions cover exact Markdown and `pulldown-cmark` events so the bold span closes before the following prose. Update existing golden expectations whose bold-to-plain adjacency intentionally changes.

The converter remains a workspace member with no GPUI dependency. The root document model, typing path, document versions, derived `Arc` caches, syntax memoization, cached text handles, persistence, and undo history are not involved.

## Risks / Trade-offs

- [The synthetic space changes visible text compared with the byte-exact DOCX run boundary] → Restrict insertion to an ended bold span followed by a Unicode alphanumeric character with no existing separator; cover every excluded boundary in exact tests.
- [Boundary state can leak across paragraphs or content containers] → Construct/reset it at the inline block entry point and add negative tests across paragraph, line-break, table-cell, and footnote boundaries.
- [Nested styles or hyperlinks can place the space inside closing syntax] → Emit the decision before the next semantic item, after the prior item has completed all delimiters, and assert combined-style/link output exactly.
- [The active base change and this delta can be archived in the wrong order] → Archive or sync `add-builtin-docx-import` first, then validate and archive this additive `docx-import` requirement; otherwise reconcile the deltas before archival.

## Migration Plan

1. Add the boundary-state helper and route existing inline text producers through it without changing their content escaping or formatting resolution.
2. Add exact and parser-semantic regression fixtures, then update intentional golden-output changes.
3. Run the focused importer suite and workspace checks available in the standard development environment.
4. Validate the OpenSpec change. Archive it only after the base `add-builtin-docx-import` capability exists in stable specs or the two changes have been reconciled.

Rollback removes the boundary-state insertion and restores the affected golden outputs. No persisted schema, preference, resource, or document migration is required.
