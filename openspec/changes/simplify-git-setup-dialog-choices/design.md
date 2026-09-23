## Context

`onboarding_view` ([`src/app/git_panel.rs:2705`](src/app/git_panel.rs)) currently renders the route decision four times: a `✓ <route>` heading, an "other setup options" disclosure (`alternatives_open` + `toggle_git_onboarding_alternatives`), the three route buttons behind that disclosure, and the selected route's name as the bottom primary button (with "Retry" after errors and "Advanced Repository Setup" when advanced review is required). User feedback asks for the standard dialog shape: three visible choices, recommended preselected, plain confirm/cancel.

The route recommendation (`onboarding_route` in `src/app/git_sync.rs`), per-route field defaults (`set_git_onboarding_mode`), and the inline address confirmations from `fix-git-onboarding-remote-usability` are correct and stay untouched.

## Goals / Non-Goals

**Goals:**

- One directly visible row of three route buttons with the contextual recommendation preselected and the selection marked.
- Generic bottom actions: confirm + cancel.
- Delete the now-dead disclosure state, handler, and strings.

**Non-Goals:**

- No change to route recommendation logic, per-route fields, advanced gating, adoption, or the address validation/confirmation flows.
- No redesign of dialog typography beyond the removed heading.

## Decisions

### 1. Layout: heading and disclosure collapse into the button row

```
Before                                After
+------------------------------+     +------------------------------+
| 开启备份与同步                |     | 开启备份与同步                |
| (hints)                      |     | (hints)                      |
| ✓ 从已有同步地址获取笔记      |     | [将此文件夹用于备份与同步]    |
| [隐藏其他设置方式]            | --> | [✓ 从已有同步地址获取笔记]    |
|   (展开时) 三个方式按钮       |     | [为此文件夹开启备份与同步]    |
| (字段...)                    |     | (字段...)                    |
| [从已有同步地址获取笔记][取消]|     | [确认] [取消]                |
+------------------------------+     +------------------------------+
```

The three existing `button` children move out of `.when(setup.alternatives_open, …)` to render unconditionally; the `selected_label` checkmark stays as the selection mark; the heading block and the disclosure button are deleted.

### 2. Primary button becomes a single confirm label

`primary` today is a three-way expression (mode name / Retry / AdvancedRepositorySetup). All three collapse into the new `GitMsg::Confirm`: after an error the message is already displayed inline above the actions, and the advanced-required case explains itself in the section header, so a mode- or state-specific verb adds no information. The button id `git-setup-submit` and its handler (`submit_git_onboarding`) are unchanged.

### 3. State and string cleanup

- `GitOnboarding.alternatives_open` and `toggle_git_onboarding_alternatives` are removed (no remaining callers; `set_git_onboarding_mode` never read the flag).
- `GitMsg::OtherSetupOptions`/`HideSetupOptions` variants and translations are deleted; a new `Confirm` variant is added (7 languages); the Simplified Chinese `Cancel` wording aligns to "取消".
- `src/app/tests.rs`'s `GitOnboarding` construction drops the removed field.

### 4. Baseline delta reconciliation

`add-git-workspace-sync` (unarchived) states alternative routes stay "behind a secondary choice". This change edits that unarchived delta's wording to the directly-visible presentation so the archived main spec carries one coherent rule, and notes the supersession in its proposal impact. Editing an unarchived change's planning artifact is within normal OpenSpec practice; no main spec is touched directly.

## Risks / Trade-offs

- [Three equal choices weaken the "one recommended route" guidance] → The contextual recommendation stays preselected and check-marked; the dialog hints above still explain the recommended path.
- [Dialog grows slightly taller with the row always shown] → The removed heading and disclosure roughly offset the added row.
- [Confirm loses the retry semantics after errors] → The error text remains visible above the actions; pressing confirm re-runs the same validation, which is the retry.

## Migration Plan

None. Pure UI presentation; no persisted state or policy changes. Rollback is a plain revert.

## Open Questions

- Final per-language wording of the confirm label — resolved during implementation following `src/i18n/git.rs` conventions.
