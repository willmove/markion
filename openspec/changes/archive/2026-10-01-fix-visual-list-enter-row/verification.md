# Verification

## Reproduction

The supplied Chinese three-item list reproduces the second-Enter failure in a
GPUI regression: the first Enter continues the list, then the second removes the
empty marker but leaves the painted caret at the preceding item's coordinates.
The source has the extra newline. Parser offset probes confirm that list item
ranges own trailing line endings for which there are no inline text events.

## Repair

- Preserve omitted trailing line endings as exact source-backed list runs. One
  CRLF produces one visible line break. Before another block, its final separator
  is not counted twice.
- Give empty rows in the mixed link/math/script layout a source-position caret
  anchor. Keep the current document cache and structural editing operations.
- Update the existing nested-list projection expectation to include its authored
  final newline; descendant text remains present exactly once.

## Coverage

- Pure source/projection/cache regression for unordered, ordered and task lists,
  LF/CRLF, one through four trailing line endings and between-item separators.
- GPUI caret geometry for the supplied example with zero, one and four trailing
  newlines, including both continuation and exit.
- Real pointer placement and Enter key binding for first/middle/last items of
  all three list families and both newline formats (18 combinations), CJK/emoji
  composition, marked-range geometry, undo, and three further Enter presses
  after exiting the empty item.
- Mixed-layout link item with blank-row caret geometry.

## Checks

- `cargo fmt --all -- --check`: passed.
- `openspec validate fix-visual-list-enter-row --strict`: passed.
- `cargo test -p markion --bin markion visual_list_`: 4 passed, including all
  three new rendered interaction regressions.
- `cargo test -p markion`: library 596 passed / 1 ignored; application 596 passed
  / 2 ignored; doctests passed (0 tests). No failures. Existing compiler warnings
  concern the network test's unused import, Windows linker messages and the
  `proc-macro-error2` dependency's future compatibility notice.
