## Why

An Omarchy installation from the pacman package crashed while initializing the system Vulkan stack. The same application opened after selecting the Arch Lavapipe driver and disabling implicit Vulkan layers. A native segmentation fault ends the renderer process before the existing Rust startup error handler can recover.

## What Changes

- Make the Arch software Vulkan driver a runtime dependency of the pacman package.
- Give the pacman desktop entry and terminal command one launcher that retries a failed renderer startup on Lavapipe.
- Verify the launcher's retry boundaries and start the installed application under Xvfb in the Arch packaging gate.

## Capabilities

### Modified Capabilities

- `release-packaging` gains software Vulkan recovery and installed-window verification for the pacman package.

## Impact

The pacman package adds a launcher and keeps the Linux binary and existing resource payload under the installation prefix. The startup code reports renderer progress to the launcher. The change concerns startup and packaging.

Non-goals include a renderer replacement and publication of a new release.
