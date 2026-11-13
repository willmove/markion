## Context

v0.5.2 is the highest stable public tag. Subsequent commits complete Visual Edit P0–P3 work and Windows input follow-ups, and add Linux client-decorated window controls. Historical reports, READMEs, and the coverage matrix have drifted from these implementations. Several active changes still contain genuine manual-verification gaps.

## Goals / Non-Goals

**Goals:** equivalent English/Chinese current documentation, a dated status index, source-linked verification boundaries, synchronized patch metadata, and the existing release process followed without claiming unfinished checks.

**Non-Goals:** application changes, universal performance promises, changing persisted formats, or marking unrelated checklists complete.

## Decisions

- Keep historical benchmark numbers and raw evidence intact; prepend supersession notes and link the final implementation and Windows verification reports. Replacing the historical reports would obscure what each measurement actually covered.
- Use `docs/project-status.md` as the dated implementation/verification index. README summaries stay concise and link there rather than duplicating every historical task.
- Generate the standalone Simplified Chinese edition from the root README's Chinese section while retaining its direct-language navigation. Other translations receive equivalent current-status corrections and links.
- Describe packaged capabilities from implementation and stable specs while explicitly naming active proposals and unperformed manual checks. Do not equate task counts with verified product coverage.
- Add a documentation delta only; preserve every application caching, canonical-source, and workspace boundary invariant.

## Risks / Trade-offs

- [Historical Windows evidence could be mistaken for this release's Linux results] → label dates, platform, build scope, and measured intervals; record current release checks separately.
- [Publishing requires credentials and complete verification] → retain explicit pending tasks until authenticated publication and final asset/mirror checks succeed; preserve public tags on any failure.
- [Localized documentation can drift] → synchronize the Chinese text mechanically and check corresponding status claims in every README.
