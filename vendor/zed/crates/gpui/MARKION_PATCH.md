# Markion GPUI patch

This directory contains the published `gpui` 0.2.2 crate, with the GPUI portions of Zed commit [`2ead8c42fb6792095d7cb02f7b89e467421dc8a0`](https://github.com/zed-industries/zed/commit/2ead8c42fb6792095d7cb02f7b89e467421dc8a0) backported.

The upstream change restores conventional Win32 keyboard-message translation, keeps `VK_PROCESSKEY` under IME ownership, and fixes international keyboard/IME routing. Markion carries this patch until a compatible crates.io GPUI release includes the fix.

The upstream package license remains in `LICENSE-APACHE`.
