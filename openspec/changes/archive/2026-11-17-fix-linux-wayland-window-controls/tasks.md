## 1. Window-control assets and strings

- [x] 1.1 Add `assets/icons/ui/minus.svg` (lucide, stroke-consistent with the existing set) and declare `Minus` in the `icon_set!` table, reusing the existing square/copy/x glyphs for maximize/restore/close
- [x] 1.2 Add minimize/maximize/restore/close tooltip messages to `Msg` with translations for all seven languages and register the four variants in the non-empty-translation test list

## 2. Client-side window controls

- [x] 2.1 Add `src/app/window_controls.rs`: the `Decorations::Client` visibility gate, the button cluster (36×28 buttons, menu-consistent hover, danger-red close hover, `WindowControlArea` tagging, localized tooltips), and the threshold-deferred drag region with double-click maximize
- [x] 2.2 Add `request_window_close` beside `quit` in the unsaved-exit flow, mirroring `install_window_close_guard`'s native-close decision (allow_close/clean → layout capture + `remove_window`; dirty → existing `begin_unsaved_exit(WindowClose)`)
- [x] 2.3 Add the `csd_drag` press-origin field and integrate the drag spacer + control cluster into the in-window menu bar row in `root_view.rs`, keeping the bar byte-identical when decorations are server-side

## 3. Wayland decoration request (root cause of the invisible cluster)

- [x] 3.1 Diagnose live on GNOME Wayland: `WAYLAND_DEBUG=1` shows Mutter advertises no `zxdg_decoration_manager_v1`, and a render-time probe reports `decorations=Server gate=false` — GPUI's default `Server` request is stored as state without negotiation, so section 2's gate never fires
- [x] 3.2 Add `window_controls::requested_decorations()` (`Some(WindowDecorations::Client)` when `gpui::guess_compositor() == "Wayland"`, `None` elsewhere, `#[cfg]`-gated to Linux/FreeBSD) and pass it as `WindowOptions.window_decorations` in `bootstrap.rs`
- [x] 3.3 Extend the CSD source-contract test to pin the request wiring so the gate cannot silently regress to an always-false state

## 4. Verification

- [x] 4.1 `cargo fmt --check`, `cargo build`, and the root-package test suite pass, including the new drag-threshold unit test, CSD source-contract assertions, and the extended i18n non-empty test list (bin target: 772 passed / 0 failed; lib target: 723 passed / 1 pre-existing `ai_actions` failure that also fails on the unmodified tree; all `crates/*` members pass via `cargo test --workspace`)
- [x] 4.2 Confirm server-side-decorated platforms and test windows render the menu bar unchanged (gate false for `Decorations::Server`) — verified live on this machine's X11 GNOME session: native Mutter title bar with min/max/close present, in-app menu bar unchanged, no CSD cluster
- [x] 4.3 Verify live on the GNOME Wayland session (Ubuntu 26.04.1, GNOME Shell 50.1): a render-time probe reports `Decorations::Client` / gate true, and a desktop screenshot shows the minimize, maximize, and close glyphs at the right end of the in-window menu bar (no native title bar, as expected for Mutter CSD)
- [x] 4.4 Reconcile the checklist against proposal/design/delta (the `openspec` CLI is not installed on this machine; structure and delta format mirror archived changes such as `2026-10-01-add-format-menu-shortcuts`)
