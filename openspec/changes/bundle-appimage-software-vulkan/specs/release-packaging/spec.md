## ADDED Requirements

### Requirement: Linux AppImage SHALL start without a host Vulkan driver
The Linux x86_64 AppImage SHALL contain a software Vulkan implementation and a Vulkan loader, built for the same C-library baseline as the application. When the host Vulkan stack cannot initialize Markion's renderer, the AppImage SHALL start on the bundled software implementation and show a usable main window without any user configuration or extra system packages.

#### Scenario: Host without any Vulkan driver opens a window
- **WHEN** the AppImage is launched on an X11 desktop (including Xvfb under `firejail --appimage`) that has no Vulkan driver installed and no GPU
- **THEN** the main Markion window appears and renders its content
- **AND** the diagnostic log records that the bundled software Vulkan driver is in use

#### Scenario: Host without a Vulkan loader still starts
- **WHEN** the AppImage is launched on a host that has neither a Vulkan loader nor a Vulkan driver
- **THEN** Markion starts on the bundled loader and software driver and shows its main window

#### Scenario: Bundled stack does not raise the C-library baseline
- **WHEN** the release pipeline assembles the AppImage
- **THEN** no bundled software-Vulkan file requires a newer glibc symbol version than the application binary itself

### Requirement: Hardware Vulkan SHALL stay preferred over the bundled software driver
The AppImage SHALL initialize the host Vulkan stack first and SHALL load the bundled software Vulkan stack only after host initialization has failed, at most once per launch. Users SHALL be able to force the bundled stack or disable the fallback through a documented environment variable.

#### Scenario: Working GPU driver keeps hardware rendering
- **WHEN** the AppImage is launched on a host whose Vulkan driver initializes successfully
- **THEN** Markion renders through the host driver and does not load the bundled software stack

#### Scenario: User forces or disables the software fallback
- **WHEN** the AppImage is launched with `MARKION_SOFTWARE_VULKAN=1`
- **THEN** Markion uses the bundled software stack without first trying the host driver
- **AND** when launched with `MARKION_SOFTWARE_VULKAN=0`, a host initialization failure is reported instead of falling back

#### Scenario: Fallback failure is reported, not retried forever
- **WHEN** renderer initialization also fails on the bundled software stack
- **THEN** Markion reports the renderer failure with remediation guidance and exits with a non-zero status without restarting again

### Requirement: Release pipeline SHALL verify that the AppImage opens a window without a GPU
Before a Linux AppImage is published, the release workflow SHALL launch it the way the AppImage catalog does — Xvfb with a window manager, `firejail --appimage`, on ubuntu-22.04 with no Vulkan driver installed — and SHALL fail unless a visible Markion window appears while the process is still running. The AppImage SHALL also keep its world-readable permissions and its updater signature SHALL be computed over the final file that contains the bundled stack.

#### Scenario: Catalog-equivalent launch succeeds
- **WHEN** the Linux build job has produced the AppImage
- **THEN** the smoke job finds a visible Markion window and a non-blank screenshot is uploaded as a workflow artifact

#### Scenario: Regression blocks publication
- **WHEN** the launched AppImage exits early or shows no visible window
- **THEN** the smoke job fails and no GitHub Release is published for that run

#### Scenario: Bundling keeps permissions and signatures valid
- **WHEN** the software Vulkan stack is added to the AppImage
- **THEN** AppRun, the application binary and every directory remain world-executable and every file world-readable
- **AND** the tagged-release `.sig` file and `update.json` entry are produced from that final AppImage
