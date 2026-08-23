## Context

See `proposal.md` for motivation. The current `about` handler in `src/app/search.rs` formats the version and repository URL into `Msg::DialogAboutDetail`, then passes that value to `window.prompt`. GPUI's prompt detail is plain text, so the visible repository URL cannot carry an activation handler and the prompt cannot express two ordered link rows.

The root view already hosts theme-aware modal overlays such as Preferences, while external navigation elsewhere in the application uses `cx.open_url`. The About surface is transient application chrome: it must not enter preferences or session persistence, and it has no relationship to document versions or Markdown-derived state.

## Goals / Non-Goals

**Goals:**

- Render an in-app About modal that preserves the existing information and localized confirmation flow while supporting two genuinely interactive URL rows.
- Keep link order and destinations centralized and independently testable.
- Match the active theme and the existing exhaustive localization model.
- Keep the modal open after either link is launched so the user can inspect or activate the other link before confirming.

**Non-Goals:**

- Generalize a reusable modal framework or change other `window.prompt` call sites.
- Add an embedded browser, URL availability check, navigation history, or browser-launch error UI.
- Persist About-dialog visibility or touch editor, Markdown, preview, or typing-path caches.

## Decisions

### 1. Replace the plain-text prompt with a root-hosted About modal

Add transient `about_dialog_open` state to `MarkionApp`. The existing About action will set it, close the active menu, retain the current About status feedback, and request a render. `root_view.rs` will conditionally add a full-window modal overlay with an occluding, theme-aware panel, the current version and description, the two link rows, and the existing localized OK control. Activating OK sets the state to false; activating a link does not.

This follows the Preferences overlay pattern and permits per-row mouse handlers. Keeping `window.prompt` and representing destinations as extra prompt buttons was rejected because prompt buttons are not URL rows, cannot preserve the requested vertical website-before-GitHub presentation, and dismiss the prompt as part of choosing a result.

### 2. Keep destinations as canonical constants and render them from one ordered model

Add `MARKION_PROJECT_WEBSITE_URL = "https://markion.app"` next to `GITHUB_REPO_URL`. The About renderer will build its two rows from one ordered definition: project website first, GitHub repository second. The visible literal URL and the value passed to `cx.open_url` will come from the same constant, preventing label/target drift.

Each URL element will use pointer cursor, link color, underline, hover treatment, and a stable debug selector. A row handler calls `cx.open_url` synchronously; this only delegates to the platform shell and performs no network work inside Markion.

Alternatives considered were duplicating URL literals in rendering callbacks and using the existing Online Documentation URL for the website. Both were rejected: duplication can drift, and the requested canonical website is distinct from the GitHub README destination.

### 3. Split the monolithic About detail translation into composable localized fields

Replace or retire the current `DialogAboutDetail` template in favor of distinct messages for the version line, product description, project-website label, and GitHub label, while reusing `DialogAboutTitle` and `DialogButtonOk`. Every new `Msg` variant will be added to every supported language's exhaustive match. URLs remain constants and are displayed verbatim rather than entering translation strings.

This avoids embedding layout and URLs inside newline-delimited translated prose. Retaining a single formatted body was rejected because it recreates the plain-text limitation and makes link ranges fragile across languages.

### 4. Keep About state isolated from document and persisted application state

The state and event flow is:

`Help → About` action → set `about_dialog_open` and clear the menu → root render adds modal → link pointer event delegates a constant URL to the platform shell → modal remains open → OK clears `about_dialog_open`.

No document text, derived Markdown cache, syntax-highlight cache, undo snapshot, session value, or preference value is read or invalidated by this flow. The overlay occludes underlying application controls so clicks do not leak through to the editor.

## Risks / Trade-offs

- **[Risk] The custom modal diverges visually from GPUI's platform prompt** → Reuse the active Markion palette and the existing Preferences overlay's spacing, border, shadow, occlusion, and control patterns; cover both light and dark themes in focused rendering tests.
- **[Risk] A missing system browser or broken OS URL association can make `open_url` appear to do nothing** → Continue displaying the exact URL as a visible fallback; GPUI does not report a launch result, so new error UI is outside this change.
- **[Risk] Modal state could overlap another transient surface** → Close the active menu when opening About, render the About panel in the root overlay layer, and make its full-window host occluding so underlying content cannot receive pointer input.
- **[Trade-off] A purpose-built About modal adds a small amount of view/state code** → The scope remains local and avoids prematurely generalizing all prompts while meeting the required link semantics.

## Migration Plan

No data migration is required. Deploy the new modal and translations with the application update. Rollback consists of restoring the previous `window.prompt` handler and removing the transient state/view; no persisted data or external contract needs reversal.
