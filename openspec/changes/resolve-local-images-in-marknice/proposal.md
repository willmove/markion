## Why

The WeChat publishing workspace (MarkNice preview) only resolves local images that live inside the managed `<stem>.assets/` directory, so the most common authoring habits — images sitting next to the Markdown file, in subfolders, or one level up (`../cover.png`) — all render as broken images with an "N local images unresolved" warning. Users who author notes outside Markion (Typora/Obsidian-style layouts) get a preview that appears broken even though every referenced file exists on disk.

## What Changes

- **Widen the publishing image scope (transparent fix).** For a saved document, the local loopback workspace will resolve and serve any supported image referenced by the document whose canonical location lies within the **parent directory of the document's own directory** — that is: the same directory as the `.md` file, any subdirectory at any depth below it, and up to exactly one directory level above it. References that would escape above the parent level (`../../x.png`), absolute paths, `file:` URLs, and unsupported or missing files remain unresolved. Untitled documents keep zero filesystem authority.
- **Add an "Organize local images" command (escape hatch).** A new user-initiated action scans the active saved document for local image references that are *not* resolvable in the widened scope (e.g. `../../shared/logo.png` or absolute paths), shows a confirmation prompt listing the images, and on confirmation copies each file into `<stem>.assets/` (reusing the existing collision-safe, content-hashed import naming) and rewrites the references in one undoable document mutation. After organizing, the images preview through the normal managed-resource path.
- The loopback server's containment root changes from the asset directory to this document-relative scope; all existing protections (only document-referenced images enumerated, opaque hash IDs, bearer-auth session routes, extension whitelist, canonical containment with symlink re-check at read time, no path disclosure) are preserved.

Non-goals: no WeChat media upload or draft publishing (that is the separate `add-wechat-draft-publishing` change), no change to the rich-copy behavior that strips local images for WeChat pasting, no changes to the bundled MarkNice frontend (`bridge.js`, CSP, and manifest stay byte-identical), no `file://` URL support.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `wechat-publishing-workspace`: "Loopback sessions protect document content and local files" and "Local images preview safely and copy with an explicit limitation" change their local-file containment scope from the document-associated asset directory to the widened document-relative publishing image scope (document directory tree plus exactly one level above it).
- `document-resources`: gains a requirement for the confirmed, undoable "organize local images" action that copies out-of-scope referenced images into the asset directory and rewrites their references.

## Impact

- `crates/wechat-workspace/src/resource.rs` — `PublishingResource` containment root becomes caller-supplied (the scope root) instead of implicitly the asset directory; the lexical reference check permits `../` components and lets canonical containment decide.
- `src/publishing.rs` — computes the widened scope root for the saved document and classifies references into `resources` / `unresolved_local_images`.
- `src/storage/resources.rs` — new organize planner that resolves out-of-scope references to candidate files (reusing `import_image_file` for the copy).
- `src/app/` — new `OrganizeLocalImages` action, menu item in the Export menu next to the WeChat publishing entry, GPUI confirmation prompt, status reporting; `src/i18n.rs` gains strings for all seven languages.
- Tests: `crates/wechat-workspace` (containment, traversal, symlink), `src/publishing.rs` (scope classification), app-level tests for the organize flow (confirm/cancel, partial failure, undo).
- Architecture invariants preserved: the snapshot build stays an export-time scan that never touches per-version derived caches; the organize command is the only part that mutates the document, and it does so as one undoable edit.
