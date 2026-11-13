## Why

Release preflight reproduces a Linux failure in the durable AI action roundtrip: creating a child folder changes its parent's link count, which incorrectly invalidates the unchanged workspace identity before the next reviewed operation. The same issue prevents safe restoration of the plan.

## What Changes

- Compare workspace/read-scope directory identity by filesystem volume and object id, retaining the recorded link count for serialization compatibility.
- Keep regular-file identity equality, hard-link rejection, path containment and replacement checks unchanged.
- Cover child-directory creation/removal, old journal identity compatibility, and the existing replacement/symlink defenses with focused tests and the full release gate.

Non-goals: broader AI permissions, new file operations, bypassing proposal review, or changing journal formats. No Markdown/cache/rendering behavior changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workspace`: distinguish stable directory object identity from mutable child-directory link counts during reviewed AI workspace operations.

## Impact

`crates/ai/src/workspace.rs`, root `src/ai_actions.rs`, and regression tests. No new dependencies, persistent schema changes, GPUI dependencies in members, or changes to per-version derived state.
