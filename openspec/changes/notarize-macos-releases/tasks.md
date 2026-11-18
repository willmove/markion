## 0. Prerequisite

- [ ] 0.1 Archive `improve-macos-distribution`, whose renamed release-packaging requirement this delta modifies, before archiving this change.

## 1. Signing-mode selection

- [x] 1.1 Add `scripts/configure-macos-signing.sh` to rewrite the build-time signing identity safely and export the signing and notarization settings. Store the private key outside the source tree with mode 0600.

## 2. Workflow integration

- [x] 2.1 In `.github/workflows/release.yml`, run the configuration script in both macOS jobs before packaging. Map the secrets in through step-level `env:` only.
- [x] 2.2 Add `scripts/notarize-macos-dmg.sh` and a **Notarize and staple DMG** step gated on `notarize == 'true'`. The step submits with `notarytool --wait --timeout 1h`, requires `Accepted` (printing the notary log otherwise), and staples.
- [x] 2.3 Verify ad-hoc signatures in development CI and require Developer ID signatures plus notarization in release jobs.
- [x] 2.4 Update the `packager.toml` signing comments. The committed identity stays `"-"`, and the comments note that CI rewrites it.

## 3. Documentation

- [x] 3.1 Add the `docs/macos-signing.md` maintainer guide: why notarization is required, mode table, certificate and API-key provisioning, secrets, verification run, and maintenance.
- [x] 3.2 Update `docs/release-process.md`: the signing section, the macOS job verification wording, and English and Chinese release-note guidance to state whether the DMGs are notarized.
- [x] 3.3 Correct the Gatekeeper guidance in `README.md` (English and its Chinese section), `README.zh-CN.md`, and `docs/faq.md`. Notarized builds open normally. Non-notarized builds, including all releases up to v0.4.9, are blocked as "damaged", with `xattr -cr` / `curl -LO` as the fallback. Remove the claim that **Open Anyway** always works.
- [x] 3.4 Fix the same claim in `README.zh-TW.md`, `README.ja.md`, `README.fr.md`, `README.de.md`, and `README.es.md`.

## 4. Validation

- [x] 4.1 `openspec validate notarize-macos-releases --strict` and `openspec validate --all --strict --no-interactive`. Parse `release.yml` as YAML. Run `bash -n` on both scripts.
- [x] 4.2 Verify Apple Developer Program enrollment and issue the G2 Developer ID Application certificate. Verify the certificate and its matching private key by producing a timestamped code signature in a temporary keychain.
- [ ] 4.2a Obtain App Store Connect API access and create the team key for notarization.
- [ ] 4.3 Maintainer: add the six repository secrets. Run the release workflow manually on `main` with an empty `mirror_tag`. Confirm that both macOS jobs log `notarize: true` and pass verification, and that a browser-downloaded artifact opens on a Mac without Terminal commands.
- [ ] 4.4 After the first notarized tag release, drop the "every release up to v0.4.9" qualifiers where they no longer help, then archive this change.

## 5. Required public signing and verification

- [x] 5.1 Require all Apple credentials for tags and manual signing rehearsals. Keep branch and pull request packages ad-hoc. Verify missing credentials, invalid credentials, and the full configuration path with test values.
- [x] 5.2 Verify the final DMG and its embedded app signatures, tickets and architecture. The new verifier rejected the release's SHA-256-matched v0.6.1 Apple Silicon DMG on macOS with Gatekeeper enabled. The existing app seal passed verification, but Gatekeeper rejected it and the app had no stapled ticket.
- [x] 5.3 Update the signing and release runbooks to match the gate.
