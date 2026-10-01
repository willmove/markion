# Pending deletion (NOT archived)

Changes parked here are **superseded or abandoned**. They were never archived:
their delta specs must **not** be synced into `openspec/specs/` — the
replacement change owns the capability's evolution. Kept only as short-term
reference; delete each folder once its replacement is verified.

This folder lives under `changes/archive/` because `archive` is the only
subdirectory the OpenSpec CLI consistently excludes from change discovery
(`openspec list`, `doctor`, workflows). Do not move it up one level — it would
be listed as an active change again.

## Contents

| Change | Status | Replaced by |
| --- | --- | --- |
| `table-toolbar-hover-dwell` | Superseded 2026-09-30 | `notion-style-visual-table-handles` (shipped in v0.4.5, commit af2838d) |
| `fix-visual-table-toolbar-targeting` | Superseded 2026-10-01 | `notion-style-visual-table-handles` (archived 2026-10-01) |

`table-toolbar-hover-dwell`'s subject — the in-flow Visual Edit table header
toolbar and its 250 ms dwell / 120 ms hide timers — was removed from the
codebase entirely by the Notion-style row/column handles rework. Its delta spec
describes a toolbar that no longer exists. See the successor's `proposal.md`
("Supersedes" section).

`fix-visual-table-toolbar-targeting` made the same header toolbar's buttons
target the caret's table. Its only delta MODIFIES "GFM table rendering with
row/column toolbar editing", which the successor's archive REMOVED from
`tables-outline`, so it cannot be synced. Index-addressed targeting with
revalidation is now specified by "GFM table rendering with row/column handle
editing".
