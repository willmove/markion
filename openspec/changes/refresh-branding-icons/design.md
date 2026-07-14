## Context

`assets/markion-logo.svg` is the canonical logo, but the checked-in generator still reads the removed `assets/logo.svg`; generated PNG, ICO, and ICNS files are absent. Windows resource embedding already targets the ICO, while README and package metadata must reference the new canonical assets.

## Goals / Non-Goals

**Goals:**

- Generate deterministic 512px PNG, multi-resolution ICO, and ICNS files from the canonical SVG with the existing Node/Sharp tooling.
- Connect the generated exports to Windows resources, macOS/Linux packaging metadata, and README display.

**Non-Goals:**

- Modify the artwork, application UI, signing/notarization, or release workflow formats.

## Decisions

- Retain the existing `sharp` + `png2icons` script because its dependencies and output formats already match the repository. Change only its source path to `assets/markion-logo.svg`.
- Use `markion-logo.svg` directly in the README to retain scalable, crisp documentation rendering. Use PNG/ICO/ICNS only where platform tooling requires raster or container formats.
- Keep `build.rs` platform-gated: it watches and embeds `assets/markion.ico` only on Windows, avoiding macOS/Linux build dependencies.
- Keep all generated files under `assets/` with stable names consumed by Cargo metadata and packaging, so regenerating them does not require integration edits.

## Risks / Trade-offs

- [Icon conversion lacks a required Node dependency] → run the existing script from the repository and surface its error; do not create substitute assets by hand.
- [A platform cache preserves a prior resource] → `cargo:rerun-if-changed` tracks the ICO, and release builds start from clean artifacts.
- [The source artwork lacks adequate raster padding] → inspect generated dimensions and transparency before accepting the outputs.
