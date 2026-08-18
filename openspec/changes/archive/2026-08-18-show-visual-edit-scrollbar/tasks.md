## 1. List scrollbar helper

- [x] 1.1 Parameterize the existing ListState overlay in `src/app/root_view.rs` so callers supply an element id and whether to mark a Split Preview sync-scroll driver, without changing Read/Split Preview thumb geometry or drag math.
- [x] 1.2 Keep Preview overlay calls marking `PaneScrollTarget::Preview`; Visual Edit MUST NOT record an Editor or Preview driver.

## 2. Visual Edit overlay

- [x] 2.1 Attach the parameterized overlay as the last child of `visual_edit_surface_view`, driving it from the active tab's `visual_list` and reusing the existing right-side reserved padding.
- [x] 2.2 Hide the Visual Edit thumb when the document is empty or `visual_list` has no scrollable range, matching Read mode.
- [x] 2.3 Confirm wheel/trackpad scrolling, overlay dragging, and existing caret/IME/block-menu pointer paths still share that same `visual_list` without mutating document text or derived Markdown caches.

## 3. Tests and validation

- [x] 3.1 Add focused tests that a long Visual Edit document's `visual_list` updates through the scrollbar offset API, preserves per-tab visual scroll independently of `preview_list`, does not bump document version or caches, and does not leave a Preview sync-scroll driver.
- [x] 3.2 Run `pwsh ./scripts/check-quality.ps1` and `openspec validate show-visual-edit-scrollbar`.
- [x] 3.3 Manually verify a long document in Visual Edit shows a draggable right-side scrollbar like Read mode, that a short/empty document hides it, and that switching tabs restores the Visual Edit scroll position.

Verification note: `cargo fmt --all -- --check`, `cargo test --workspace` (including the new Visual Edit scrollbar tests), and `openspec validate show-visual-edit-scrollbar` all passed. `openspec validate --all --strict` still fails on the unrelated pre-existing change `fix-oss-mirror-upload`. Visual Edit now uses the same ListState overlay as Read mode (`list_pane_scrollbar_view`, right-side reserved padding, hide when `max_scroll <= 1px`), stacked above the IME bridge so the thumb remains draggable. Please spot-check a long document in Visual Edit in the running app.
