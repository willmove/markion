# Proposal: prefer-appimage-update-on-arch

## Why

The "Check for Updates…" dialog maps every Linux x86_64 machine to the `.deb` asset (`update.rs` `browser_download_url` hardcodes `_amd64.deb`). On pacman-based distributions such as Arch Linux and Omarchy — where the PDF font fidelity of this project was actually verified — the offered artifact does not match the system's package manager; the portable `.AppImage` is the artifact Arch users can actually use.

## What Changes

- On Linux x86_64, the manual-download asset URL SHALL be selected by distribution family: pacman-based systems (detected via `/etc/os-release`, including `arch`, `omarchy`, `manjaro`, and other Arch descendants) get the `.AppImage` URL, while deb-based systems keep the `.deb` URL.
- The Windows signed-install path, macOS mapping, up-to-date logic, and the Release-page fallback for unknown platforms are unchanged.

**Non-goals:** pacman/`.pkg.tar.zst` package production; AUR publication; changes to the signed updater endpoint list; any installer-format additions.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `release-packaging`: the update-check requirement's platform-asset mapping gains the Linux distribution-family rule — pacman-based systems are offered the AppImage asset, deb-based systems the DEB asset, and anything else the Release page.

## Impact

- **Code**: `src/app/update.rs` (`browser_download_url` gains an os-release-aware Linux branch behind an injectable helper so tests stay hermetic; no new dependencies, no i18n changes).
- **Docs**: none required beyond the spec delta.
