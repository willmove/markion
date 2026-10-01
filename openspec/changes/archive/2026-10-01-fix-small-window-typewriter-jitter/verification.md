# Verification

## Reproduction and diagnosis

Inspected the supplied 10.8-second recording using extracted frames. The visible case is a short Visual Edit document with Chinese input and repeated newlines in a small window.

Before the fix, `cargo test --bin markion visual_typewriter_small_window_short_document -- --nocapture` failed in 0.02 seconds:

```text
step=2 frame=0: typing on centered row displaced caret: 24px
test result: FAILED. 0 passed; 1 failed
```

The preceding frame had a centered caret and list offset 12px. The next input retained that offset but painted the caret 24px below center. Initial input in an empty document additionally produced a first-frame delta of -150px before boundary space was available. This distinguishes a visible delayed correction from a failure to eventually center.

The follow-up repeated-Enter reproduction failed before the terminal-row correction on the first press:

```text
press=1: every terminal Enter must immediately create the caret's blank visual row
Paragraph source_range: 0..5
editable run: 0..4 ("Body")
hidden marker: 4..5 ("\n")
```

On the second press the additional newline became a `Whitespace` row, explaining the reported odd/even alternation. After the correction, one through eight consecutive Enter presses all leave the caret owned by the terminal whitespace row and centered in every explicitly drawn frame.

## Automated results

- `cargo test visual_ -- --nocapture`: 31 library and 129 app tests passed, including projection, visual source coverage, incremental identity, navigation, tail whitespace, and typewriter behavior.
- `cargo test visual_typewriter_small_window_consecutive_enters_do_not_alternate -- --nocapture`: passed after failing on press one before the correction.
- `cargo test`: passed, including 559 library tests with 1 existing ignored test, 529 app tests with 2 existing ignored tests, and doc tests; no failures.
- `cargo fmt --check`: passed.
- `git diff --check` for the touched app, visual-model, test, and OpenSpec files: passed.
- `openspec validate fix-small-window-typewriter-jitter`: passed.

New tests inspect every explicitly drawn frame, including the first frame after input, inside the same App update to prevent the test platform from consuming the transient frame. They cover 640x400 and 480x300 windows, resizing to 380x260, Chinese text, IME composition/commit, eight consecutive Enter presses, repeated newlines, LF and CRLF terminal ownership, and a paragraph taller than the viewport. Every observed caret is within 1 logical pixel of center. They verify a newly painted caret rather than cached bounds alone and preserve document version, undo history length, derived visual-block Arc identity, cached text allocation, contiguous source coverage, and stable block identity across scroll-only frames.

## Limitations

Tests use GPUI's Windows test platform. The installed computer-control surface did not expose native application automation, so the built app was not replayed interactively. No release, installer, or public tag was created. Unrelated existing changes in the checkout were preserved.
