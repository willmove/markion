## 1. Gallery and document opening

- [x] 1.1 Replace the Help action with the in-app thumbnail gallery, including scrolling, keyboard navigation, and localized feedback. Exercise the Help action in the app.
- [x] 1.2 Open the selected sample as a named draft tab and resolve its bundled images and Markdown links. Verify the original document and workspace remain available, and that the draft has no installed save path.
- [x] 1.3 Update the welcome text and bundled index, then run the example verifier.

## 2. Verification

- [x] 2.1 Run the focused app tests, a Windows build, formatting and strict OpenSpec validation.

## Verification record

- In a Windows installation fixture with spaces, Chinese characters, and a comma in its path, opened the gallery from the Help menu, scrolled the cards, and opened the image guide and meeting notes in their own tabs. The guide rendered its bundled image. Switched back to the edited workspace document and undid its unsaved change.
- The focused GPUI test exercised the Help action, card and keyboard activation, bundled Markdown links, draft tab reuse, resource resolution, and open document preservation. The application test suite reported 942 passed and two ignored.
- The Windows build, Rust formatting, example resource verification, and strict OpenSpec validation passed. The staged resource payload matched all 19 bundled files.
