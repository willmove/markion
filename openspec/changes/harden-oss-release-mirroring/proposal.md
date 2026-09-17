## Why

The v0.3.10 release exposed a repeatable failure in the direct `curl` PUT mirror path: GitHub-hosted runners connected to Aliyun OSS but received no response for 30 minutes while uploading the first 21 MB installer. The release mirror needs a supported transfer client that can complete or fail promptly enough for operators to repair an already-published release safely.

## What Changes

- Replace the hand-written HMAC-SHA1 REST upload loop with the official Aliyun `ossutil` client, pinned and checksum-verified in CI.
- Preserve byte-for-byte installer mirroring, stable `latest/` object keys, secret-backed credentials, repair dispatches, and public reachability checks.
- Bound transfer retries and timeouts so an unavailable OSS write path produces an actionable failure instead of hours of serial hangs.
- Document the supported mirror client and repair behavior in the canonical release runbook.

Non-goals: changing release assets, updater signatures, public tags, application behavior, or cached Markdown/editor state.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-packaging`: Require the OSS mirror to use a supported, bounded, resumable upload path while retaining current artifact integrity and repair guarantees.

## Impact

- `.github/workflows/release.yml`: OSS client setup and upload implementation.
- `docs/release-process.md`: mirror implementation and recovery guidance.
- GitHub Actions downloads the pinned official Aliyun CLI binary during the mirror job; no runtime application dependency is added.
- No project architecture or cached-per-version Markdown invariant is touched.
