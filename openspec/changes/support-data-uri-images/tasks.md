## 1. Dependency

- [x] 1.1 Add a direct dependency on the `data-url` crate to the root `Cargo.toml` `[dependencies]` (already vendored transitively in `Cargo.lock`; pin to the resolved version so the lock does not churn).
- [x] 1.2 Run `cargo build` and confirm the crate resolves without a new download.

## 2. PreviewImageKey: data-URI identity

- [x] 2.1 In `src/app/preview_image.rs`, branch `PreviewImageKey::from_url` so a URL starting with `data:` (per `is_remote_resource`) is assigned the identity `data:<full-uri>` instead of `remote:<...>`. Keep the local-path branch and the existing `remote:` branch untouched.
- [x] 2.2 Add `PreviewImageKey::data_url(&self) -> Option<&str>` that strips the `data:` prefix, mirroring the existing `local_path()` / `remote_url()` accessors.
- [x] 2.3 Add a unit test asserting `from_url("data:image/png;base64,..", None)` yields `identity == "data:image/png;base64,.."` and that `data_url()` / `local_path()` / `remote_url()` return Some / None / None respectively.

## 3. Loader: decode data URIs inline

- [x] 3.1 In `load_preview_image` (`src/app/preview_image.rs`), insert a branch between the `local_path()` and `remote_url()` arms: when `key.data_url()` is Some, parse and decode it with `data_url::DataUrl::process(url).decode()`, mapping the `Err` / non-`Ok` result to a `String` error so it flows into the existing missing-resource placeholder path.
- [x] 3.2 Capture the decoded MIME type from the `DataUrl` and use it (alongside the existing byte-scan) to decide the SVG vs. raster branch in `load_preview_image` — treat `image/svg+xml` as SVG. Route the resulting bytes into the existing `rasterize_svg_bytes` / `decode_raster_bytes` tail (no new decode path).
- [x] 3.3 Ensure the `remote_url()` branch is only reached for `http(s)://`-style URLs, so `reqwest` never receives a `data:` scheme. Add an assertion-style unit test (or `debug_assert!`) that `remote_url()` never returns a string starting with `data:`.

## 4. Classification helper

- [x] 4.1 Confirm `is_remote_resource` (`src/app/preview.rs`) still returns true for `data:` URIs (required so `from_url` routes them away from the local-path branch). No code change expected; verify and document with the existing test at `src/app/tests.rs:481` plus a new assertion that a data URI is classified as remote but produces a `data:`-prefixed identity.

## 5. Tests

- [x] 5.1 Unit test: a valid base64 PNG data URI decodes through `load_preview_image` (or a thin helper extracted from it) and yields a `PreviewImageReady` with nonzero dimensions.
- [x] 5.2 Unit test: a valid base64 SVG data URI decodes through the SVG branch and yields a `PreviewImageReady` whose display dimensions match the SVG intrinsic size (subject to the max-edge clamp).
- [x] 5.3 Unit test: a non-base64 (URL-encoded) data URI decodes to the same bytes as its base64 equivalent.
- [x] 5.4 Unit test: a malformed data URI (truncated base64 / invalid percent-encoding) returns `Err`, which the existing missing-resource view will surface as the placeholder.
- [x] 5.5 Unit test: two identical data URIs in the collected URL list produce the same `PreviewImageKey` so they dedupe in the cache.

## 6. Build & verify

- [x] 6.1 Run `cargo test` (root package) and confirm all new and existing image-loading tests pass.
- [x] 6.2 Run `cargo test --workspace` and `cargo build` to confirm no workspace invariant (gpui-free `crates/*`) is broken and there are no new warnings in the modified files.
- [x] 6.3 Manually open a `.md` document containing `![](data:image/png;base64,...)` and an `<img src="data:...">` block, and verify both render in preview and Visual Edit without a network request (confirm via the existing missing-image placeholder not appearing).
