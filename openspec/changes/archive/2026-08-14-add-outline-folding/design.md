## Context

See `proposal.md` for the motivation and `specs/tables-outline/spec.md` for the behavior contract. The outline panel currently asks the active `MarkdownDocument` for its cached heading vector and renders one indented GPUI row per heading. The row itself owns the existing navigation listener. `Heading` supplies a level, title, anchor, and source offset, but it does not encode parent/child links; duplicate titles and anchors are valid.

Folding is presentation-only state. It must remain isolated with the document tab and must not mutate the document or invalidate the per-version `Arc`-shared preview/outline/stat caches, memoized syntax highlighting, or cached text handle.

```text
cached heading vector + per-tab collapsed node keys + canonical active index
                              |
                              v
                 linear outline projection
            (tree relations, visibility, disclosure,
                 visible active representative)
                              |
                              v
                     visible GPUI rows
                       /             \
              disclosure click     label click
                    |                    |
       update per-tab fold state   existing context-aware
       and request a re-render     outline navigation
```

## Goals / Non-Goals

**Goals:**

- Derive tree visibility from the existing flat heading order in one linear pass without parsing Markdown again.
- Keep folding state per document tab and stable across ordinary body edits during the tab's lifetime.
- Separate disclosure activation from heading navigation so each click has one deterministic effect.
- Preserve an understandable active-section signal when the exact active heading is hidden.

**Non-Goals:**

- Changing `Heading`, Markdown parsing, source ranges, or the cached derived-outline contract.
- Persisting UI folding state in preferences, session recovery, or document files.
- Reusing outline folds to hide document content in any editing or preview mode.
- Introducing keyboard tree semantics in this change.

## Decisions

### Project a tree view from the cached flat outline

A pure projection helper will scan the current heading vector once. A row has descendants when the next heading has a deeper level; a collapsed row hides subsequent deeper rows until the scan reaches the same or a shallower level. A small stack of visible ancestors is sufficient to determine visibility, disclosure state, and the visible representative for the canonical active heading.

This keeps the Markdown core and cached `Heading` value unchanged. Adding parent indexes or a second parse-derived tree to `MarkdownDocument` was rejected because folding is only a sidebar presentation concern and the flat order already contains the necessary structure. Recursively searching descendants for every row was rejected because it can turn rendering into quadratic work on large outlines.

### Identify folds by a structural path, not a source offset or anchor alone

Each projected node will receive an internal key formed from its ancestor path and its own `(level, title, same-named-sibling ordinal)` segment. The path distinguishes duplicate titles in different branches and duplicate sibling headings, survives source-offset shifts caused by body edits, and naturally ceases to match when the relevant hierarchy is renamed or removed. On a new document version, the folding state will be reconciled against live projected keys so obsolete keys are dropped rather than later applying to an unrelated section.

Using `offset` alone was rejected because any edit before a heading shifts it. Using `anchor` alone was rejected because current anchors are title-derived and duplicate headings can share one. Resetting all folds on every document edit was rejected because typing in a collapsed section's body would make the sidebar repeatedly expand.

### Store folding state inside each document tab

`DocumentTabState` will own session-only outline folding state: the collapsed key set plus the minimal version/snapshot metadata needed for reconciliation. New document tabs initialize with an empty collapsed set, which means fully expanded. Image tabs have no outline state. Tab switches therefore restore the corresponding document's folds without adding a global path map or persisted preference.

Application helpers will toggle a projected node key and notify GPUI. The state is excluded from undo/redo and document snapshots because it is interaction-only, like other per-tab presentation state.

A single application-level collapsed set was rejected because folds would leak between documents. Persisting keys was rejected because structural identities are scoped to the current in-memory heading hierarchy and the requirement is session-only.

### Give disclosure and navigation separate hit targets

Expandable rows will render a right/down chevron in a fixed-width disclosure slot; leaf rows retain an equally sized inert spacer so labels remain aligned. The disclosure control has its own pointer listener, updates folding state, and stops propagation. The label region owns the existing outline-navigation listener. Thus a disclosure click never changes the canonical cursor or preview scroll, while a label click never changes folding state.

The established embedded Lucide icon path can be extended with chevron assets so the affordance follows theme colors and does not introduce translatable text. Making the entire row toggle folding was rejected because it would conflict with the existing, frequently used click-to-jump contract. Requiring double-click was rejected because it is less discoverable and delays navigation.

### Preserve explicit folds when the active heading is hidden

The projection will map a hidden canonical active heading to its nearest visible collapsed ancestor for active styling. Cursor movement will not auto-expand the tree, so an explicit fold remains stable while editing or navigating outside the outline. When the subtree is expanded, the exact active heading resumes its normal highlight.

Auto-expanding ancestors on cursor movement was rejected because typing or source navigation would silently undo the user's organization. Rendering no active feedback was rejected because the existing outline contract tracks the current section.

## Risks / Trade-offs

- [A structural edit changes a heading's path and loses its fold] -> Treat the changed node as a new outline identity and leave it expanded; this is safer than hiding a possibly unrelated section.
- [Duplicate same-named siblings are inserted or reordered] -> Include the sibling ordinal only as a duplicate disambiguator and prune keys against the new projection; focused tests will cover duplicate titles and hierarchy edits.
- [Nested pointer handlers invoke both fold and navigation] -> Use distinct disclosure/label elements, stop propagation in the disclosure handler, and add GPUI interaction coverage for both hit targets.
- [Filtering breaks compact layout or scrolling] -> Preserve current row metrics and scroll container, add a fixed-width disclosure slot, and verify long partially folded outlines remain scrollable.
- [A large outline adds per-frame work] -> Keep projection linear and allocation-bounded over the already-requested heading vector; no parsing, preview derivation, or additional per-heading recursive scans occur.

## Migration Plan

No stored data, preferences, file formats, or public APIs change. Add the projection/state helpers and disclosure assets, route the two click targets, then verify default, nested, duplicate-heading, tab-isolation, active-hidden, and navigation behavior. Rollback consists of removing the per-tab presentation state and restoring the current single-listener rows; Markdown content and user preferences require no migration.
