## Why

Markion currently starts with one in-memory Markdown tour, but installed users have no reusable collection of everyday writing templates or focused format exercises. Shipping an offline example library makes the supported editing workflows discoverable and gives users useful documents to copy and adapt immediately.

## What Changes

- Bundle a Chinese-first library under `assets/examples`, with a bilingual index, a practical getting-started guide, focused Markdown/HTML/table/image/code/math/diagram exercises, and reusable daily-work templates.
- Include realistic starter content, replacement placeholders, editing exercises, and a local image so the collection works offline and remains portable when copied as a directory.
- Add a localized Help → Examples and Templates action to both menu surfaces that reveals the bundled collection in the system file manager; point to it from the welcome document.
- Verify that every native package contains the complete, unchanged example collection using the existing package inspection pipeline.

Non-goals: a template marketplace, a new document-creation wizard, automatic copying into user workspaces, modifying existing user files, changing supported Markdown syntax, or publishing a release.

## Capabilities

### New Capabilities

- `bundled-examples`: Offline example documents, reusable writing templates, discovery, safe reuse guidance, and distribution completeness.

### Modified Capabilities

None. Existing workspace and release flows remain in place; the collection adds a new resource payload and Help action.

## Impact

- Assets: new Markdown files and a local sample image in `assets/examples`; existing `packager.toml` and portable packaging already copy all of `assets`.
- UI: Help menu definitions, action handling, localized labels/status feedback, and a short welcome pointer. Reuse `bundled_resource_path` and system file-manager integration.
- Validation: collection link/content checks and package file/hash comparisons in the package inspection script.
- Invariants: no changes to per-version Markdown caches, syntax highlighting, cached text handles, document persistence, or file-tree rendering. No new dependencies.
