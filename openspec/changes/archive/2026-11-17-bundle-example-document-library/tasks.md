## 1. Offline example collection

- [x] 1.1 Add the bilingual index, five focused guides, twelve daily-use templates, and bundled sample PNG under assets/examples; verify each document has useful content, reuse guidance, and working local references.

## 2. Installed discovery

- [x] 2.1 Add the shared Examples and Templates Help action to both menus with all supported UI translations, missing-resource feedback, and a welcome pointer; verify compilation, resource lookup, and relevant menu/localization checks without changing document caches.

## 3. Distribution integrity and validation

- [x] 3.1 Add a collection validator and invoke it for every extracted native package; verify source validation and complete/missing/altered copied-payload cases.
- [x] 3.2 Run OpenSpec validation and appropriate Rust checks, review the final scoped diff, and record verification results and any unavailable native package/UI checks.

## Verification record (2026-11-17)

- `cargo test --workspace`: passed; 2,216 tests passed, zero failed, five ignored by the existing suite configuration, across 51 result groups. Includes the application's menu, localization, bundled-resource lookup, and welcome-content coverage.
- `cargo fmt --all -- --check`: passed.
- `openspec validate bundle-example-document-library --strict`: passed.
- `scripts/verify-examples.ps1`: passed for 18 Markdown documents (index, five guides, twelve templates); strict UTF-8, index coverage, and local Markdown/HTML references checked.
- Copied-payload checks: all 19 files accepted; a missing template, an altered template, and a broken relative link each rejected with its path.
- Windows portable payload smoke: assembled a local ZIP fixture with the freshly compiled debug application, full assets, and third-party notices; `verify-packaged-workspace.ps1 -Format zip` passed after extraction, including all 19 example file hashes and existing publishing workspace checks. This is a debug payload fixture, not a release installer.
- `git diff --check`: passed. Pre-existing unrelated working-tree changes were preserved.
- Not performed locally: live UI/system-file-manager interaction, NSIS installation, native macOS/Linux package builds, or release publication. The existing native CI verification now checks examples in every supported package format. This change remains active and ready for archive; stable specs were not edited.
