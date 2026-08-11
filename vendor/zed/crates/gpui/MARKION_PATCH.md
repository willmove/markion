# Markion GPUI patch

This directory contains the published `gpui` 0.2.2 crate, with the GPUI portions of Zed commit [`2ead8c42fb6792095d7cb02f7b89e467421dc8a0`](https://github.com/zed-industries/zed/commit/2ead8c42fb6792095d7cb02f7b89e467421dc8a0) backported.

The upstream change restores conventional Win32 keyboard-message translation, keeps `VK_PROCESSKEY` under IME ownership, and fixes international keyboard/IME routing. Markion carries this patch until a compatible crates.io GPUI release includes the fix.

In addition, `src/taffy.rs` (`to_grid_repeat`) uses explicit `0.0_f32` / `1.0_f32` literals instead of the upstream unsuffixed `0.0` / `1.0`. Newer rustc emits the `float_literal_f32_fallback` lint on the unsuffixed form, which Markion's CI promotes to a hard error via `RUSTFLAGS=-D warnings`. This is a compile-only change with no behavioral effect.

The upstream package license remains in `LICENSE-APACHE`.
