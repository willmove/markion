## 1. Windows font fallback

- [x] 1.1 Resolve missing Windows families through explicit candidates and system UI fallback, and verify both paths with native DirectWrite using a deliberately missing family.

## 2. Quoted table editing

- [x] 2.1 Retain table ownership and render the grid inside its quote in Visual Edit, Read, and Split Preview. Verify quote child order and preview cell selection.
- [x] 2.2 Map quoted table cells and inline breaks to authored bytes and preserve those bytes outside each edit. Verify Unicode, CRLF, alignment, and column-width metadata with focused tests.
- [x] 2.3 Exercise the actual grip menu Quote action, cell typing, and one-step undo in a rendered Visual Edit regression. Run the Windows tests and validate the change.
