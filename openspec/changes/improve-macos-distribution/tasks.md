## 1. Ad-hoc signing

- [x] 1.1 Set `[macos] signing-identity = "-"` in `packager.toml` and update the signing/comments blocks (ad-hoc sealed bundle, not Developer-ID/notarized; seven packages mirrored); verify with `cargo packager --release --config packager.toml --formats app` locally if a full local build is available, otherwise rely on CI verification in 2.2

## 2. CI matrix: cross-compiled macOS x86_64 + signature gate

- [x] 2.1 In `.github/workflows/release.yml`, add the `markion-macos-x86_64` matrix entry (`x86_64-apple-darwin`, formats `app,dmg`); make the build step pass `--target x86_64-apple-darwin` for that entry; split the packaging step so the x86_64 entry invokes `cargo packager ... --target x86_64-apple-darwin --binaries-dir target/x86_64-apple-darwin/release` with `CARGO_BUILD_TARGET=x86_64-apple-darwin` (keeps the packager's `before-packaging-command` building the same target), while existing entries keep today's command unchanged
- [x] 2.2 Add a post-packaging macOS step that fails the job unless `dist/Markion.app` passes `codesign --verify --deep --strict` and reports `Signature=adhoc`

## 3. Mirror and update metadata

- [x] 3.1 Extend the `mirror-oss` job for `Markion_${VERSION}_x64.dmg`: mirror-repair download pattern, upload list, reachability verification list, and the `manifest.json` assets map (`macos-x86_64`)
- [x] 3.2 In `src/app/update.rs`, add `("macos", "x86_64") => "_x64.dmg"` to the asset-suffix map and extend the tests with an Intel-mac asset-matching case; verify with `cargo test --bin markion update` (plus `cargo test --workspace` in 5.1)

## 4. Documentation

- [x] 4.1 `README.md` + `README.zh-CN.md`: platform table gains the Intel x64 DMG row; replace the false Rosetta statement; state ad-hoc signing and the resulting Gatekeeper behavior; add the quarantine-free `curl -LO` download alternative alongside the existing `xattr -cr` instructions
- [x] 4.2 `README.zh-TW.md`, `README.ja.md`, `README.fr.md`, `README.de.md`, `README.es.md`: correct the Rosetta sentence and the macOS platform row (factual fix; no new sections)
- [x] 4.3 `docs/faq.md`: platform table row for x86_64, correct the Rosetta note, update the Troubleshooting entry for ad-hoc-signed behavior (Open Anyway works; xattr and curl alternatives), and the platforms paragraph
- [x] 4.4 `docs/release-process.md`: seven-package asset checklist (add the x64 DMG everywhere the six packages are enumerated), mirror contents, and a note on the ad-hoc signing verification step

## 5. Validation

- [x] 5.1 Confirm `cargo check --target x86_64-apple-darwin` passes (or fix target-conditional code), run `cargo test --workspace` and `cargo fmt --check`, and validate the change (`openspec validate improve-macos-distribution --strict` when the CLI is available; otherwise re-read the delta against the spec)
