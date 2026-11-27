## Why

Visual Edit always shows a fold control above code blocks. Make the control opt-in so the default view opens directly to the code.

## What Changes

- Add a localized switch in Preferences under Appearance → Code blocks. The switch defaults to off and takes effect immediately.
- When enabled, the switch makes manual folding available while each code block starts expanded. Turning it off expands previously folded blocks across tabs.
- Non-goals include automatic folding and changes to math or diagram source controls.

## Capabilities

### New Capabilities

### Modified Capabilities

- `code-and-math` makes session-only visual code folding opt-in.
- `theme-preferences` defines the switch location, default, persistence, and reset behavior.

## Impact

Preferences storage, localization, and Visual Edit code rows change. A preference change invalidates the affected row measurements. The per-version Markdown and syntax-highlighting caches remain reusable.
