## Context

See proposal.md. `split_terminal_blank_row` removes LF/CRLF from a final heading/paragraph range. The resulting whitespace row owns the offset before the line ending. Its mapping also clamps EOF onto the last newline and maps CRLF clicks to LF, allowing insertion inside a CRLF pair.

## Goals / Non-Goals

Keep source ranges contiguous and lossless while assigning caret positions to actual source lines. Preserve existing list/quote projections and audit their boundary behavior. No parallel editable document or full reparse on cursor movement.

## Decisions

Data flow: canonical mutation/version -> cached visual blocks -> row ownership -> projection/whitespace line mapping -> paint, hit testing, navigation, IME bounds.

Retain terminal line endings with their original paragraph/heading and create a zero-length EOF whitespace row when needed. A nonempty tail gap counts its final EOF line explicitly. Use one whitespace line mapping for height, caret, pointer, and navigation, with CRLF treated as one ending. Prefer an exact-start row over an EOF-inclusive previous row. This avoids a special rendering-only caret offset that would leave pointer and keyboard behavior inconsistent.

## Risks / Trade-offs

- Existing tests encode the old terminal range split -> replace only assertions contradicted by the corrected source-line contract and add painted-geometry/input tests first.
- Quotes and list containers own some internal blank lines -> preserve their projections and test them separately.
- Zero-length EOF rows affect virtualization identity -> include accurate line counts in cached height signatures and exercise repeated Enter/Backspace.

## Migration Plan

No data migration. Roll back the scoped implementation if necessary. Keep the change unarchived until implementation and validation are complete.

## Related audit findings

- Mixed prose fragments used independent inclusive source ranges for caret ownership. Hidden closing link syntax can lie outside every fragment, while shared endpoints belong to two fragments. Resolve the canonical cursor through the full projection and let exactly one fragment claim the resulting visible source boundary. Unfocused table fragments never claim a caret.
- Bare list markers (`-`, `+`, `*`, `1.`, `1)`) are valid empty CommonMark items, but prefix extraction required trailing whitespace. Accept marker-only lines, treating CRLF atomically, so progressive reveal provides an editable caret position.
- Vertical navigation selected the first inclusive line window at a shared endpoint. At a list's trailing blank line this skipped the list text on Up. Choose the candidate line closest to the actual painted caret position; keep all prose fragment navigation windows while retaining one caret owner.
- Quoted prose and quoted lists also need an EOF insertion row after an authored terminal newline. That row is outside the quote, since its source line has no quote prefix. Existing quoted-content tests now separately assert the exact empty EOF row before checking their original quote invariants.
- Quoted headings were emitted at top level by preview derivation. Their uncovered `> ` prefix became a phantom whitespace row, moving the heading/caret down. Route heading events into the existing quote children collection, preserving outline offsets and the shared per-version parser.
- Superscript/subscript glyph offsets must not create extra navigation lines. Normalize fragment navigation Y to the body row using the same font metrics as rendering; retain actual text-layout Y for hit testing.
