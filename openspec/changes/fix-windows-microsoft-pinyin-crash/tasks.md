## 1. GPUI Windows Input Backport

- [x] 1.1 Vendor the published GPUI 0.2.2 package outside `crates/*`, retain its license, and select it with a reproducible Cargo patch without making it a workspace member.
- [x] 1.2 Backport the GPUI portions of upstream commit `2ead8c42fb6792095d7cb02f7b89e467421dc8a0`, including standard Win32 message translation and IME-owned `VK_PROCESSKEY` routing.
- [x] 1.3 Regenerate the lockfile and compile Markion against the patched GPUI on Windows.

## 2. Panic-Safe Platform Ranges

- [x] 2.1 Add a single checked UTF-16 platform-range conversion path that rejects stale, out-of-bounds, reversed, or non-boundary ranges.
- [x] 2.2 Use checked text access and safe selection fallback in ordinary replacement, marked-text replacement, and candidate-bound lookup without changing document-version cache invalidation.

## 3. Regression Coverage

- [x] 3.1 Add tests for invalid/stale platform ranges at non-ASCII Visual Edit positions and verify they cannot panic or corrupt canonical source.
- [x] 3.2 Extend composition tests for Microsoft Pinyin-shaped preedit updates, commit/cancel behavior, candidate geometry, and one-step undo.
- [x] 3.3 Run focused IME tests and `cargo test --workspace`.

## 4. Validation

- [x] 4.1 Run `cargo fmt --check`, validate the OpenSpec change, and document the remaining manual Microsoft Pinyin smoke check.

## 5. Crash-Stack Correction and Build Reliability

- [x] 5.1 Reproduce the reported post-backport crash with redirected stderr and capture the exact Rust stack at the Win32 callback boundary.
- [x] 5.2 Add a failing regression for out-of-order/overlapping Visual Edit composition highlights, then merge the IME underline with existing inline styles into sorted, non-overlapping UTF-8 ranges.
- [x] 5.3 Reproduce MSVC C1056 under normal Cargo parallelism; keep cc-rs's MSVC object compilation sequential and omit its unstable `-Brepro` flag for Microsoft `cl.exe` while preserving Cargo's Rust-crate parallelism.
- [x] 5.4 Re-run focused regressions, the workspace suite, OpenSpec validation, and the original interactive Microsoft Pinyin scenario.
