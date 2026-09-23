## ADDED Requirements

### Requirement: Outline heading anchors match in-document hash links
Each outline heading SHALL expose a stable `anchor` string that in-document `#fragment` links resolve against. The id SHALL be the authored heading attribute `{#id}` when pulldown-cmark reports one; otherwise a Unicode-preserving slug of the visible title. Empty titles SHALL use `section`. Duplicate ids in document order SHALL uniquify with a numeric suffix (`hello`, `hello-1`). Changing folding, hovering, or clicking the outline SHALL still not mutate Markdown.

#### Scenario: Outline anchors follow authored ids
- **WHEN** a heading is authored as `## Title {#custom}`
- **THEN** that outline item’s `anchor` is `custom`

#### Scenario: Duplicate outline titles uniquify
- **WHEN** two headings share the visible title `Hello` and neither has an authored id
- **THEN** their outline anchors are `hello` and `hello-1` in document order
