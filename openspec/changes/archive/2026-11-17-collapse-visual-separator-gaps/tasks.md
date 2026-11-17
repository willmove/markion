## 1. Gap geometry

- [x] 1.1 Count body lines from a whitespace source range in `src/visual.rs` (collapse the first newline only between blocks; no phantom end line) and store that count as `height_signature`. Verify with the list-tail and blank-line mapping unit tests.
- [x] 1.2 Stop flooring the virtual-list line clamp at one, so a signature of zero stays distinct from one visible line. Verify the splice still remeasures a tail whitespace row whose signature grows.

## 2. Visual Edit layout and caret

- [x] 2.1 Render a collapsed separator as paragraph spacing inside the row, with extra newlines at body line height, and keep the empty end row at zero body height. Verify a single inter-block newline's row height tracks paragraph spacing and does not equal a body line.
- [x] 2.2 Map clicks and Up/Down across separator and extra lines without inserting a newline. Verify gap-click and arrow tests still land inside the whitespace source range.
- [x] 2.3 Update the typing tests that froze following content: typing into a structural separator moves later blocks down. Verify those GPUI tests pass.

## 3. Notes

- [x] 3.1 Update the blank-line row in `docs/visual-editing-quality.md` to describe separator spacing versus extra body lines. Verify the sentence no longer claims every gap reserves a body line.
