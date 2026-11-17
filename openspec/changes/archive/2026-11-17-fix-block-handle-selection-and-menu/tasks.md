## 1. Block-selection boundary

- [x] 1.1 Reproduce and fix exclusive-end activation in Visual projection without shortening complete source-unit selections. Verify paragraph/H2, CRLF/Chinese H3 and adjacent-heading cases, extended/reversed selections, next list/quote prefixes, real collapsed carets, empty structural rows, inline reveal and IME; assert selected source slices, next-block projection and unchanged version/cache identity.

## 2. Safe block actions

- [x] 2.1 Add shared, version/identity-validated block Copy/Cut/Add Below commands using canonical source-unit ownership and existing clipboard/transaction paths. Verify portable text/HTML payloads, no mutation for Copy, exact one-step undo for Cut/insertion, nested list/quote ownership, LF/CRLF and UTF-8 preservation, and stale/ambiguous rejection without changing unrelated following markers.

## 3. Compact popup and grip gestures

- [x] 3.1 Add a grip invocation mode to the shared block menu and render direct compact Text/H1-H6 and structural-format controls plus separated editing actions through the existing root overlay. Verify pointer and keyboard access, current/disabled states, localized tooltips, light/dark theme styling, narrow/short viewport reachability and painting above later rows/media; preserve explicit context-menu selection and submenu behavior and the ongoing slash-palette edits.
- [x] 3.2 Open the popup only after a revalidated ordinary completed grip click and establish the clicked single source-unit selection before opening. Verify Shift extension, dragging past the threshold, multi-block drag, canceled/stale gestures, Escape/outside dismissal, scroll/tab/mode closure and that selection/menu lifecycle preserves document source, version, history and shared caches.

## 4. Preference and persistence

- [x] 4.1 Wire default-enabled `block_handle_menu` through preferences/TOML load-save, application initialization/snapshot/reset and a localized General-tab toggle. Verify true/false round trips, missing/invalid fallback without losing unrelated settings, restart/reset behavior, immediate closure of a grip popup when disabled, selection-only disabled clicks, explicit context-menu availability and no document/cache mutation.

## 5. Integration verification

- [x] 5.1 Run Rust formatting, focused projection/selection/block-menu/preferences regressions, the root application test suite, `cargo build`, and `openspec validate fix-block-handle-selection-and-menu --strict`. Perform Windows desktop smoke checks of ordinary/disabled click, drag, Cut/undo and Add Below with a paragraph immediately before H2/H3; verify Shift-click and viewport edges through GPUI input/layout tests. Record that the available native Windows input APIs cannot hold Shift during a click, distinguish automated coverage from OS-input evidence, and report any remaining unverified scope explicitly.

## 6. Reorder separator follow-up

- [x] 6.1 Reproduce and fix merged paragraphs in the shared source-unit move helper using cached block ownership for tight-list exceptions. Verify both directions, single/multiple selected units, no/one/multiple terminal LF/CRLF, whitespace-only blank lines, Unicode, independent quote/list boundaries and intact tight list subtrees with exact-source and reparsed-structure regressions.
- [x] 6.2 Add desktop regressions for real grip drag and keyboard/menu movement at document boundaries, asserting independent paragraphs, complete list/multi-block ownership, resulting selection, stale rejection and exact one-step undo; run the focused move and existing grip suites.
- [x] 6.3 Run root tests, desktop build, formatting and strict OpenSpec validation; verify an OS drag/save/undo on an isolated fixture, close the earlier separator follow-up in the verification record, and update project progress and usage documentation with current results and limitations.

## Verification record

Implemented and verified on Windows on 2026-11-16. The initial grip implementation recorded 1570 passing root tests. The separator follow-up added three library and two desktop regressions: final root `cargo test` passed 1575 tests with 3 existing ignored and 0 failed (752 library, 818 desktop, 5 integration). Desktop build, formatting, diff whitespace checks and strict OpenSpec validation passed. Eight GPUI drag combinations and an OS drag/save/single-Undo exact-byte comparison close the earlier separator follow-up. See [the verification record](../../../docs/verification/block-grip-menu-windows-2026-11-16.md) for native checks, build hashes/screenshots, raw summary and coverage limits. Stable specs have not been changed directly; all nine tasks are complete and the change is ready for a later archive.
