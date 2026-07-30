## Context

See `proposal.md` for motivation. The current checker in `src/app/update.rs` fetches GitHub's latest Release, selects the platform asset, and passes its full URL to GPUI's native prompt as plain detail text. GPUI 0.2.2 maps that prompt to a Windows `TaskDialogIndirect`, whose plain content truncates the unbroken URL and provides no link activation. Tagged builds already publish a current-user NSIS installer and mirror it to Aliyun OSS, but `manifest.json` contains filenames only and the published assets have no updater signature.

`cargo-packager` 0.11.8 has no `[updater]` configuration field, but its separate `cargo-packager-updater` 0.2.3 library accepts a release endpoint and a base64-encoded Minisign public key, verifies the manifest signature before installation, and supports passive NSIS installation. The existing build remains unsigned at the operating-system level.

## Goals / Non-Goals

**Goals:**
- Make every update-available prompt actionable without displaying a raw URL.
- Authenticate the Windows updater payload with a release-controlled Minisign key.
- Preserve manual download on every platform and whenever signed installation is unavailable or fails.
- Keep blocking network, signature, and installer work off the GPUI render path.
- Prevent updater-triggered process exit from bypassing unsaved-document protection.

**Non-Goals:**
- Platform code signing, notarization, background updates, forced restarts, or automatic rollback.
- macOS or Linux self-replacement in this change.
- Replacing GitHub Releases as the version source of truth.

## Decisions

### Use a native prompt action instead of rendering or widening the URL

The available-version prompt will contain only the version summary and two localized buttons. The primary button is `Download and Install` when the Windows signed updater is configured and `Download Update` otherwise; the secondary button is `Later`. The handler awaits the prompt response. Browser-fallback paths call GPUI's existing `open_url` support with the already-selected asset URL.

Alternative considered: create a custom rich-text prompt with wrapping, selection, and hyperlinks. Rejected because a button is clearer, works with every native prompt implementation, and avoids a new dialog component solely for a URL.

### Limit automatic installation to Windows x86_64 NSIS

The root package will depend on `cargo-packager-updater` only under `cfg(target_os = "windows")`. A Windows release with a non-empty compile-time public key uses its `check_update` and `download_and_install` flow with passive NSIS mode. macOS and Linux use the browser fallback.

Alternative considered: enable all updater formats immediately. Rejected because the present macOS artifact is unsigned/notarized and Linux users may be running either a `.deb` installation or an AppImage from a non-writable path. Those platforms need separate installation-origin and trust decisions.

### Keep GitHub as discovery and use a stable updater manifest endpoint

The existing GitHub latest-release API remains responsible for announcing a newer version and selecting the manual-download asset. The signed Windows path separately reads `https://github.com/willmove/markion/releases/latest/download/update.json`. The manifest points to the OSS-mirrored NSIS installer so the large payload uses the domestic mirror, while the small authoritative metadata remains attached to GitHub Releases.

If signed installation fails, the already-known GitHub asset URL is retained for a `Download Manually` prompt action.

Alternative considered: replace all discovery with the OSS manifest. Rejected because GitHub remains the release source of truth and the current mirror is a latest-only cache; retaining independent discovery also makes mirror failures recoverable.

### Inject the public key at compile time and keep private material in release secrets

Windows builds read `MARKION_UPDATE_PUBLIC_KEY` via `option_env!`. The release workflow maps it from `CARGO_PACKAGER_SIGN_PUBLIC_KEY`; branch and pull-request builds may omit it and automatically use browser download. A tag-only `prepare-update` job reads `CARGO_PACKAGER_SIGN_PRIVATE_KEY` and `CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD`, signs the exact NSIS output with `cargo packager signer sign`, and fails rather than publish incomplete updater metadata when any key is missing.

The public key is intentionally embedded in the binary; it authenticates future manifests and payloads. The private key is never written to the repository or uploaded as an artifact.

Alternative considered: commit the public key. Rejected for this change because no production keypair exists in repository state; compile-time injection lets the maintainer establish or rotate the production key without committing placeholder identity material.

### Reuse cargo-packager's updater manifest format

The generated `update.json` contains `version`, `pub_date`, and a `platforms.windows-x86_64` entry with `url`, the exact `.sig` file content, and `format: "nsis"`. Both the GitHub Release and OSS mirror receive `update.json` and the `.sig` file. The existing human-auditable SHA-256 list remains, but SHA-256 does not replace signature verification.

### Gate installation on clean document state

The prompt response returns to the app context before any updater work begins. If any tab is dirty, Markion shows a save-first warning and stops. Clean-state updater work runs through GPUI's background executor; the updater library exits the process only after launching the passive NSIS installer. Failure returns to the app context, updates status, and offers manual download.

No document-derived caches are read or invalidated:

```text
GitHub release check ──▶ available-version prompt
                              │
                 ┌────────────┴────────────┐
                 │                         │
        browser fallback          Windows signed path
                                           │
                                    dirty-tab gate
                                           │ clean
                                    background updater
                                           │
                              manifest → download → verify
                                           │
                                  launch NSIS → exit
```

## Risks / Trade-offs

- **[Risk] Tag publication begins without configured secrets** → The tag-only metadata job validates all three signing values and fails the release gate before publishing updater metadata.
- **[Risk] OSS latest content and GitHub latest metadata briefly disagree during publication** → Automatic failure preserves the running app and exposes the immutable GitHub asset as a manual fallback; release completion still requires both publication jobs.
- **[Risk] Minisign is mistaken for platform signing** → Documentation and UI call it update verification; existing SmartScreen warnings remain documented.
- **[Risk] The updater library buffers the installer in memory** → Accepted for the Windows-only first iteration; status clearly indicates download/verification and the work stays off the render thread.
- **[Risk] An updater library process exit bypasses editor confirmation** → The dirty-tab gate runs before starting the updater and prevents automatic installation whenever any document is unsaved.

## Migration Plan

1. Generate a production cargo-packager signing key outside the repository.
2. Configure the three repository secrets before the first tagged release containing this change.
3. Publish a tag; require native builds, signed update-metadata preparation, GitHub Release publication, and OSS mirror upload to succeed.
4. Verify `update.json`, the NSIS `.sig`, the public-key-enabled Windows binary, successful signature verification, passive installer launch, and the manual fallback.
5. Roll back a faulty updater implementation by publishing a fixed higher patch version; do not move or delete a public tag.
