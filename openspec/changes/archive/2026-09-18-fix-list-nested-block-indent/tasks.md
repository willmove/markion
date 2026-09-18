# Tasks

## 1. Model: list depth on nested-capable preview blocks

- [x] 1.1 Add `list_depth: usize` to `PreviewBlock::CodeBlock`, `MathBlock`, `Table`, and `Html` in `src/model.rs`, plus a `PreviewBlock::list_depth()` accessor returning the field for those variants and 0 for every other variant. Verify with `cargo check`.
- [x] 1.2 Capture `list_stack.len()` at emit time in `src/lib.rs` for the fenced/indented code and fenced-math arm, the table arm, and standalone/inline-HTML block pushes (`push_html_block` gains a depth parameter, used on both its merge and append paths). Fix the remaining constructions (`MathBlock` display-dollar arm, `absorb_table_column_width_comments`-adjacent pushes, tests) to compile. Verify with `cargo check`.

## 2. Visual Edit projection carries the depth

- [x] 2.1 Add `list_depth: usize` to `VisualBlock`; set it from the preview block in `visual_block_from_preview` and to 0 in `gap_block`, `callout_title_block`, `front_matter_visual_block`, and `source_island`. Verify with `cargo check`.

## 3. Renderers indent nested block rows

- [x] 3.1 In Reading mode (`preview_block_view` in `src/app/preview.rs`), apply a left inset of `(depth − 1) × 18 + 22` px to blocks with `list_depth > 0`. Verify with `cargo check`.
- [x] 3.2 In the Visual Edit row view (the `match &block.kind` row builder in `src/app/preview.rs`), apply the same inset around rows whose `VisualBlock::list_depth > 0`, inside block chrome so caret/hit-testing stay row-relative. Verify with `cargo check`.

## 4. Regression tests

- [x] 4.1 Parse-level tests in `src/lib.rs`: the reported fixture (`1.  Item 1` / `2. Item 2` / nested `1.     Sub item 1` / `2. sub item 2`) yields outer items, the empty nested item, a `CodeBlock` with `code == "Sub item 1"` and `list_depth == 2`, and the sibling nested item, in source order; a fence nested in a single-level list records depth 1; a top-level fence records depth 0; a table nested in a list records depth 1. Verify with `cargo test list_nested` (root crate).
- [x] 4.2 Visual-projection test in `src/visual.rs`: the reported fixture's code row carries `list_depth == 2` while item rows stay untouched, and existing nested-fence fixtures (`nested_code_partition_boundary_keeps_exact_caret_positions`, `minimax_fixture_list_nested_code_renders_without_raw_boxes`) still pass. Verify with `cargo test visual` (root crate).

## 5. Validation and archive

- [x] 5.1 Run `cargo test --workspace` and confirm every crate's suite passes. Fix any exhaustive-match fallout the compiler surfaces.
- [x] 5.2 Run `openspec validate fix-list-nested-block-indent`, then archive the change (`openspec archive fix-list-nested-block-indent`) so the delta specs sync into `openspec/specs/markdown-editing/`.
