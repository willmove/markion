## Why

Windows retesting of #53 found that moving a list parent back up with the keyboard can split the preceding sibling's subtree. The menu already resolves a destination at the selected item's depth.

## What Changes

- Use the existing adjacent block resolver for keyboard movement of a single list subtree.
- Verify that moving a parent down and back up keeps both subtrees intact and keeps following prose outside the list.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing` covers keyboard movement of a single list subtree.

## Impact

The keyboard command reuses versioned source units and applies one checked edit per move. The change covers the keyboard target for a single list subtree.
