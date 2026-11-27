## Context

Both Help menus already share one action. The bundle contains a Markdown index, numbered guides and templates, and relative links and images.

## Decisions

- Host the gallery in the main GPUI window. Render each card as a small page with text extracted from its Markdown. Read and summarize the collection once per gallery opening and keep the summaries in the gallery state.
- Reuse the normal Markdown editor and new-tab flow. Read the chosen sample into an editable document with no save path, and retain the bundled source path on the tab for its title and resource resolution until the first save. This keeps the bundle outside the autosave path.
- Open relative sample Markdown links with the same draft behavior. A repeated selection focuses the existing draft, including edits the user has made. The regular tab lifecycle handles the prior document.
- Give the gallery its own keyboard focus and scrolling. Escape or the close button restores editor focus. Missing resources produce localized feedback in the window.
