## 1. Remote Image Request Resolution

- [x] 1.1 Add a focused HTTP(S) request-URL normalizer that removes a literal URI fragment while preserving the scheme, authority, path, query string, and percent-encoded data.
- [x] 1.2 Route remote sources through the normalized GPUI URI image source in `preview_image_source`, while preserving the existing local absolute/document-relative path branches and cached derived-state flow.
- [x] 1.3 Confirm Markdown preview, Read mode, Visual Edit, and supported raw-HTML image call sites continue to share `preview_image_source` while retaining the authored URL in source-backed blocks and exports.
- [x] 1.4 Install one reusable TLS-enabled implementation of GPUI's HTTP client during application startup, preserving response status/headers/body and redirect behavior without moving network state into document caches.
- [x] 1.5 Remove the unconditional alt/URL/title chrome beneath rendered Markdown images in preview and Visual Edit, and exclude this non-visible metadata from preview text selection.

## 2. Regression Coverage

- [x] 2.1 Add deterministic unit tests for a WeChat-style HTTPS URL with query parameters and `#imgIndex`, ordinary fragment-free HTTP(S) URLs, percent-encoded `%23`, and local paths containing `#`.
- [x] 2.2 Add or extend preview derivation coverage to prove the full authored remote image URL remains in the cached preview block while only the GPUI request source is fragment-free.
- [x] 2.3 Add a deterministic loopback HTTP test for the concrete client and startup wiring coverage so the app cannot silently regress to GPUI's null client.
- [x] 2.4 Add regression coverage proving image blocks contribute no visible/selectable metadata runs while retaining their source-backed URL data.

## 3. Verification

- [ ] 3.1 Run the targeted remote-image tests and `cargo test --workspace`, resolving failures within this change's scope.
- [ ] 3.2 Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `openspec validate fix-remote-image-preview`.
