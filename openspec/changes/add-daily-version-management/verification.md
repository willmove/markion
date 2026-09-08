# Verification

Verified on 2026-09-08.

## Automated checks

- `rustfmt --edition 2024 --check` passed for all touched root-crate Rust files.
- `cargo fmt -p markion-git-sync -- --check` passed.
- `git diff --check` passed. Git only reported the repository's existing LF-to-CRLF checkout warnings.
- `cargo test --workspace` passed, including all root-package tests and every workspace member. The `markion-git-sync` portion passed 41 unit tests, 3 fixture tests, and 14 integration tests.
- Focused document lifecycle tests passed:
  - `git_version_message_input_does_not_mutate_document`
  - `git_restore_is_one_dirty_undoable_edit_and_preserves_other_tabs`
  - `git_restore_rejects_a_stale_tab_version`
  - `cancelling_dirty_git_restore_preserves_the_buffer`
- `openspec validate add-daily-version-management --strict` passed.

## Covered behavior

- A local version commit uses the authored nonblank message, stages only the reviewed selected paths, preserves unselected changes, rejects external staging and stale content, and performs no network operation.
- Repository and active-file histories use bounded literal-path Git queries, follow renames, and expose commit metadata.
- Comparisons are bounded and report text, binary, and truncated states without enabling external diff or text conversion.
- Restoring a historical regular UTF-8 file creates one dirty, undoable editor mutation, leaves disk and Git metadata untouched, and rejects stale or cancelled replacements.

## Repository menu follow-up

- `git_actions_live_only_in_repository_menus` passed, proving that both native and in-window File menus contain no Git command and both Repository menus contain the complete existing command set.
- `every_menu_title_wires_click_and_hover_behavior` and `every_application_dropdown_uses_shortcut_aware_rows` passed for the added Repository menu.
- `every_message_returns_non_empty_text_for_every_language` passed with the Repository title included in all seven languages.
- Targeted `rustfmt --check` and `git diff --check` passed for this change.
- `cargo fmt --all -- --check` found one unrelated formatting difference at `src/source_mapped.rs:1140`; that concurrently modified file was not changed by this work.
- The follow-up `cargo test --workspace` run reached the root library and reported three unrelated failures in concurrently modified source-mapping code:
  - `source_mapped::tests::stable_ids_follow_source_lineage_not_equal_text`
  - `source_mapped::tests::stable_ids_handle_splits_nested_lists_islands_and_document_isolation`
  - `visual::tests::maps_common_inline_runs_to_exact_source_content`
- Each unrelated failure reproduces under its focused root-library filter. The menu and localization checks remain green, so this follow-up does not claim a fully green workspace run.
- `openspec validate add-daily-version-management --strict` passed after the menu artifacts were updated.
