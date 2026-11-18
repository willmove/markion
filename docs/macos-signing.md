# macOS Developer ID signing and notarization

This guide is for maintainers. It explains how to make the macOS DMGs open with a plain double-click after a browser download, without Terminal commands and without the "Markion is damaged and can't be opened" dialog.

## Why ad-hoc signing is not enough

Browsers tag every download with the `com.apple.quarantine` attribute. When a quarantined app is first opened, Gatekeeper requires a **Developer ID** signature and an Apple **notarization** ticket. Markion's default build is sealed with a free ad-hoc signature (`[macos] signing-identity = "-"` in `packager.toml`). That signature is valid: the v0.4.9 DMG has an intact seal, a bound Info.plist and matching resource hashes. However, an ad-hoc signature carries no developer identity, so current macOS on Apple silicon refuses the quarantined app and words that refusal as "damaged". macOS 15 removed the Control-click → Open bypass. Without notarization, users must clear the quarantine flag (`xattr -cr`) or download with a tool that does not set it (`curl -LO`).

Notarization requires a paid [Apple Developer Program](https://developer.apple.com/programs/) membership. The release workflow requires the secrets below before it can publish. It signs, notarizes and staples both macOS DMGs automatically.

## What the workflow does

`scripts/configure-macos-signing.sh` runs before packaging both architectures on tagged releases and manual signing rehearsals.

| Secrets configured | Branch / PR build | `v*` tag or manual signing rehearsal |
| --- | --- | --- |
| complete | ad-hoc | **Developer ID + notarized + stapled** |
| missing any | ad-hoc | job fails before packaging |

In signing runs, cargo-packager signs `Markion.app` with the hardened runtime and a secure timestamp. It then notarizes the app with `notarytool` and staples it. Next it builds and signs the DMG, and `scripts/notarize-macos-dmg.sh` notarizes and staples the DMG. The **Verify macOS code signature** step mounts the final DMG read-only and verifies:

- Developer ID Application signatures and stapled tickets on the DMG and its embedded app
- The embedded app's team identifier, hardened runtime, timestamp and expected executable architecture
- `spctl` reporting `source=Notarized Developer ID` for the DMG and its embedded app when Gatekeeper assessments are enabled

Branch pushes and pull requests build ad-hoc without reading Apple secrets.

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

1. Go to [App Store Connect → Users and Access → Integrations → App Store Connect API](https://appstoreconnect.apple.com/access/integrations/api). If prompted, the Account Holder first requests API access. Open **Team Keys** and generate a key with the **Developer** role.
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
3. Download the matching macOS artifact through a browser on a Mac with Gatekeeper assessments enabled. Open the DMG, drag Markion to Applications, and double-click it. You should see at most the standard "downloaded from the Internet" confirmation. Verify the DMG and its embedded app from the command line:

   ```bash
   bash scripts/verify-macos-release.sh /path/to/Markion_X.Y.Z_aarch64.dmg arm64
   bash scripts/verify-macos-release.sh /path/to/Markion_X.Y.Z_x64.dmg x86_64
   ```

Follow the [release process](release-process.md) for the stable tag and repeat verification on the public downloads.

## Maintenance

- **Certificate expiry.** Check the expiry shown in Apple Developer and rotate before that date. Create a new G2 certificate and update `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD` and, if the name changed, `APPLE_SIGNING_IDENTITY`. A secure timestamp preserves the validity of existing signatures after the signing certificate expires. [Apple’s Developer ID renewal guidance](https://developer.apple.com/news/?id=w4atic4c).
- **API key rotation:** revoke the key in App Store Connect, generate a new one, and update `APPLE_API_KEY`, `APPLE_API_ISSUER` and `APPLE_API_PRIVATE_KEY`.
- **Notarization rejected.** The job prints the notary log. If the problem was credentials or an Apple service outage, correct the issue and re-run the failed jobs. If it requires a repository change, fix it in a subsequent release commit under a fresh tag.
- **Apple service outage:** a stalled notarization fails the job (the DMG submission gives up after one hour) instead of publishing an un-notarized DMG. Re-run the failed jobs once Apple's [system status](https://developer.apple.com/system-status/) is green.
