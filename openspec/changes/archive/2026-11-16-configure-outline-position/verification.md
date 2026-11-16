# Verification

Date: 2026-11-16. Host: macOS Apple Silicon.

## Focused checks

- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- `cargo test --lib outline_position`: 2 passed. Covers both persisted sides and field-local fallback for missing, unknown, and non-string values without discarding other preferences.
- `cargo test --bin markion outline`: 19 passed. Includes geometry in all four document modes and light/dark themes, Files remaining on the left, coexistence with the AI panel, actual preference pointer activation and persistence, hidden-sidebar geometry, document/cache identity, nested folds, right-side navigation, scrolling, divider dragging/clamping, width reset, and preference reset.
- `cargo clippy --workspace`: passed with warnings.
- `cargo build`: passed.
- `cargo run -p wechat-workspace --bin verify-bundle -- assets/marknice-workspace`: passed; verified 23 files (3,140,387 bytes).
- `openspec validate configure-outline-position --strict`: passed.

## Repository-wide validation limits

The initial `cargo test` run using the host's default temporary directory finished its library suite with 737 passed, 6 failed, and 1 ignored. All six failures were in unchanged `ai_actions` tests: `ReadGrant::new` rejected the `/var` symlink ancestor of the macOS temporary directory with `AiError::Scope`. Re-running with `TMPDIR=/private/tmp` resolved all six failures: the root library suite passed 743 tests with 1 ignored.

`TMPDIR=/private/tmp cargo test --workspace` then reached the root application suite with 796 passed, 2 failed, and 2 ignored. The failures are in unchanged AI input tests, and both reproduce when the AI panel test group is run separately:

- `app::ai_panel::tests::focused_composer_blocks_editor_shortcuts_and_preserves_profile_drafts`: `ctrl-a` followed by Backspace produces `hell` instead of the expected empty input on this host.
- `app::ai_panel::tests::printable_keys_work_in_composer_and_settings_without_editing_document`: the final `1` key is absent from the input.

Checking the remainder with those two names explicitly filtered reached an additional PDF font failure. A final `TMPDIR=/private/tmp cargo test --workspace --no-fail-fast` run exercised all targets without filtering. It reported the same two AI input failures plus `markion-pdf` tests `dump` and `representative_document_renders_pdf_bytes`, both failing with `Fonts("krilla rejects face 0")`. No AI input, AI action, or PDF implementation files were changed, and none of these failures is classified as passing coverage.

Final workspace totals across all test and documentation-test targets: **2,138 passed, 4 failed, 5 ignored**. The failures span the root application test target and the two PDF integration-test targets described above.

`openspec validate --all --strict --no-interactive` reported 15 passed and 91 failed items before archive. This change passed; existing changes and stable specs report long requirement descriptions and placeholder Purpose sections under the installed OpenSpec 1.14.1 CLI. No unrelated stable-spec cleanup is included in this feature.

The UI evidence is from GPUI layout and pointer-event tests. Native interactive checks on Windows and Linux were not performed.

## Archive

`openspec archive configure-outline-position --yes` completed successfully on 2026-11-16. All six tracked tasks are complete, and the two added requirements were synced into `openspec/specs/theme-preferences/spec.md` by the archive command.
