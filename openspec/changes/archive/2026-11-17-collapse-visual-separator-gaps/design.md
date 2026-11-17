## Context

See proposal.md for why. Visual blocks already split source so a content row keeps its own line ending and the following gap is a `Whitespace` row. A single inter-block gap is one newline. Today that row's height is one body line (and a document-end range adds another phantom line). Paragraph spacing is a separate margin on the last row of a paragraph/whitespace run. Read mode never builds these rows; it only applies paragraph spacing between preview blocks.

Derived visual blocks stay cached per document version. The virtual list remeasures a whitespace row when its `height_signature` changes. Typography changes already drop list measurements, so a spacing band inside the row is remeasured without a new document version.

## Goals / Non-Goals

**Goals:**

- Paint a structural separator as paragraph spacing while keeping its source offset clickable and reachable with Up/Down.
- Paint only the newlines beyond that separator, or every newline of a leading or trailing gap, as body lines.
- Stop painting a body line for the empty end-of-document insertion row.

**Non-Goals:**

- A preference, source normalization, or HTML-block margin changes. Raw HTML keeps its existing bottom margin, so an HTML block followed by a separator can still be roomier than Typora by that margin alone.
- Changing Enter/Backspace structure rules.

## Decisions

1. **Classify the gap from its source range.** `start > 0 && end < text.len()` and at least one newline means the gap is between blocks: the first newline is the separator. Otherwise every newline is a body line. An empty range at `text.len()` is the insertion row and has zero body lines. Do not add a phantom line because the range ends at the document end.

   Alternative: delete the row when it is only a separator. Rejected — the caret, click, and arrow paths need a block that owns that newline.

   Alternative: give the separator zero height and reuse the previous block's margin. Rejected for paragraph-to-paragraph gaps, where that previous margin is already zero so the hit target would disappear.

2. **Put the separator in the row's height, not an extra margin.** The row height is `paragraph_spacing + extra_lines * line_height` when the first newline collapses, and `newlines * line_height` otherwise. The collapsed row's body-flow margin is zero so the band does not stack on the margin the same row used to own before a heading. Clicks in the band land on the first newline; clicks below it walk later newlines at body line height. The caret on the separator stays inside the band.

3. **Navigation stops stay one per newline.** Up/Down still enter the row. A single separator is one stop. Further newlines are further stops. The caret at the document end of a non-empty trailing gap shares the last newline's line instead of a phantom line past it. The empty insertion row remains the stop after a block that consumed its own line ending.

4. **`height_signature` stores body-line count, including zero.** The virtual-list clamp no longer forces a minimum of one line, so growing a separator into one visible blank line changes the signature and drops the cached height. Source bytes and the per-version derived cache are untouched.

5. **Typing into a separator may move later content.** The old tests froze the following block because the blank row already reserved a body line. A spacing-only gap does not reserve that line, so the new paragraph pushes content down. Extra authored blank lines still reserve their body lines.

## Risks / Trade-offs

- [HTML block margin plus the separator band is wider than a paragraph-to-paragraph gap] → Accepted. The reported blank line is the body-height row; restyling HTML margins is out of scope.
- [A zero-height end row might not paint a caret] → The caret is painted from the row bounds even when the content height is zero, overflowing into the end padding. If a test shows it missing, give the focused empty row a caret quad without reserving a body line when unfocused.
- [Loose lists lose a full blank line between items] → The separator remains a spacing hit target, matching paragraphs. Additional blank lines stay full lines.

## Migration Plan

No stored data migration. Existing documents keep their newlines; only Visual Edit geometry changes. Rollback is reverting the layout helpers and the signature formula.
