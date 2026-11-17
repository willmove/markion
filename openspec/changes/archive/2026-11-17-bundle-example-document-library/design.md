## Context

See proposal.md for motivation. Native packaging already includes the full `assets` tree, including the portable ZIP. `scripts/verify-packaged-workspace.ps1` extracts every supported format and finds the packaged `assets/marknice-workspace` tree. `bundled_resource_path` already handles executable-adjacent resources, macOS Resources, Linux `/usr/lib/markion`, and the development checkout. Both menu surfaces dispatch the same GPUI actions.

## Goals / Non-Goals

**Goals:** A useful, portable collection with no network dependency and a discoverable entry that works across supported resource layouts.

**Non-Goals:** Introducing a template registry, changing first-launch/session behavior, opening multiple example tabs automatically, or copying user data at startup.

## Decisions

### Keep documents as ordinary checked-in Markdown

Use numbered, ASCII filenames and Chinese prose, plus a bilingual `README.md` index. Include five practical guides and twelve scenario templates with realistic examples, placeholders, and exercises. Copy the existing Markion PNG into `assets/examples/images` so copying only `examples` preserves its local image references. No images are downloaded and no new artwork is needed. Keeping ordinary files avoids a bespoke template engine and lets users browse, duplicate, and edit them through existing workflows.

### Reveal the index through Help

Resolve `assets/examples/README.md` with the existing shared resource lookup and pass it to GPUI's system file-manager reveal API. The native and custom menus use the same action and localized label; missing resources set a localized status. The index explains copying the directory into user-owned storage before editing, especially for read-only macOS/Linux bundles. Opening the resource tree directly as a writable workspace or creating copies implicitly could expose installed originals to autosave or surprise users with new files.

Data flow: Help action → resource path lookup → file-manager reveal (or localized missing-resource status). Only menu/status state changes. No editor load, mutation, derived-cache invalidation, or document-version change occurs. The welcome constant gains a short fixed-English pointer, retaining its existing syntax tour.

### Check the payload within the existing package inspection

Add a reusable PowerShell collection verifier that checks UTF-8 Markdown, nonempty documents, index coverage, and local Markdown/image links, then optionally compares file hashes against a packaged collection. Package inspection invokes it on the sibling `examples` tree beside each discovered publishing workspace. A missing examples tree cannot pass because another unrelated directory happens to contain an index. Compare source and packaged content after normalizing neither bytes nor line endings; the checkout used to build is the expected payload.

## Risks / Trade-offs

- [Installed files may be read-only or overwritten] → Explicit copy-first guidance in the index and getting-started guide; the Help action does not load editable documents.
- [Chinese-first content is less accessible to other locales] → A bilingual index explains the collection and identifies each document; menu labels follow all supported UI locales. Full translated template sets can be added independently.
- [Export fidelity differs by format] → Exercises ask users to review exported output and keep editable Markdown; avoid promises about pixel-identical HTML, diagrams, or math in every export format.
- [Concurrent unrelated working-tree edits] → Apply narrow patches and verify only the intended diff without resetting existing work.

## Migration Plan

No data migration. Existing resource packaging picks up the directory on the next build. Rollback removes the collection, Help entry, welcome pointer, and package check without affecting user copies. This change does not tag or publish a release.
