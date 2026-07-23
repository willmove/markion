# Tasks – Simplify Visual Image Presentation

## 1. Remove the `VisualBlockEditor::Image` editor variant

- [x] 1.1 Remove the `Image` variant from `VisualBlockEditor` in `src/model.rs` and update `fields()` accordingly
- [x] 1.2 Remove `image_field_ranges` and the `PreviewBlock::Image` arm from `visual_block_editor` in `src/visual.rs`
- [x] 1.3 Remove `VisualBlockEditor::Image` from the `visual_editor_tab_target` / `visual_editor_edge_target` matches in `src/lib.rs`
- [x] 1.4 Remove `ImageAlt | ImageDestination | ImageTitle` from the `insert_newline` field match in `src/app/editing.rs`

## 2. Simplify the Visual Edit image presentation

- [x] 2.1 Remove `visual_image_editor` and `visual_image_field_row` from `src/app/preview.rs`
- [x] 2.2 Update the `VisualBlockKind::Image` rendering branch to show image + read-only caption (Title, or Alt Text when no Title), falling back to the source-backed island when no editor exists

## 3. Update tests

- [x] 3.1 Update `src/visual.rs` tests that assert on `VisualBlockEditor::Image`
- [x] 3.2 Update `src/lib.rs` tests that destructure or assert on `VisualBlockEditor::Image`
- [x] 3.3 Update or remove the `visual_direct_image_fields_traverse_sanitize_and_stay_tab_local` test in `src/app/tests.rs`

## 4. Verify

- [x] 4.1 Run `cargo test --workspace` and resolve all failures
- [x] 4.2 Run `openspec validate simplify-visual-image-presentation`
