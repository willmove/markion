## 1. Search Overlay State and Dismissal

- [x] 1.1 Add a single close/dismiss helper for Find / Replace that hides the overlay, clears `search_focus`, calls `refresh_search_matches()`, and preserves `search_query` / `replace_text`.
- [x] 1.2 Wire the new helper to an explicit close control in the search UI.
- [x] 1.3 If Escape participates in dismissal, preserve the existing priority: pending name input first, Find / Replace overlay second, file-tree filter last.

## 2. Floating Layout

- [x] 2.1 Remove the existing in-flow `search_panel_view(self, cx)` row from the root flex column so opening search no longer shifts the workspace.
- [x] 2.2 Render the Find / Replace UI as an absolute-positioned root overlay near the upper-right of the editor/preview workspace.
- [x] 2.3 Constrain the overlay width for Find and Replace modes so it remains compact and does not overflow small windows.
- [x] 2.4 Ensure the overlay stays above editor/preview content and does not interfere with existing higher-priority menu, context-menu, or preferences-panel overlays.

## 3. Theme-Aware Styling

- [x] 3.1 Update `search_panel_view` to use `ThemePalette` for surface, border, text, muted summary text, and shadow/hover-compatible colors.
- [x] 3.2 Update `search_field_view` to use palette-driven background, text, normal border, and active border colors.
- [x] 3.3 Update search toolbar buttons and the close control to use palette-driven backgrounds, text colors, borders, and hover states.
- [x] 3.4 Remove hard-coded light-only search chrome colors from the Find / Replace overlay path.

## 4. Behavior Preservation

- [x] 4.1 Verify Find and Replace shortcuts still open the overlay in the correct mode and focus the correct search field.
- [x] 4.2 Verify query editing, case-sensitive and regex toggles, next/previous navigation, current/total count display, replace current, and replace all still behave as before.
- [x] 4.3 Verify closing the overlay clears active match highlights but keeps the find query and replacement text available on reopen.
- [x] 4.4 Verify switching themes while the overlay is visible immediately updates overlay styling.

## 5. Validation

- [x] 5.1 Add or update focused tests for close/dismiss behavior and match clearing where practical.
- [x] 5.2 Run `cargo test` for the root crate.
- [x] 5.3 Run `cargo check` if full tests are impractical during implementation.
- [x] 5.4 Run `openspec validate floating-find-replace-bar`.
