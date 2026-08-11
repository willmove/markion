## Why

On Windows, starting Microsoft Pinyin composition in Visual Edit can terminate Markion before any text is committed. The failure crosses the native GPUI input callback as a fatal process exit, so the editor must make the Win32 IME path safe and regression-test the composition boundary that current handler-only tests bypass.

## What Changes

- Patch the GPUI Windows input path so `VK_PROCESSKEY` remains owned by the IME and Win32 messages follow the conventional `TranslateMessage` flow.
- Harden Markion's platform-input range handling so stale or invalid native UTF-16 ranges cannot panic while crossing the Win32 callback boundary.
- Compose the Visual Edit IME underline with existing inline highlights as sorted, non-overlapping UTF-8 ranges so GPUI cannot derive font runs that split a CJK code point.
- Make cc-rs compile native objects sequentially only for MSVC, avoiding the reproducible C1056 build failure without disabling Cargo's Rust-crate parallelism.
- Add regression coverage for preedit, replacement, commit, cancellation, candidate geometry, and undo behavior at non-ASCII Visual Edit positions.
- Preserve the canonical `MarkdownDocument.text` model and the existing per-document derived-state/cache invariants.

Non-goals: redesigning Visual Edit, changing Markdown parsing or serialization, replacing GPUI, adding a custom input method, or changing non-Windows keyboard behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: clarify that native Windows IME preedit, including Microsoft Pinyin, must remain alive and preserve the existing Visual Edit composition guarantees.

## Impact

- Windows GPUI platform integration and dependency resolution in `Cargo.toml`/`Cargo.lock`.
- Visual Edit's `EntityInputHandler` implementation and its UTF-16-to-UTF-8 range validation.
- Visual Edit projection highlight composition and the locally patched cc-rs build helper.
- Windows-focused regression tests plus the existing root and workspace test suites.
