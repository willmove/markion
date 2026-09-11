# Third-Party Notices

Markion includes the following third-party components relevant to native PDF
viewing.

## pdfium-render 0.9.3

Copyright (c) the pdfium-render contributors.

`pdfium-render` is distributed under the MIT License or the Apache License 2.0,
at the user's option. Markion disables its default features and enables only the
`pdfium_7881` API binding. Source and license texts:
https://github.com/ajrcarey/pdfium-render

## PDFium build 7881

Copyright 2014 The PDFium Authors. All rights reserved.

Markion packages one target-specific, dynamically loaded PDFium runtime from
the non-V8 binary archives published for Chromium/PDFium build 7881 by
`bblanchon/pdfium-binaries`. PDFium is distributed under a BSD-style license;
its source tree also contains third-party components whose notices are
maintained by the upstream PDFium project. Markion does not package V8, XFA,
JavaScript support, headers, import libraries, samples, debug artifacts, or
fonts from the binary archives.

Runtime provenance:
https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F7881

PDFium license and third-party notices:
https://pdfium.googlesource.com/pdfium/+/chromium/7881/LICENSE and
https://pdfium.googlesource.com/pdfium/+/chromium/7881/third_party/

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
