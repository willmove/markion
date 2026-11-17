## 1. Corner-radius decision

- [x] 1.1 Add `window_controls::csd_corner_radii(decorations, maximized, fullscreen) -> Corners<Pixels>`: zero for `Decorations::Server`, 12 px otherwise, zeroed when maximized/fullscreen and per tiled edge from `Tiling`
- [x] 1.2 Unit-test the helper: normal, maximized, fullscreen, single-edge tiled, and server-decorated cases

## 2. Rounded chrome

- [x] 2.1 Root view: apply the four radii to the root background and append the input-transparent hairline rounded border overlay as the last root child (CSD only)
- [x] 2.2 Menu bar row: apply the top radii to its background; on CSD use 12 px left padding and 0 right padding so menu backgrounds stay out of the corner zone and the cluster sits flush right
- [x] 2.3 Close button: carry `rounded_tr` equal to the top-right radius so its hover background respects the window corner

## 3. Verification

- [x] 3.1 `cargo fmt --check` on touched files (no new diffs; the two pre-existing rustfmt-version diffs in `tests.rs` are unchanged), `cargo build`, and `cargo test --bin markion` pass: 773 passed / 0 failed (lib target's pre-existing `ai_actions` failure unchanged)
- [x] 3.2 Extend the CSD source-contract test to pin the radius wiring (root radii, menu bar radii/padding, close-button radius)
- [x] 3.3 Verify live on GNOME Wayland (Ubuntu 26.04.1): screenshots show all four corners rounded with the desktop/underlying window visible through the corner pixels, the hairline outline following the silhouette, the flush-right control cluster and the status bar's help button inside it, and no background spill; server-side platforms unaffected by construction (gate false)