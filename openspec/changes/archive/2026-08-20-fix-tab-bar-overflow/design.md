## Context

`tab_bar_view` (`src/app/editing.rs:2087`) renders the strip as a plain flex row: tabs have no width bounds, the label has no truncation, and the container has no overflow handling, so tabs past the available width are clipped by GPUI's default overflow behavior with no recovery path. The dirty marker is a `" *"` suffix baked into the label string (`editing.rs:2107-2111`), which truncation would swallow first. The trailing "+" button is a sibling of the tabs inside the same row.

GPUI 0.2.2 already provides every primitive needed (verified against the vendored source):

- Vertical wheel deltas are automatically routed into horizontal scrolling when a container scrolls on x only and does not set `restrict_scroll_to_axis` (`gpui/src/elements/div.rs:2424-2429`) — wheel support costs nothing.
- `ScrollHandle::scroll_to_item(ix)` has a horizontal "make child visible" branch that scrolls the minimal amount and does nothing when the child is already visible (`div.rs:3188-3194`); child bounds of a scroll container are tracked automatically (`div.rs:1375-1380`).
- Scroll offsets are clamped to content on every layout pass (`div.rs:1752-1758`), so closing tabs cannot leave the strip scrolled past its end.
- `.truncate()` (ellipsis + nowrap) and the `max_w` + `overflow_hidden` idiom are already used in the codebase (`src/app/preview.rs:2195-2197`, `src/app/root_view.rs:613-618`); a stateful scroll container with `track_scroll` exists at `src/app/root_view.rs:758-770`.
- `Stateful<Div>::tooltip(builder)` exists (`div.rs:538`); the builder returns any view, which GPUI positions near the pointer. Tabs currently have no element id, and tooltips require a stateful element.

Data flow note (rendering-state rule): the tab strip is pure presentation. It reads `tabs`, `active_tab`, and `is_dirty()`; it performs no document parsing and touches none of the versioned derived-state caches. The only new state is one app-level `ScrollHandle`, which is window-presentation state, deliberately not per-tab (a strip scroll position is not part of a document's isolated tab state and must not be restored per tab).

## Goals / Non-Goals

**Goals:**

- Keep every open tab reachable and closable at any tab count and window width.
- Preserve the current visual language of the strip (content-sized tabs, active top-border treatment, 30px band height).
- Zero new dependencies, zero i18n surface, zero changes to the tab model or persistence.

**Non-Goals:**

- Visible scrollbar or chevron affordances, an overflow dropdown listing hidden tabs, drag-to-reorder, middle-click close, tab pinning, multi-row wrapping. These can layer on later without changing this design.
- Any change to the single-tab layout (band hidden at ≤1 tab, unchanged).

## Decisions

### D1: GPUI scroll container, not virtualization or a custom element

The strip becomes `div().id("tab-bar-scroll").overflow_x_scroll().track_scroll(&app.tab_bar_scroll)` with `.min_w_0().flex_1()` inside the band row. Tab count is small (tens) and each tab is cheap, so `list`-style virtualization is unnecessary; a stock scroll container gives wheel, trackpad, offset clamping, and per-child bounds tracking for free.

*Alternative rejected:* an overflow dropdown menu ("»" listing hidden tabs) — determining which tabs are hidden requires post-layout measurement and a frame of latency; the scroll strip reaches the same goal with no measurement.

### D2: Shrink-then-scroll width policy with per-tab bounds

Each tab gets `.max_w(px(220)).flex_shrink().min_w(px(96))`. Inside the tab, the label wrapper gets `.min_w_0().truncate()`; the close "×" and the dirty indicator get `.flex_shrink_0()`. Because the strip's content lays out as flex inside the scroll container, tabs first shrink toward `min_w` as the count grows and only overflow (enabling scroll) once every tab is at minimum width — the Chrome/VS Code/Zed behavior. The exact 220/96 values are tunable during implementation without changing the design.

*Alternative rejected:* equal-width `flex_1` tabs — wastes width on short titles, discards the current content-sized look, and still needs a scroll backstop.

### D3: Dirty indicator becomes a separate non-shrinking element

Replace `format!("{name} *")` with the plain label plus a separate `.flex_shrink_0()` child (compact `•` glyph, colored via the palette) shown only when the tab is dirty. This is required for the truncation requirement: any suffix inside the label is the first thing truncation removes, and a truncated dirty tab reading as clean is a data-loss-adjacent lie. The glyph rides outside the truncated text so it survives every width.

### D4: Auto-reveal via one helper called from the activation entry points

Add `MarkionApp::reveal_active_tab_in_strip()` that calls `self.tab_bar_scroll.scroll_to_item(self.active_tab)`. Call it from `switch_active_tab` (`src/app/application.rs:170`), `new_tab` (`src/app/documents.rs:140`), the close paths (`close_tab` / `close_tab_confirmed` and any other path that reassigns `active_tab`, e.g. after closing), and the open-existing-tab focus path. The handle's `FirstVisible` strategy already implements "minimal scroll, no-op when visible"; the strip's `cx.notify()` cycle consumes the pending reveal at the next prepaint. Keeping one helper means the policy has a single definition site.

### D5: "+" pinned outside the scroll region

The band row becomes `[scroll strip (flex_1 · min_w_0)] [separator + "+" button (flex_shrink_0)]`. A "+" inside the scroll region could itself be scrolled out of reach — the defect resurfacing on the very control that creates tabs.

### D6: Tooltip via a minimal render-only view on a stateful tab element

Each tab gains an element id (`.id(ElementId::Named("document-tab", index))` — required for tooltip attachment and harmless otherwise) and `.tooltip(move |_, cx| cx.new(|_| TabTooltip { .. }).into())`. `TabTooltip` is a tiny `Render` view showing the full title and, when `tab.path()` is file-backed, the full path beneath it, styled from the same palette (panel background, border, small padding). Showing it on every tab (not only truncated ones) is deliberate: "is this title truncated?" is unknowable at element-construction time without font measurement, and tooltips on all tabs match browser convention. No i18n strings are introduced — the tooltip renders the tab's own title/path data.

### D7: Test seam instead of asserting on private GPUI state

`ScrollHandle`'s pending-reveal state is private to GPUI, so behavioral tests cannot read it. The helper sets a `#[cfg(test)] last_tab_strip_reveal: Option<usize>` field on `MarkionApp` (the codebase already uses `#[cfg(test)]` fields in `DocumentTabState`), letting tests assert that switch/new/close requested the right index. Structural facts (scroll container present, `+` outside it, `truncate()` on labels, dirty element outside the label, `flex_shrink_0` on controls) follow the repo's established `include_str!` string-assert idiom (as in `src/app/tests.rs:5775-5808`).

## Risks / Trade-offs

- [Wheel scrolling is undiscoverable — no visible scrollbar] → The active-tab auto-reveal covers the primary flows (switching always lands on a visible tab); chevron buttons driven by `max_offset().width > 0` are a clean future addition on top of this design.
- [Tooltip on every tab could feel noisy] → Standard browser behavior; cheap to restrict later by comparing measured label width if it bothers anyone.
- [`restrict_scroll_to_axis` left at its false default is what makes plain wheels work; a future edit setting it would silently break wheel scrolling] → Structural test asserts the strip relies on the plain `overflow_x_scroll` arrangement.
- [Minimum-width tabs squeeze the truncated label, "•", and "×" together] → `min_w(96)` keeps them legible at 12px text; value is a single constant to tune.
- [Stale scroll offset when the band reappears after dropping to one tab] → GPUI clamps offsets to content on layout, so a persisted offset self-corrects to zero.

## Migration Plan

Single-commit presentation change; no persistence, config, or document-format impact. Rollback is reverting the commit. The `" *"`-suffix dirty display is replaced in the same commit by the separate indicator, so no intermediate state shows truncated dirty tabs as clean.
