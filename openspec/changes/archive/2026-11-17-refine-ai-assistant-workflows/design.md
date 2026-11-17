## Context

See proposal.md for motivation. `AiUi` already separates draft settings from persisted profiles and uses synthetic tests, scoped requests and checked writing application. Its panel is one scrolling column, message height is fixed, and its custom input shapes only hard lines. Existing AI changes remain unarchived.

## Goals / Non-Goals

Goals: fix setup sequencing, prioritize conversation, make task scope and review understandable, and preserve independent input/undo and filesystem safeguards.

Non-goals: replace the Markdown engine, add a general component framework, change permission policy or persistence formats, or certify a live cloud endpoint.

## Decisions

1. Split discovery validation from generation validation. Discovery validates address, protocol, limits needed by transport and credentials without requiring a model. Test and save continue validating the complete draft.
2. Save-and-enable is an explicit commit intent consumed only after successful credential/profile persistence. Preserve a capability result only when the committed request identity matches the tested identity. Enabling alone preserves the same profile's verification, while disabling still invalidates work and grants.
3. Render header, flexible body and bounded composer as sibling regions. Complex review/history/tool controls live in the scrollable body, keeping the composer visible at small sizes. Use existing embedded Lucide icons and focusable helpers; unavailable controls cannot dispatch.
4. Retain one chat surface, with a workspace mode selector and contextual writing disclosure. Show language/tone only for their selected action; validate before generation with dedicated localized messages. Group settings into service, writing, history/privacy and advanced limits with a persistent save footer.
5. Soft wrapping uses GPUI text measurement and source byte offsets. Visual rows are derived from input text, available width and font metrics and cached locally. Mouse selection, IME bounds and Up/Down map through visual rows. Input changes do not touch document versions; accepted writing changes still use the checked mutation boundary.

```text
AI input text + width/font --> cached visual rows --> paint / hit-test / IME
draft fields --> discovery/test --> identity-bound capability
explicit save intent --> credential store --> profile commit --> enable
document snapshot --> writing request --> review --> checked mutation --> document version/cache
```

6. Context chips identify file and range and distinguish pending attachments from sources used in previous turns. History reflects actual persistence mode. History deletion requires explicit confirmation. Writing review summarizes changed lines with additions/removals; full text remains available and application is unchanged.

## Risks / Trade-offs

- Wrapped rows must preserve UTF-8 boundaries and CJK/emoji IME geometry: verify narrow inputs, caret movement, selection and composition in GPUI tests.
- Settings edits during asynchronous save/test can make results obsolete: bind results and save intent to draft/configuration identity, retaining draft values on failure.
- Compact layouts can hide rare actions: disclosures are keyboard reachable and reviews stay in the primary body scroll.
- Native desktop control is unavailable in this session: use GPUI rendered-layout integration tests and document the remaining manual visual acceptance explicitly.

## Migration Plan

Existing profiles load unchanged. Only UI-local disclosure/intent/wrapping state is added. Update guides and record automated acceptance. Do not archive ahead of the two prerequisite AI changes.
