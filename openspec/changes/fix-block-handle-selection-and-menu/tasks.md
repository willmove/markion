## 1. Block-selection boundary

- [ ] 1.1 Reproduce and fix exclusive-end activation in Visual projection without shortening complete source-unit selections. Verify paragraph/H2, CRLF/Chinese H3 and adjacent-heading cases, extended/reversed selections, next list/quote prefixes, real collapsed carets, empty structural rows, inline reveal and IME; assert selected source slices, next-block projection and unchanged version/cache identity.

## 2. Safe block actions

- [ ] 2.1 Add shared, version/identity-validated block Copy/Cut/Add Below commands using canonical source-unit ownership and existing clipboard/transaction paths. Verify portable text/HTML payloads, no mutation for Copy, exact one-step undo for Cut/insertion, nested list/quote ownership, LF/CRLF and UTF-8 preservation, and stale/ambiguous rejection without changing unrelated following markers.

## 3. Compact popup and grip gestures

- [ ] 3.1 Add a grip invocation mode to the shared block menu and render direct compact Text/H1-H6 and structural-format controls plus separated editing actions through the existing root overlay. Verify pointer and keyboard access, current/disabled states, localized tooltips, light/dark theme styling, narrow/short viewport reachability and painting above later rows/media; preserve explicit context-menu selection and submenu behavior and the ongoing slash-palette edits.
- [ ] 3.2 Open the popup only after a revalidated ordinary completed grip click and establish the clicked single source-unit selection before opening. Verify Shift extension, dragging past the threshold, multi-block drag, canceled/stale gestures, Escape/outside dismissal, scroll/tab/mode closure and that selection/menu lifecycle preserves document source, version, history and shared caches.

## 4. Preference and persistence

- [ ] 4.1 Wire default-enabled `block_handle_menu` through preferences/TOML load-save, application initialization/snapshot/reset and a localized General-tab toggle. Verify true/false round trips, missing/invalid fallback without losing unrelated settings, restart/reset behavior, immediate closure of a grip popup when disabled, selection-only disabled clicks, explicit context-menu availability and no document/cache mutation.

## 5. Integration verification

- [ ] 5.1 Run Rust formatting, focused projection/selection/block-menu/preferences regressions, the root application test suite, `cargo build`, and `openspec validate fix-block-handle-selection-and-menu --strict`. Perform a Windows desktop smoke check of ordinary click, disabled click, Shift-click, drag, clipboard/Cut/undo, Add Below and viewport edges with a paragraph immediately before H2/H3; record results and any unverified scope without marking incomplete checks complete.
