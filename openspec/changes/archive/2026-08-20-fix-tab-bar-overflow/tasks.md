## 1. State and reveal helper

- [x] 1.1 Add an app-level `tab_bar_scroll: ScrollHandle` field to `MarkionApp` (`src/app/state.rs`), initialized in every constructor path; it is window-presentation state, not per-tab state
- [x] 1.2 Add `MarkionApp::reveal_active_tab_in_strip()` that calls `self.tab_bar_scroll.scroll_to_item(self.active_tab)` and records the requested index into a `#[cfg(test)] last_tab_strip_reveal: Option<usize>` field on `MarkionApp`
- [x] 1.3 Call `reveal_active_tab_in_strip()` from every path that changes the active tab: `switch_active_tab` (`src/app/application.rs`), `new_tab`, the close paths that reassign `active_tab` (`src/app/documents.rs`), and the open-existing-tab focus path; audit remaining direct `active_tab` assignments to confirm none is missed

## 2. Strip restructure in `tab_bar_view`

- [x] 2.1 Turn the tab row into a stateful horizontal scroll container: `.id("tab-bar-scroll")` + `.overflow_x_scroll()` + `.track_scroll(&self.tab_bar_scroll)` with `.min_w_0().flex_1()`, keeping the 30px band height, border, and background unchanged
- [x] 2.2 Move the "+" new-tab button (and a small separator) out of the scroll container into a pinned `flex_shrink_0` region at the band's right edge, preserving its existing action wiring
- [x] 2.3 Bound each tab: `.max_w(px(220)).flex_shrink().min_w(px(96))` (constants named near `DOCUMENT_TAB_BAND_HEIGHT`), label wrapper `.min_w_0().truncate()`, close "×" `.flex_shrink_0()` — preserving the active-tab top-border treatment and hover behavior
- [x] 2.4 Replace the `format!("{name} *")` dirty suffix with a separate `flex_shrink_0` compact indicator child (e.g. `•`) shown only when the tab is dirty, colored from the palette

## 3. Hover tooltip

- [x] 3.1 Add a minimal `TabTooltip` render-only view (full title; full path line when the tab is file-backed) styled from the palette, and attach it via `.id(ElementId::Named("document-tab", index))` + `.tooltip(...)` on each tab

## 4. Tests and validation

- [x] 4.1 Structural tests in `src/app/tests.rs` following the `include_str!` idiom: strip uses `overflow_x_scroll` + `track_scroll`; "+" lives outside the scroll container (marker-order assertion); labels carry `truncate()` with `min_w_0`; dirty indicator is a separate element outside the label; close control is `flex_shrink_0`
- [x] 4.2 Behavioral tests using the `last_tab_strip_reveal` seam: switching, opening, closing, and focusing an existing tab each request the resulting active index; no reveal request is issued when the active tab is unchanged
- [x] 4.3 Run `cargo test` (root package) and `cargo build`; confirm no existing tab-bar or layout assertions regressed
