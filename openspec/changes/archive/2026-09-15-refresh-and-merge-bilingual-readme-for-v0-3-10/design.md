## Context

See `proposal.md` for motivation. The repository currently has separate English and Simplified Chinese README files, while the requested root landing page must contain both editions. The release must also follow `docs/release-process.md`, preserve the public-tag immutability rule, and pass native packaging plus GitHub/OSS distribution checks. GitHub CLI was absent during initial preflight.

## Goals / Non-Goals

**Goals:**

- Produce evidence-backed, equivalent English and Simplified Chinese documentation with Chinese in the lower half of the root README.
- Preserve `README.zh-CN.md` as a compatible direct-language entry point without allowing its substantive content to drift.
- Keep documentation integration separate from the dedicated version-only release commit.
- Publish and verify v0.3.10 through the canonical release workflow.

**Non-Goals:**

- Change editor behavior, formats, persisted data, dependencies, or packaging targets.
- Redesign the release workflow or rotate updater keys.

## Decisions

1. **Derive claims from repository evidence.** README updates will be based on stable specs, `v0.3.9..HEAD`, completed change artifacts, relevant implementation/configuration, and package definitions. Commit subjects alone are insufficient. The alternative—editing from memory—would make capability and limitation claims hard to verify.
2. **Treat the Chinese body as synchronized content in two entry points.** The same ordered Chinese sections will appear in the lower half of `README.md` and in `README.zh-CN.md`; only top-level navigation may differ. Stable HTML anchors in the root README will switch between language halves. Removing the standalone file was rejected because existing external links may depend on it.
3. **Separate integration from release metadata.** Documentation and the archived spec delta will be committed and pushed first. Version fields will then be updated in a dedicated `Release Markion v0.3.10` commit, matching the release runbook and keeping the release diff auditable.
4. **Stop on failed publication gates.** A tag is pushed only after local and manual verification. Once public, it is never deleted or moved; workflow failures are diagnosed and fixed forward, with the documented mirror-repair path used only when applicable.

## Risks / Trade-offs

- [Duplicated Chinese content can drift] → Compare normalized Chinese section bodies before committing and keep parity checks in the task checklist.
- [README claims can describe incomplete in-flight changes] → Include only behavior verified in stable specs and implementation; distinguish completed functionality from known limitations.
- [Missing GitHub CLI or authentication can block release] → Resolve and verify the CLI/authentication during preflight before any tag is created.
- [Native or mirror jobs can fail after the public tag exists] → Preserve the tag, inspect failed logs, fix forward, and do not report completion until every required job and asset is verified.

## Migration Plan

No application or data migration is required. Existing links to `README.zh-CN.md` continue to work. Documentation changes land before the release commit; rollback before tagging is a normal revert, while any post-tag correction follows the repository's fix-forward policy without rewriting the tag.
