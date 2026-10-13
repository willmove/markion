# macOS Developer ID signing and notarization

This guide is for maintainers. It explains how to make the macOS DMGs open with a plain double-click after a browser download, without Terminal commands and without the "Markion is damaged and can't be opened" dialog.

## Why ad-hoc signing is not enough

Browsers tag every download with the `com.apple.quarantine` attribute. When a quarantined app is first opened, Gatekeeper requires a **Developer ID** signature and an Apple **notarization** ticket. Markion's default build is sealed with a free ad-hoc signature (`[macos] signing-identity = "-"` in `packager.toml`). That signature is valid: the v0.4.9 DMG has an intact seal, a bound Info.plist and matching resource hashes. However, an ad-hoc signature carries no developer identity, so current macOS on Apple silicon refuses the quarantined app and words that refusal as "damaged". macOS 15 removed the Control-click → Open bypass. Without notarization, users must clear the quarantine flag (`xattr -cr`) or download with a tool that does not set it (`curl -LO`).

Notarization requires a paid [Apple Developer Program](https://developer.apple.com/programs/) membership (US$99 per year). Once the secrets below are configured, the release workflow signs, notarizes and staples both macOS DMGs automatically. Without them it keeps producing ad-hoc builds and warns on every tag.

## What the workflow does

`scripts/configure-macos-signing.sh` runs in both macOS jobs (arm64 and the cross-compiled x86_64) and selects a mode:

| Secrets configured | Branch / PR build | `v*` tag or manual run |
| --- | --- | --- |
| none | ad-hoc | ad-hoc, plus a workflow warning |
| Developer ID only | Developer ID | Developer ID (not notarized), plus a workflow warning |
| Developer ID + API key | Developer ID | **Developer ID + notarized + stapled** |
| an incomplete set | job fails | job fails |

In notarizing runs, cargo-packager signs `Markion.app` with the hardened runtime and a secure timestamp. It then notarizes the app with `notarytool` and staples it. Next it builds and signs the DMG, and `scripts/notarize-macos-dmg.sh` notarizes and staples the DMG. The **Verify macOS code signature** step checks the following and fails the job when any check does not hold:

- the `Developer ID Application` authority;
- the team identifier;
- the hardened runtime;
- the stapled tickets on the `.app` and the DMG;
- `spctl` reporting `source=Notarized Developer ID`, where the runner has Gatekeeper enabled.

Fork pull requests receive no secrets and keep building ad-hoc.

## One-time setup

You need a Mac to create the certificate. Only the **Account Holder** of the developer team can create Developer ID certificates.

### 1. Create the Developer ID Application certificate

1. On the Mac, open **Keychain Access → Certificate Assistant → Request a Certificate From a Certificate Authority…**. Enter your email, choose **Saved to disk**, and save the `.certSigningRequest` file.
2. At [developer.apple.com → Certificates](https://developer.apple.com/account/resources/certificates/list), click **+** and choose **Developer ID Application** with the G2 Sub-CA. Then upload the CSR and download the `.cer` file.
3. Double-click the `.cer` file to install it into your login keychain.
4. Confirm the identity and copy its exact name:

   ```bash
   security find-identity -v -p codesigning
   # 1) ABCDEF… "Developer ID Application: Your Name (TEAMID1234)"
   ```

5. In Keychain Access → **My Certificates**, expand the certificate. Right-click it, choose **Export…**, save it as `.p12`, and set a strong export password.
6. Base64-encode the `.p12` file for the secret:

   ```bash
   base64 -i DeveloperID.p12 -o DeveloperID.p12.b64
   ```

### 2. Create an App Store Connect API key for notarization

1. Go to [App Store Connect → Users and Access → Integrations → App Store Connect API](https://appstoreconnect.apple.com/access/integrations/api), open **Team Keys**, and generate a key with the **Developer** role.
2. Download `AuthKey_<KEYID>.p8`. Apple allows this download only once, so store it safely.
3. Note the **Key ID** and the **Issuer ID** shown above the key list.

### 3. Add the repository secrets

Go to **Settings → Secrets and variables → Actions → New repository secret** and add the following secrets. You can also use the `gh` command shown after the table.

| Secret | Value |
| --- | --- |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Your Name (TEAMID1234)`, exactly as `security find-identity` prints it |
| `APPLE_CERTIFICATE` | the contents of `DeveloperID.p12.b64` |
| `APPLE_CERTIFICATE_PASSWORD` | the `.p12` export password |
| `APPLE_API_KEY` | the Key ID, for example `ABC123DEFG` |
| `APPLE_API_ISSUER` | the Issuer ID (a UUID) |
| `APPLE_API_PRIVATE_KEY` | the full text of `AuthKey_<KEYID>.p8`, including the `BEGIN`/`END` lines |

```bash
gh secret set APPLE_SIGNING_IDENTITY --body 'Developer ID Application: Your Name (TEAMID1234)'
gh secret set APPLE_CERTIFICATE < DeveloperID.p12.b64
gh secret set APPLE_CERTIFICATE_PASSWORD
gh secret set APPLE_API_KEY --body 'ABC123DEFG'
gh secret set APPLE_API_ISSUER --body '00000000-0000-0000-0000-000000000000'
gh secret set APPLE_API_PRIVATE_KEY < AuthKey_ABC123DEFG.p8
```

Then delete the local `.p12`, `.b64` and `.p8` copies, or keep them only in a password manager. Never commit them.

### 4. Verify before the next release

1. Open **Actions → release → Run workflow** on `main` and leave `mirror_tag` empty. A manual run notarizes but publishes nothing.
2. Both macOS jobs should log `macOS signing mode: developer-id; notarize: true`, and both should pass **Notarize and staple DMG** and **Verify macOS code signature**.
3. Download the `markion-macos-arm64` artifact on a Mac through a browser, so the download is quarantined. Open the DMG, drag Markion to Applications, and double-click it. You should see at most the standard "downloaded from the Internet" confirmation. To verify from the command line:

   ```bash
   spctl --assess --type execute -vv /Applications/Markion.app   # source=Notarized Developer ID
   xcrun stapler validate /Applications/Markion.app
   ```

After that, tag releases as usual (see [release-process.md](release-process.md)).

## Maintenance

- **Certificate expiry:** Developer ID Application certificates are valid for five years. Once one expires, `codesign` fails and so does the tag build; no unsigned DMG is published. Create a new certificate, then update `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD` and, if the name changed, `APPLE_SIGNING_IDENTITY`. Apps already notarized keep opening after the certificate expires.
- **API key rotation:** revoke the key in App Store Connect, generate a new one, and update `APPLE_API_KEY`, `APPLE_API_ISSUER` and `APPLE_API_PRIVATE_KEY`.
- **Notarization rejected:** the job prints the notary log. The usual causes are an unsigned nested binary, a missing hardened runtime or a missing secure timestamp, all of which cargo-packager handles. Fix the cause and re-run the workflow on the same tag. Do not move the tag.
- **Apple service outage:** a stalled notarization fails the job (the DMG submission gives up after one hour) instead of publishing an un-notarized DMG. Re-run the failed jobs once Apple's [system status](https://developer.apple.com/system-status/) is green.
