## Context

`FileIdentity` records volume, object id and link count. Ordinary files require one link, but Unix directory link counts change when child directories are created or removed. `ReadGrant::validate` and `Journal::grant` currently compare the complete tuple for directories, invalidating an approved plan's own folder creation. The release suite reproduces this in `durable_roundtrip_and_newer_content_conflict`.

## Goals / Non-Goals

**Goals:** preserve grants and durable journals across ordinary child-directory changes while rejecting replacement directories, symlinks and hard-linked files.

**Non-Goals:** permission expansion, altered file baselines, automatic replay, or persistent schema migration.

## Decisions

- Add a named directory-only identity comparison using volume/object id. Use it only for grant root/scope directory checks; regular-file equality and `ordinary` retain full link-count checks.
- Keep serialized `links` fields untouched. Existing version-1 journals with an earlier directory count continue to identify the same filesystem object without rewriting their schema.
- Keep directory post-operation records and all containment, source-content, dependency, review, and collision checks intact. Markdown mutations and derived caches are outside this path.

## Risks / Trade-offs

- [Ignoring directory link count could hide a replaced object] → volume and object id must still match; retain directory-replacement regression coverage and symlink rejection.
- [Accidentally applying the helper to file baselines] → give it a directory-specific name, document its restriction, and retain ordinary-file hard-link tests.
