# Markion PDF Plugin — Third-Party Notices

The optional `dev.markion.pdf` package contains the following components. They
are distributed with the plugin, independently from the Markion core package.

## pdfium-render 0.9.3

Copyright (c) the pdfium-render contributors.

`pdfium-render` is distributed under the MIT License or the Apache License 2.0,
at the user's option. The plugin disables default features and enables only the
`pdfium_7881` API binding. Source and license:
https://github.com/ajrcarey/pdfium-render

## PDFium build 7881

Copyright 2014 The PDFium Authors. All rights reserved.

The plugin contains one target-specific, dynamically loaded runtime from the
non-V8 archives published for Chromium/PDFium build 7881 by
`bblanchon/pdfium-binaries`. PDFium is distributed under a BSD-style license
and incorporates third-party components covered by upstream notices. The
package excludes V8, XFA, JavaScript support, headers, import libraries,
samples, debug artifacts, and fonts from those binary archives.

Runtime provenance:
https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F7881

PDFium license and upstream notices:
https://pdfium.googlesource.com/pdfium/+/chromium/7881/LICENSE and
https://pdfium.googlesource.com/pdfium/+/chromium/7881/third_party/
