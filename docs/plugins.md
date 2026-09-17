# Optional first-party plugins

Markion keeps features with large native payloads outside the core installer.
The first optional package is **Markion PDF Viewer**. A fresh Markion install
can still discover `.pdf` files and show that a provider is available, but it
does not contain PDFium or download anything merely because a PDF was opened.

## Install and manage a plugin

Open **Preferences → Plugins**. Before an install or update Markion shows the
signed publisher identity, capability and permission declarations, target
compatibility, exact download size, estimated installed size, and whether a
restart is needed. Installation always requires confirmation; document open,
session restore, and file-tree scanning never install code silently.

The manager supports update, disable, re-enable, rollback to one retained
verified version, and uninstall. Download cancellation is cooperative. A
failed download can be retried; a bad signature, digest, manifest, target, or
health check is rejected before activation. Markion keeps the last verified
catalog for offline startup. Installed and rollback bytes are included in the
displayed disk usage, and uninstall reports the reclaimed package storage.

Disabling or uninstalling a provider stops its process and turns its open tabs
into closable provider-unavailable views. Enabling, installing, or rolling back
can reload those tabs without restarting Markion.

## Storage impact

The accepted same-runner host measurements are byte-exact:

| Core format | Installer increase | Installed increase |
|---|---:|---:|
| Windows NSIS | 75,681 B | 215,675 B |
| macOS DMG | 80,141 B | 171,188 B |
| Linux DEB | 85,336 B | 209,396 B |
| Linux AppImage | 135,168 B | 211,796 B |

These are below the release gates of 1 MiB compressed and 2 MiB installed.
Users who do not install PDF support therefore pay roughly 0.07–0.13 MiB in
the installer and 0.16–0.21 MiB on disk for the reusable plugin platform.

The optional PDF package is independently limited to 6 MiB to download and
10 MiB after extraction. The accepted native prototype matrix measured
3,815,994–3,955,016 B compressed and 8,039,787–8,762,516 B installed. The
current real Windows worker measures 4,047,124 B compressed and 8,297,772 B
installed. Native release CI publishes an exact report for every target and
fails if either budget is exceeded.

## Trust and scope

Version 1 accepts only packages and catalogs signed by Markion's dedicated
plugin key. Workers are separate processes, so a crash or protocol failure
does not load native code into the editor process. This is process isolation,
not an operating-system sandbox: permission declarations describe which data
Markion supplies, but cannot constrain all ambient access of a trusted native
program.

This release is deliberately not a third-party marketplace or arbitrary-code
ecosystem. Future first-party publishing integrations—for example Xiaohongshu
or Toutiao—can reuse installation, signing, updates, supervision, progress,
cancellation, and storage management, while adding a separately versioned,
typed publishing capability and host-owned confirmation/credential UI.
