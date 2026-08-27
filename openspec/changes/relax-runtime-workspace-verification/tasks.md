## 1. Minimal launch gate in wechat-workspace

- [x] 1.1 Add `verify_launch_gate(root)` to `crates/wechat-workspace/src/assets.rs` (manifest parse + `validate_provenance` + `index.html` LF-normalized digest, reusing the existing digest helpers and `BundleError` variants) and export it from `lib.rs`
- [x] 1.2 Unit-test the gate: a valid bundle passes; extra unlisted files (including files nested in subdirectories) still pass; a tampered `index.html` fails with `DigestMismatch { path: "index.html" }`; a missing manifest entry or missing file fails with `MissingFile`; an unparseable manifest or invalid provenance fails
- [x] 1.3 Add the incident-shaped regression test: a valid bundle polluted with KaTeX-era leftovers (`static/vendor/katex.min.js`, `LICENSE.katex.txt`, font files) passes the gate while full `verify_bundle` still rejects it with `UnlistedFile`

## 2. Rewire runtime call sites

- [x] 2.1 Switch `discover_workspace_assets()` candidate acceptance from `verify_bundle` to `verify_launch_gate` and update its tests for polluted-candidate acceptance
- [x] 2.2 Switch `WorkspaceService::new()` to `verify_launch_gate` and extend the server tests so service construction and session creation succeed on a polluted bundle directory
- [x] 2.3 Confirm `cargo test -p wechat-workspace` passes with no change to `verify_bundle` behavior or the `verify-bundle` CLI (`rejects_remote_and_unlisted_runtime_files` and `verifies_the_checked_in_workspace` unchanged and green)

## 3. Verification and documentation

- [x] 3.1 Run `cargo test --workspace` and confirm the app layer (`src/app/publishing.rs`, i18n) needs no changes for the relaxed gate
- [x] 3.2 Manually verify on the affected Windows machine (v0.1.24 → v0.2.2 upgraded install with KaTeX leftovers): the publish menu opens the browser session; record the result in `docs/marknice-workspace-release-evidence.md` and note the runtime gate versus release gate distinction there
