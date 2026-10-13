## 0. Prerequisite

- [ ] 0.1 Archive `improve-macos-distribution`, whose renamed release-packaging requirement this delta modifies, before archiving this change.

## 1. Signing-mode selection

- [x] 1.1 Add `scripts/configure-macos-signing.sh`. It selects `adhoc`, `developer-id`, or `developer-id` + notarize from the six Apple secrets and the ref: notarize only on `v*` tags and `workflow_dispatch`. It rewrites the working copy's `signing-identity` with a TOML-safe value and exports the cargo-packager variables via `$GITHUB_ENV` only when set. It writes the `.p8` key with mode 0600, fails on partial secret sets or on a non-`Developer ID Application:` identity, and warns on un-notarized tag builds. Verified locally against 10 secret/ref combinations: modes, exported variable names, identity rewrite with quotes, key-file mode, and error/warning messages.

## 2. Workflow integration

- [x] 2.1 In `.github/workflows/release.yml`, run the configuration script in both macOS jobs before packaging. Map the secrets in through step-level `env:` only.
- [x] 2.2 Add `scripts/notarize-macos-dmg.sh` and a **Notarize and staple DMG** step gated on `notarize == 'true'`. The step submits with `notarytool --wait --timeout 1h`, requires `Accepted` (printing the notary log otherwise), and staples.
- [x] 2.3 Replace the ad-hoc-only gate with a mode-aware **Verify macOS code signature** step:
  - ad-hoc mode requires `Signature=adhoc`;
  - Developer ID mode requires the authority, team identifier, hardened runtime, and the DMG signature;
  - notarize mode requires `stapler validate` on the `.app` and DMG, plus `spctl` `source=Notarized Developer ID` when the runner's Gatekeeper is enabled.
- [x] 2.4 Update the `packager.toml` signing comments. The committed identity stays `"-"`, and the comments note that CI rewrites it.

## 3. Documentation

- [x] 3.1 Add the `docs/macos-signing.md` maintainer guide: why notarization is required, mode table, certificate and API-key provisioning, secrets, verification run, and maintenance.
- [x] 3.2 Update `docs/release-process.md`: the signing section, the macOS job verification wording, and English and Chinese release-note guidance to state whether the DMGs are notarized.
- [x] 3.3 Correct the Gatekeeper guidance in `README.md` (English and its Chinese section), `README.zh-CN.md`, and `docs/faq.md`. Notarized builds open normally. Non-notarized builds, including all releases up to v0.4.9, are blocked as "damaged", with `xattr -cr` / `curl -LO` as the fallback. Remove the claim that **Open Anyway** always works.
- [x] 3.4 Fix the same claim in `README.zh-TW.md`, `README.ja.md`, `README.fr.md`, `README.de.md`, and `README.es.md`.

## 4. Validation

- [x] 4.1 `openspec validate notarize-macos-releases --strict` and `openspec validate --all --strict --no-interactive`. Parse `release.yml` as YAML. Run `bash -n` on both scripts.
- [ ] 4.2 Maintainer: enroll in the Apple Developer Program, then create the Developer ID Application certificate and the App Store Connect API key per `docs/macos-signing.md`.
- [ ] 4.3 Maintainer: add the six repository secrets. Run the release workflow manually on `main` with an empty `mirror_tag`. Confirm that both macOS jobs log `notarize: true` and pass verification, and that a browser-downloaded artifact opens on a Mac without Terminal commands.
- [ ] 4.4 After the first notarized tag release, drop the "every release up to v0.4.9" qualifiers where they no longer help, then archive this change.
