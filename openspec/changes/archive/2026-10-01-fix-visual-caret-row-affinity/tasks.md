## 1. Reproduce and repair

- [x] 1.1 Add a failing GPUI regression comparing the painted marker-line caret with the terminal blank-line caret, and verify canonical input locations.
- [x] 1.2 Correct terminal block ownership and whitespace row mapping; verify LF/CRLF, EOF, multiple blanks, and per-version cache reuse with model and GPUI tests.

## 2. Audit and validate

- [x] 2.1 Audit adjacent empty headings, paragraphs with inline content, list/quote rows, pointer/arrow navigation, mode transitions, and Unicode/IME/undo; add regression coverage for confirmed related defects.
- [x] 2.2 Run root regression tests and OpenSpec validation, and document results and any remaining limitations in the visual-editing quality guide.

Validation (2026-09-17): `cargo test --no-fail-fast` passed: 649 library tests and 622 application tests, with 3 pre-existing ignored tests. `rustfmt --check` for the changed Rust files, `git diff --check`, and `openspec validate fix-visual-caret-row-affinity --strict` passed. The original GPUI reproducer failed before the repair with identical caret bounds for the marker-line end and the following EOF line.
