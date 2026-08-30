# Tasks: add-updater-manifest-fallback

## 1. Sequencing

- [x] 1.1 Archive `add-signed-one-click-updates` (complete, unarchived) so the updater requirements this change modifies exist in `openspec/specs/release-packaging/spec.md`, then run `openspec validate add-updater-manifest-fallback`

## 2. Client updater endpoint list

- [x] 2.1 In `src/app/update.rs`, replace the single `SIGNED_UPDATE_MANIFEST_URL` constant with two constants — the OSS mirror manifest URL (primary) and the GitHub Release `latest/download/update.json` URL (fallback) — and pass them as an ordered endpoint list to `cargo_packager_updater::Config.endpoints`
- [x] 2.2 Update the module documentation and the endpoint-URL test assertions to cover both endpoints and their order (OSS first)
- [x] 2.3 Run `cargo test --workspace` and confirm zero failures

## 3. Release workflow: two manifest variants

- [x] 3.1 In `.github/workflows/release.yml` `prepare-update`, generate two manifests from the same signed installer: one whose `platforms.windows-x86_64.url` is the GitHub asset URL (attached to the Release as `update.json`) and one whose URL is the OSS `latest/` object (uploaded by `mirror-oss`), sharing `version`, `pub_date`, `format`, and the identical `signature`
- [x] 3.2 In the `release` job, upload the GitHub-URL variant as the Release's `update.json` asset; in `mirror-oss`, upload the OSS-URL variant as the `update.json` object
- [x] 3.3 Extend `mirror-oss` verification to assert the mirrored manifest's installer URL is on the OSS host (in addition to the existing version and HTTP-200 checks)

## 4. Documentation

- [x] 4.1 Update `docs/release-process.md`: the final-verification checklist confirms both manifest variants (each names its own host, versions and signatures match), and the updater description no longer calls the GitHub manifest the single source

## 5. Validation

- [x] 5.1 Run `openspec validate add-updater-manifest-fallback` and `cargo test --workspace` with zero failures
- [ ] 5.2 On the next tagged release, confirm the GitHub Release's `update.json` names `github.com`, the OSS `update.json` names the OSS host, both carry the same signature, and the manual-download fallback still works
