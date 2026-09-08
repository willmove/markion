## 1. Establish regressions that detect omitted content

- [x] 1.1 Add a deterministic root GPUI regression using the observed preceding-paragraph/blank-line action history, followed by repeated text input and one Enter per line. Assert actual painting of a viewport-intersecting preceding text range together with caret centering on the first and unchanged frames; run it before the fix and record the missing-content failure and minimized fixture in `verification.md`.
- [x] 1.2 Add a focused GPUI list integration regression for a child autoscroll rectangle extending above its own row. Observe painted predecessor items and the attainable pixel target with variable and zero-height rows, padding, and start-boundary clamping; include unaffected downward reveal and short-content bottom alignment. Run these through a test host that actually includes the patched GPUI implementation.

## 2. Correct upward autoscroll normalization

- [x] 2.1 Normalize the upward-autoscroll result before the existing prepaint retry by resolving the first visible predecessor at the current width, preserving the attainable content-space target and measuring only necessary unmeasured rows. Keep the change local, preserve alignment behavior, and make the regressions in section 1 pass without changing Markdown ownership, adding document mutations, or increasing retry counts.

## 3. Verify Visual Edit integration and preserved behavior

- [x] 3.1 Exercise existing-document text/Enter and IME composition cycles at 640 x 400 and 480 x 300, including wrapped content, explicit font size 15 / paragraph spacing 13, and resize/typography transitions. Check visible preceding text and the caret together; permit text that legitimately leaves the viewport to stop painting.
- [x] 3.2 Verify first/final-row centering, manual scrolling until subsequent caret activity, disabled-typewriter reveal, and unchanged-frame stability with the corrected path. Preserve existing focus presentation and source/Split behavior using relevant current regression tests.
- [x] 3.3 Extend the rendering regression to compare text, dirty state, version, undo history, shared Markdown/visual caches, and cached text handles across presentation-only frames. Confirm the normalization performs no scroll-induced parsing or highlighting invalidation and remove temporary diagnostic instrumentation.

## 4. Build, native acceptance, and verification record

- [x] 4.1 Run the new content-coverage/list tests, `cargo test --bin markion visual_typewriter_small_window -- --nocapture`, relevant existing scrolling/reveal tests, and `cargo test`. Record exact commands and results, distinguishing root tests from any separately executed vendored GPUI unit tests.
- [x] 4.2 Run formatting checks appropriate to the touched root and vendored Rust files, scoped whitespace checks, and `cargo build`; record the resulting `target/debug/markion.exe` build identity for native verification. Do not modify unrelated files to satisfy these checks.
- [ ] 4.3 Verify the rebuilt debug executable in a small native Windows window using an existing multi-block document and repeated line-then-single-Enter input. Check both immediate and settled frames for preceding-content visibility, record document/viewport/typography conditions, and distinguish representative reproduction from the original document's exact cadence. Keep this task outstanding if native verification is unavailable or the reported alternating symptom remains unexplained.
- [x] 4.4 Finalize this change's `verification.md` with before/after coverage evidence, cache-preservation results, native findings, and any remaining limitations; run `openspec validate fix-visual-typewriter-content-visibility --strict` and confirm the follow-up requirement coexists with the earlier typewriter and focus-mode deltas before marking implementation complete.
