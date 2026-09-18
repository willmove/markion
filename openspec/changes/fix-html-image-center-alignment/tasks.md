## 1. Shared HTML image layout

- [x] 1.1 Make the centered/right-aligned HTML image wrapper fill the rendered content width before applying flex justification, preserving image dimensions, cache behavior, and left-aligned behavior; verify with the focused HTML image regression test.

## 2. Verification

- [x] 2.1 Run the targeted parser/application tests and `openspec validate fix-html-image-center-alignment`; verify the supplied `<p align="center"><img ...></p>` case remains centered through the shared Read, Split Preview, and Visual Edit HTML rendering path.
