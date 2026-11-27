## Why

Users who choose Help → Examples and Templates want to see the available Markdown documents and open one in Markion. The current action sends them to an external file manager and leaves them to locate and open a file.

## What Changes

- Open a scrollable gallery of bundled Markdown thumbnails inside the Markion window.
- Open the selected Markdown in an editable tab with its own save destination and access to the bundled document's relative links and images.
- Keep the current workspace and its documents available while browsing the gallery and opening samples.
- Update the welcome document and example index to describe the gallery.

## Capabilities

### Modified Capabilities

- `bundled-examples` gains an in-app thumbnail gallery and document opening from the gallery.

## Impact

The shared Help action, GPUI window, tab presentation, and bundled example instructions change. Thumbnails are built when the gallery opens, outside the active document's derived caches.

Non-goals include a template registry and changes to the package layout.
