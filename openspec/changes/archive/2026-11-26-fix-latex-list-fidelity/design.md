## Context

`render_latex_body` currently consumes cached `PreviewBlock::ListItem` values but groups consecutive items only by `ordered`, ignoring their `level` and `index`. The same list logic is duplicated for top-level blocks and blockquote children. `push_latex_list_item` already preserves task state and rich inline text. See `proposal.md` for motivation and `specs/export/spec.md` for the required behavior.

Data flow remains one-way: cached preview blocks for the current document version → a contiguous slice of list items → LaTeX environments and item labels. No document mutation, new derived state, cache writes, or version changes are involved.

## Goals / Non-Goals

**Goals:**

- Translate the flat, level-annotated preview item stream into correctly nested `itemize`/`enumerate` environments.
- Preserve every resolved ordered index and existing task/rich-text rendering.
- Share one implementation between top-level lists and lists inside blockquotes.

**Non-Goals:**

- Do not change the preview model, parsing, public export APIs, or other output formats.
- Do not introduce LaTeX packages, dependencies, counter configuration in the preamble, or compilation validation.

## Decisions

1. **Render contiguous list-item slices with recursive descent by resolved level.** A small helper walks one environment at a time: equal-level/equal-kind items are siblings, deeper items are rendered after the current parent item's content, and a shallower level closes the current environment. A kind change at the same level closes one environment and opens another at that level. This places child environments before the next parent sibling and removes the duplicated top-level/blockquote logic. An explicitly open/closed environment stack was considered but rejected because parent/child kind transitions are easier to mismanage without adding value.

2. **Use the resolved `index` as an explicit optional label on ordered `\item` commands.** For example, index 6 becomes `\item[6.]`. Unordered items retain the current unlabeled form, and task symbols follow any ordered label as before. Explicit labels preserve the exact resolved numbering at arbitrary nesting depth without relying on LaTeX's limited `enumi`–`enumiv` counter hierarchy. Using `\setcounter` was considered and rejected because it only restores the first index, needs depth-specific counters, and complicates mixed or deeply nested lists.

3. **Keep `push_latex_list_item` as the single item-content renderer.** Extend it only with the optional resolved ordered label; retain its checkbox and `render_latex_rich_text` behavior. The list-structure helper consumes existing `PreviewBlock` values and introduces no parallel document tree.

## Risks / Trade-offs

- [A child environment emitted in the wrong position can change parent/sibling semantics] → Assert the exact environment ordering for nested same-kind and mixed-kind lists, including a later top-level sibling.
- [Explicit labels may differ cosmetically from LaTeX's automatic number formatting] → Accept the visible decimal labels because they preserve resolved values directly and avoid counter-depth limits; ordered lists remain `enumerate` environments.
- [Recursive rendering depends on parser-provided levels] → Cover level decreases and same-depth kind changes; recurse only over the already parsed contiguous item slice and do not invent intermediate items or levels.

## Migration Plan

No data or API migration is required. The change is confined to built-in LaTeX serialization and can be rolled back as one commit. Existing `.tex` consumers receive the same content with corrected structure and numbering.
