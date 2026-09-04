## Context

See `proposal.md` for motivation. The About Markion dialog is already a root-hosted modal (`about_dialog_view` in `src/app/root_view.rs`) with a localized title, version line, product description, two official project-link rows (`AboutLink::ProjectWebsite` then `AboutLink::GithubRepository`), and an OK control. Each link row shows a localized label plus the verbatim HTTPS URL and opens that URL through `cx.open_url` without dismissing the dialog.

GitHub does not expose an unauthenticated “star this repository” deep link. Starring still happens on the repository page at `https://github.com/willmove/markion`, which is already `GITHUB_REPO_URL`.

This surface is transient application chrome. It must not enter preferences or session persistence, and it has no relationship to document versions or Markdown-derived state.

## Goals / Non-Goals

**Goals:**

- Place a short localized star invitation immediately after the product description.
- Attach a clickable GitHub link under that invitation that opens the canonical repository URL in the system default browser.
- Reuse the existing About link-row interaction (pointer cursor, underline, theme colors, dialog stays open).
- Route every new user-visible string through `src/i18n.rs` for all supported languages.

**Non-Goals:**

- Authenticating to GitHub or starring from inside Markion.
- A Help-menu “Star on GitHub” item, status-bar reminder, or first-run prompt.
- Changing the existing project-website or GitHub-repository destinations, order, or labels.
- An embedded browser, URL-availability check, or launch-error UI.
- Touching editor, Markdown, preview, or typing-path caches.

## Decisions

### 1. Reuse `GITHUB_REPO_URL` instead of inventing a star-only URL

The star link SHALL open exactly `https://github.com/willmove/markion`. That is where GitHub’s Star control lives. Reusing `GITHUB_REPO_URL` keeps the destination in one constant and matches the existing GitHub row.

Alternative considered: a `stargazers` or hypothetical auto-star URL. Rejected — GitHub requires a signed-in session to star, and no public URL can complete that action for the user. Opening the repository page is the standard desktop-app pattern.

### 2. Render a dedicated invitation block, then a star link row, above the existing official links

Layout inside the About panel:

1. Title
2. Version
3. Product description
4. Localized invitation copy
5. Star link row (localized label + verbatim repository URL)
6. Existing project-website and GitHub rows
7. OK

The invitation is its own muted/body text node (`Msg::DialogAboutStarInvite`), not mixed inline markup. The star row reuses `about_link_row` so it stays visually consistent and independently clickable.

Alternative considered: turn the existing GitHub row into the only star CTA. Rejected because that row is an official repository listing; the user asked for an explicit ask plus an attached star link.

Alternative considered: make “Star” an inline hyperlink inside the invitation sentence. Rejected because this dialog’s links are already label-plus-URL rows with debug selectors and tests; inline mixed text would add a one-off GPUI layout without a clearer user benefit.

### 3. Keep `AboutLink::ALL` as the two official project links

Add `AboutLink::GithubStar` (same URL as `GithubRepository`, distinct label and debug selectors) and render it only in the invitation block. Do **not** append it to `AboutLink::ALL`, so existing ordered-link tests and the website-then-GitHub contract stay intact.

The star row still calls `open_about_link`, which already delegates to `cx.open_url` and leaves `about_dialog_open` true.

### 4. Short, localized copy; keep “Star” as the GitHub term

Canonical Simplified Chinese invitation: `觉得有用的话，欢迎给个 Star，谢谢！`

English invitation: `If Markion helps you, please give it a Star on GitHub. Thank you!`

Star-link label examples: `在 GitHub 上 Star` / `Star on GitHub`. Leave the word “Star” untranslated in every language so it matches GitHub’s control name.

Every new `Msg` variant gets exhaustive arms for En, ZhHans, ZhHant, Ja, Fr, De, and Es. URLs stay constants and are shown verbatim.

## Risks / Trade-offs

- **[Risk] Two About rows open the same GitHub URL** → Acceptable: one is a star CTA with a distinct label, the other remains the official repository listing. Both reuse `GITHUB_REPO_URL`, so they cannot drift.
- **[Risk] Clicking the link does not actually star the repo** → The invitation and link only take the user to the repository page; starring still needs a GitHub session. Show the exact URL as the visible fallback.
- **[Risk] Invitation copy feels pushy or too long in some languages** → Keep one short sentence; do not add dismissible banners or repeated prompts.
- **[Trade-off] Purpose-built invitation block instead of inline markdown** → A few extra view nodes, but it matches the current About interaction model and existing tests.

The About overlay remains presentation-only: opening it, clicking the star link, and dismissing OK MUST NOT read or invalidate document text, derived Markdown caches, syntax-highlight caches, undo snapshots, session values, or preferences.

## Migration Plan

No data migration. Ship the new copy and link with the application update. Rollback is removing the invitation block, `GithubStar` variant, and new `Msg` arms; no persisted data or external contract needs reversal.

## Open Questions

None. Copy and destination are fixed in this design.
