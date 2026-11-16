# Verification

Date: 2026-11-16. Host: macOS Apple Silicon.

## Corrected behavior

- Right placement renders real file-tree rows on the left and outline headings on the right simultaneously, with the document and its tabs between them. Each pane has its own content-specific header; neither contains a Files/Outline tab switch.
- GPUI geometry checks cover all four document modes and light/dark themes. Clicking an actual temporary Markdown file keeps both panes visible in their respective positions. The right outline remains separate from the docked AI panel.
- File Tree and Outline actions and preference controls toggle their own pane. The Sidebar master toggle hides or shows both. Returning Left restores the original shared tabs, including when only the right outline was visible.
- Actual divider drags clamp each pane independently; resizing and double-click reset leave the other pane's width intact. Captured session layout round-trips both widths through temporary-file storage. App session writes are disabled in GPUI tests, so this storage check explicitly writes the captured layout through the normal session adapter.
- Preference migration and round-trip tests cover independent visibility and missing/invalid legacy values. Existing outline navigation, folding, scrolling, document/cache preservation, and preference-reset tests pass.

## Checks

- `cargo fmt --all -- --check` and `git diff --check`: passed.
- Root package tests with `TMPDIR=/private/tmp` and the two previously known AI input failures explicitly filtered: **1,546 passed, 0 failed, 3 ignored, 2 filtered**. This includes 744 library tests, 797 application tests, and 5 additional binary/integration tests.
- `cargo clippy -p markion`: passed with warnings.
- `cargo build`: passed.
- `openspec validate separate-right-outline-from-file-tree --strict`: passed.

## Limits

The exact filtered tests are `app::ai_panel::tests::focused_composer_blocks_editor_shortcuts_and_preserves_profile_drafts` and `app::ai_panel::tests::printable_keys_work_in_composer_and_settings_without_editing_document`. They failed in the preceding change's unfiltered macOS test runs and remain recorded as failures, not passing coverage. The physical temporary directory avoids the previously diagnosed `/var` symlink rejection in AI scope tests.

The previous full-workspace PDF font failures and repository-wide strict OpenSpec warnings are documented in `../2026-11-16-configure-outline-position/verification.md`. This correction does not modify those subsystems or rerun their unrelated failed checks. Windows/Linux native interactive checks were not performed.
