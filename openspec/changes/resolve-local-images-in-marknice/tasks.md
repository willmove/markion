## 1. Publishing image scope in the wechat-workspace crate

- [x] 1.1 Relax the lexical reference check in `crates/wechat-workspace/src/resource.rs` to permit `..` path components (canonical containment becomes the deciding authority) while still rejecting absolute paths, drive prefixes, `://` URLs, and NUL; update unit tests so `%2e%2e/outside.png` is now rejected by containment rather than lexically.
- [x] 1.2 Change `PublishingResource::from_path` to take the caller-supplied scope root (rename the `asset_root` parameter to `scope_root`); keep the read-time canonicalization, containment re-check, extension whitelist, and regular-file check semantics in `read()` unchanged.
- [x] 1.3 Extend crate tests to cover the scope matrix: same-level file, any-depth subfolder, exactly one level up (`../x.png`), parent tree (`../assets/x.png`), escape above the parent level (`../../x.png` → rejected), absolute path (rejected), symlink escape (rejected), unsupported extension (rejected), missing file (rejected), and that only caller-enumerated candidates ever become resources.

## 2. Snapshot classification in the root crate

- [x] 2.1 Update `build_publishing_snapshot` in `src/publishing.rs` to compute the scope root as the parent of the document's directory (degenerating to the document's own directory when no parent exists) and pass it to `PublishingResource::from_path`.
- [x] 2.2 Update `src/publishing.rs` tests: sibling, nested, and parent-level images resolve into `resources`; `../../` escapes and absolute paths land in `unresolved_local_images`; untitled documents still grant no resources; the existing derived-cache and version invariants stay asserted (no recompute, no dirty flip).

## 3. Organize planner in storage

- [x] 3.1 Add a planner in `src/storage/resources.rs` that, given a saved document path and its authored image references, resolves out-of-scope local candidates: relative references of any `../` depth and absolute local paths that point to readable, whitelisted, regular image files; exclude `file:` URLs, unsupported extensions, and unreadable or missing files.
- [x] 3.2 Add planner tests covering the candidate/no-go matrix, including Windows absolute paths and references that stay in-scope (not offered).

## 4. Rewrite machinery in the document model

- [x] 4.1 Add a command-time, parser-based scan (same pulldown-cmark options and `html_preview_parts` semantics as `publishing_image_references`) that returns exact image-destination byte ranges per authored URL for both Markdown image syntax and inline HTML `src`; keep it out of the per-keystroke derived caches.
- [x] 4.2 Implement the organized rewrite as a single `apply_transformed_text`-style splice replacing only the mapped destination ranges, and test: Markdown and inline HTML images rewrite, a plain link sharing the destination is preserved, the version advances exactly once with one undo step, and byte-identical imports reuse the stored asset file.

## 5. App action, menu, and localization

- [x] 5.1 Add the `OrganizeLocalImages` action: untitled documents get a save-first status (no changes), saved documents scan and report a nothing-to-organize status when no candidates exist.
- [x] 5.2 Show the GPUI confirmation prompt (`window.prompt`) listing the candidate count and destination summary; Cancel performs no filesystem or document change.
- [x] 5.3 On confirmation: copy each candidate through the existing import path (`import_image_file`), apply the rewrite from task 4.2, leave the document dirty and unsaved, and report organized/skipped/failure counts in the status bar; failed copies leave their references untouched.
- [x] 5.4 Register the menu item in the Export menu next to the WeChat publishing entry.
- [x] 5.5 Add all new user-facing strings (menu label, prompt title/detail/buttons, statuses) to `src/i18n.rs` for en, zh-hans, zh-hant, ja, fr, de, and es.
- [x] 5.6 Add app-level tests: confirm and cancel flows, partial copy failure, untitled guard, shared-destination link preservation end-to-end, and single-step undo after organizing.

## 6. Verification

- [x] 6.1 Run `cargo test --workspace` and confirm green; confirm `git status` shows no changes under `assets/marknice-workspace/` (bundle and manifest stay byte-identical).
- [ ] 6.2 Manual smoke test: publish a document with same-level, nested, and parent-level images (all preview), then one with `../../` and absolute-path images (unresolved warning), run organize with confirmation, and re-publish to verify the organized images now preview through the managed path.
