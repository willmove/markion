# Design: prefer-appimage-update-on-arch

## Context

`browser_download_url` in `src/app/update.rs` maps `("linux", "x86_64")` to the `_amd64.deb` asset suffix unconditionally. Every release also ships a portable `_x86_64.AppImage`, which is the only artifact pacman-based systems (Arch, Omarchy, Manjaro, EndeavourOS, …) can use without forcing a foreign package format through the distro's package manager. The update-available dialog and its "Download Update" action are the only places this mapping applies.

## Goals / Non-Goals

**Goals:**

- Linux x86_64 users on pacman-based distributions are offered the AppImage; deb-based users keep the DEB.
- Hermetic tests: the distribution detection is a pure function over `/etc/os-release` contents.

**Non-Goals:**

- Producing pacman packages or publishing an AUR package.
- Changing the signed Windows updater path, the macOS mapping, or the OSS/GitHub endpoint list.
- Any new user-facing strings (the dialog and buttons are unchanged; only the offered URL changes).

## Decisions

### Detect the distribution family from `/etc/os-release`

At update-check time, read `/etc/os-release` and treat the system as Arch-family when the `ID` value or any token of the space-separated `ID_LIKE` list equals `arch` (this covers `ID=arch`, `ID=manjaro` + `ID_LIKE=arch`, `ID=endeavouros` + `ID_LIKE=arch`, and Omarchy, whose `os-release` identifies an Arch descendant). Arch-family → `_x86_64.AppImage`; anything else — including a missing or unparseable file — keeps the `_amd64.deb` behavior of today. `/etc/os-release` is mandated by systemd and present on every mainstream desktop distribution, so the fallback default is rare and conservative.

Alternative considered: always offer the AppImage on Linux — rejected because deb users genuinely want the DEB (desktop entry, uninstall path); offer both URLs — rejected because the dialog offers a single primary download and duplicating URLs reintroduces the truncation problem that motivated button-based prompts.

### Pure, injectable selection function

The selection logic lives in a function taking the parsed os-release text as `Option<&str>` (plus the existing `os`/`arch`), so unit tests cover Arch, Manjaro, Debian, and a missing file without touching the filesystem; the production path reads `/etc/os-release` once per update check. Existing cross-platform tests keep passing through the unchanged wrapper signature.

## Risks / Trade-offs

- [A pacman-based system whose `os-release` lacks `arch` in `ID`/`ID_LIKE` still gets the DEB] → the Release page remains one click away via the manual-download fallback; the AppImage is also linked from the Release page and OSS mirror.
- [Behavior differs per Linux distro, which is harder to support] → the mapping is a single pure function with a table of fixture-based tests; misclassification degrades to today's behavior.

## Open Questions

None.
