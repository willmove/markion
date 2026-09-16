## Why

Enter in a Visual Edit list can update the source without showing the new row or moving the painted caret. Existing tests check the inserted prefix but do not assert the resulting row and caret geometry.

## What Changes

- Reproduce the supplied Chinese list with and without trailing blank lines.
- Fix the derived row/projection/layout ownership responsible for invisible continued items.
- Verify continuing and exiting lists, typing, undo and cached-vs-fresh block parity.

Non-goals: new list syntax or unrelated HTML changes.

## Capabilities

### New Capabilities

### Modified Capabilities

- `markdown-editing`: require visible continued list rows and matching caret geometry immediately after Enter.

## Impact

Visual list derivation and GPUI rendering/selection, plus parser and rendered interaction tests. Canonical source mutation and per-version cached blocks remain the only document model.
