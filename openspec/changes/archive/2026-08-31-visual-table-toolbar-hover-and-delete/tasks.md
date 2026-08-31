## 1. Presentation-only visibility

- [x] 1.1 Add tab-local `hovered_visual_table_block` (or equivalent) on the editor tab, initialize and clear it on the same tab/mode reset paths as `hovered_visual_source_block`, and update it from `visual_table_view` `on_hover` without mutating document text, dirty state, undo, or derived Arc caches
- [x] 1.2 Gate the Visual Edit table-editing header so it is omitted while idle and shown only when that table is hovered or `visual_table_toolbar_target` is Some for that block; keep preview/read tables toolbar-free

## 2. Whole-table delete control

- [x] 2.1 Add a localized delete-table toolbar label through `src/i18n.rs` for every supported language; keep the existing compact `+Row`/`-Row`/… labels unchanged
- [x] 2.2 Append a delete-table control after `-Col` that revalidates `BlockTarget` and calls `delete_visual_block`, enabled only when `block_can_reorder_at` would succeed, with debug selectors `visual-table-delete-table` / `visual-table-delete-table-disabled`
- [x] 2.3 Preserve compact shared button metrics, row/column targeting, disabled-boundary semantics, and the existing `P1Msg::BlockDeleted` / block-edit error reporting path

## 3. Tests and docs

- [x] 3.1 Add or adjust rendered GPUI tests for idle-hidden header, hover-shown header, caret-shown header after pointer leave, and no version/dirty/history change from show/hide
- [x] 3.2 Add rendered tests for whole-table delete (one undo restores source and selection), disabled nested/unsupported delete, and isolation so deleting or hovering one table does not mutate another
- [x] 3.3 Update existing Visual Edit toolbar tests that assume an always-visible six-button header so they focus or hover first and still cover all six row/column actions
- [x] 3.4 Update `docs/visual-editing-quality.md` table evidence for interaction-gated chrome and whole-table delete

## 4. Verification

- [x] 4.1 Run focused table/toolbar tests, `cargo fmt --all --check`, `cargo test`, and `cargo test --workspace`; resolve failures without adding gpui deps to workspace members
- [x] 4.2 Run `openspec validate visual-table-toolbar-hover-and-delete` and confirm every implementation task is checked off
