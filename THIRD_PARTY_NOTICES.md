# Third-Party Notices

Markion includes the following third-party components relevant to math rendering.

## RaTeX 0.1.13

Copyright (c) the RaTeX contributors.

RaTeX crates (`ratex-parser`, `ratex-layout`, `ratex-svg`, `ratex-types`, and their
RaTeX dependencies) are distributed under the MIT License. Source and license:
https://github.com/erweixin/RaTeX

## KaTeX fonts

RaTeX embeds KaTeX math font files through `ratex-katex-fonts`. Those fonts are
distributed under the SIL Open Font License 1.1. Font provenance and the full
license text are included in the `ratex-katex-fonts` crate source published on
crates.io and at https://github.com/erweixin/RaTeX.

No font files are modified by Markion.

## Local MarkNice publishing workspace

The packaged browser workspace is derived from MarkNice commit
`c009c1ec7e7c92f89afa5a32edcb126b5296bda7` and is distributed under the MIT
License. It includes pinned browser builds of marked 15.0.12 (MIT), MathJax
3.2.2 (Apache-2.0), html-docx-js 0.3.1 (MIT), and JSZip 3.10.1 (MIT).
Exact provenance, SHA-256 digests, and complete license texts are shipped in
`assets/marknice-workspace/`.

## Software Vulkan driver in the Linux AppImage

The Linux x86_64 AppImage ships a software Vulkan stack in
`usr/lib/markion-vulkan/`, built by `scripts/build-lavapipe.sh` and used only
when the host Vulkan stack cannot start Markion's renderer. Each component's
license text is shipped in `usr/lib/markion-vulkan/licenses/`, and exact
versions in `usr/lib/markion-vulkan/VERSIONS`.

- Mesa 24.3.4 lavapipe (`libvulkan_lvp.so`), MIT License,
  https://mesa3d.org — `licenses/mesa-license.rst`.
- LLVM 15.0.7, statically linked into lavapipe, Apache License 2.0 with LLVM
  Exceptions, https://llvm.org — `licenses/llvm-copyright.txt`.
- Khronos Vulkan loader 1.3.296 (`libvulkan.so.1`), Apache License 2.0,
  https://github.com/KhronosGroup/Vulkan-Loader —
  `licenses/vulkan-loader-LICENSE.txt`.
- Ubuntu 22.04 builds of libxcb extension libraries (`libxcb-present`,
  `libxcb-randr`, `libxcb-shm`, `libxcb-sync`, `libxcb-xfixes`) and
  `libxshmfence`, MIT/X11 licenses, and ncurses `libtinfo`, MIT-style license —
  `licenses/<package>-copyright.txt`.

None of these components are modified by Markion.
