# OpenSpec archival review — 2026-11-17

Scope: review current implementation, completed task evidence and delta/main-spec compatibility. Task checkmarks alone are not an archive decision. All selected changes use the `spec-driven` schema. This review changes documentation and archival state, not application behavior.

## Archived and synced

The following changes have completed artifacts/tasks, recorded implementation evidence, strict change validation, and verified main-spec merges. Existing scenarios were retained; only then were the directories moved.

| Change | Archived tasks |
| --- | --- |
| `add-configurable-ai-agent` | [Tasks](../../openspec/changes/archive/2026-11-17-add-configurable-ai-agent/tasks.md) |
| `refine-ai-assistant-workflows` | [Tasks](../../openspec/changes/archive/2026-11-17-refine-ai-assistant-workflows/tasks.md) |
| `refine-ai-settings-and-preference-tabs` | [Tasks](../../openspec/changes/archive/2026-11-17-refine-ai-settings-and-preference-tabs/tasks.md) |
| `bound-preview-raster-memory` | [Tasks](../../openspec/changes/archive/2026-11-17-bound-preview-raster-memory/tasks.md) |
| `bundle-appimage-software-vulkan` | [Tasks](../../openspec/changes/archive/2026-11-17-bundle-appimage-software-vulkan/tasks.md) |
| `fix-linux-wayland-window-controls` | [Tasks](../../openspec/changes/archive/2026-11-17-fix-linux-wayland-window-controls/tasks.md) |
| `round-csd-window-corners` | [Tasks](../../openspec/changes/archive/2026-11-17-round-csd-window-corners/tasks.md) |
| `fix-visual-table-cell-navigation` | [Tasks](../../openspec/changes/archive/2026-11-17-fix-visual-table-cell-navigation/tasks.md) |
| `fix-visual-undo-viewport-and-selection-copy` | [Tasks](../../openspec/changes/archive/2026-11-17-fix-visual-undo-viewport-and-selection-copy/tasks.md) |
| `show-filename-in-title-bar` | [Tasks](../../openspec/changes/archive/2026-11-17-show-filename-in-title-bar/tasks.md) |
| `add-updater-manifest-fallback` | [Tasks](../../openspec/changes/archive/2026-11-17-add-updater-manifest-fallback/tasks.md) |
| `support-linux-in-app-updates` | [Tasks](../../openspec/changes/archive/2026-11-17-support-linux-in-app-updates/tasks.md) |
| `collapse-visual-separator-gaps` | [Tasks](../../openspec/changes/archive/2026-11-17-collapse-visual-separator-gaps/tasks.md) |
| `compact-slash-command-panel` | [Tasks](../../openspec/changes/archive/2026-11-17-compact-slash-command-panel/tasks.md) |
| `align-emoji-completion-palette` | [Tasks](../../openspec/changes/archive/2026-11-17-align-emoji-completion-palette/tasks.md) |
| `add-slash-command-preference` | [Tasks](../../openspec/changes/archive/2026-11-17-add-slash-command-preference/tasks.md) |
| `fix-block-handle-selection-and-menu` | [Tasks](../../openspec/changes/archive/2026-11-17-fix-block-handle-selection-and-menu/tasks.md) |
| `unify-compact-block-context-menu` | [Tasks](../../openspec/changes/archive/2026-11-17-unify-compact-block-context-menu/tasks.md) |
| `enrich-default-welcome-samples` | [Tasks](../../openspec/changes/archive/2026-11-17-enrich-default-welcome-samples/tasks.md) |
| `bundle-example-document-library` | [Tasks](../../openspec/changes/archive/2026-11-17-bundle-example-document-library/tasks.md) |
| `clarify-keyboard-shortcuts` | [Tasks](../../openspec/changes/archive/2026-11-17-clarify-keyboard-shortcuts/tasks.md) |
| `display-menu-shortcut-labels` | [Tasks](../../openspec/changes/archive/2026-11-17-display-menu-shortcut-labels/tasks.md) |
| `improve-find-replace-and-read-search` | [Tasks](../../openspec/changes/archive/2026-11-17-improve-find-replace-and-read-search/tasks.md) |
| `limit-initial-file-tree-expansion` | [Tasks](../../openspec/changes/archive/2026-11-17-limit-initial-file-tree-expansion/tasks.md) |
| `stabilize-visual-edit-typing-viewport` | [Tasks](../../openspec/changes/archive/2026-11-17-stabilize-visual-edit-typing-viewport/tasks.md) |

## Completed checklists retained for reconciliation

These changes remain active. Their historical task completion is preserved, but their complete delta cannot safely be treated as a replacement for the current contract. No runtime work or manual verification is claimed by this documentation review.

| Change | Reason |
| --- | --- |
| [add-recent-workspace-switcher](../../openspec/changes/add-recent-workspace-switcher/tasks.md) | Session restoration and dirty-tab policy overlap later tab changes; reconcile the complete combined contract before sync. |
| [collapse-visual-math-and-diagram-source](../../openspec/changes/collapse-visual-math-and-diagram-source/tasks.md) | Hover-toggle presentation overlaps newer direct-editor and pending/error requirements; verify the final combined presentation contract. |
| [fix-visual-blank-line-height-jump](../../openspec/changes/fix-visual-blank-line-height-jump/tasks.md) | The historical body-flow description overlaps the later separator-gap rules; reconcile typography and whitespace scenarios together. |
| [fix-windows-microsoft-pinyin-crash](../../openspec/changes/fix-windows-microsoft-pinyin-crash/tasks.md) | Preserve the later composition-scoped undo contract when merging the historical Windows/XIM description. |
| [improve-macos-distribution](../../openspec/changes/improve-macos-distribution/tasks.md) | Its signing/Gatekeeper claims and broad release-note snapshot need reconciliation with conditional notarization and the concise bilingual policy. |
| [move-visual-selection-formatting-to-context-menu](../../openspec/changes/move-visual-selection-formatting-to-context-menu/tasks.md) | The original block-owning formatting description differs from the newer combined selection/clicked-block menu; reconcile without narrowing exact selection scope. |
| [protect-unsaved-edits](../../openspec/changes/protect-unsaved-edits/tasks.md) | Merge with per-workspace restoration and later tab-open behavior without reintroducing obsolete dirty-open prompts. |
| [refine-default-shortcuts-and-md-ref-scrollbar](../../openspec/changes/refine-default-shortcuts-and-md-ref-scrollbar/tasks.md) | Its direct Edit shortcut scenario differs from the current Source/Split toggle; preserve current registry and saved overrides. |
| [update-default-shortcuts-and-markdown-reference](../../openspec/changes/update-default-shortcuts-and-markdown-reference/tasks.md) | Its historical Read and Visual Edit defaults have been superseded; do not restore them through archival. |
| [visual-source-toggle-images-tables-and-fence-language](../../openspec/changes/visual-source-toggle-images-tables-and-fence-language/tasks.md) | The image source-toggle behavior was superseded by Image Properties; do not add an obsolete image-toggle requirement. |

## Other active work

Changes with unchecked implementation or manual acceptance tasks remain active, including DOCX corpus/platform coverage, MarkNice export/browser/viewer coverage, native CSD resize interaction, image-resource interoperability, notarization provisioning, Git/scope follow-ups and direct WeChat publishing. No unchecked task was marked complete merely to archive it. The concurrently prepared release-acceptance suite is separate documentation work and is not certified by this review.

## Documentation repair and validation

- Replaced 14 legacy main-spec Purpose placeholders with descriptions of their existing capability; no requirement was introduced through this editorial repair.
- Rebased older MODIFIED delta blocks to retain current main-spec scenarios. These retained baseline scenarios do not establish implementation or acceptance of the pending change.
- Preserved the current combined bilingual README contract, updater host fallback and independent same-kind visual-block identities while syncing the applicable changes.
- Updated documentation links to archived paths. Strict validation is rerun after all merges and retained-delta repairs.
- Full workspace regression: 2,216 passed, 0 failed, 5 existing ignored, across 51 suite results. This is the Windows source-tree regression, not a native multi-platform acceptance matrix.
