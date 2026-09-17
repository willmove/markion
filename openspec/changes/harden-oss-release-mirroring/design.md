## Context

See `proposal.md` for the incident motivation. The current mirror signs a whole-object REST PUT in shell and gives each attempt 1,800 seconds. In two independent GitHub-hosted runners, the first 21 MB upload sent no publicly visible object and returned no response for the full timeout. The release is already public, so recovery must operate from immutable Release assets and must not move `v0.3.10`.

## Goals / Non-Goals

**Goals:**

- Use Alibaba Cloud's maintained transfer implementation, including multipart upload and resumable retry behavior.
- Pin and checksum-verify the CI tool so a moving external download cannot silently change release behavior.
- Keep credentials in environment variables and keep the existing post-upload public verification.
- Make prolonged write-path failures terminate within a practical, documented bound.

**Non-Goals:**

- Rebuild, re-sign, or modify published installers.
- Change updater or manifest formats, OSS object locations, or application code.
- Add a runtime dependency to Markion.

## Decisions

### Use pinned `ossutil` 2.x for uploads

The mirror job will download the official Linux amd64 archive from Alibaba Cloud, verify the published SHA-256 checksum, and invoke `ossutil cp` for each exact destination. Credentials, endpoint, and region remain environment-backed. This replaces custom request signing and allows the official client to select multipart/resume behavior.

Alternative: retain `curl` and tweak HTTP headers or protocol selection. This is smaller but leaves authentication, transfer behavior, and retry semantics in bespoke shell and does not address the repeated no-response failure confidently.

### Keep per-file uploads and existing verification

Files remain explicit so logs identify the failed object and the workflow can preserve the current artifact allowlist. The public HTTP 200 and manifest-version checks remain the release gate. Installer digests continue to be generated before upload.

Alternative: recursively copy the entire `dist` directory. This is less verbose but could publish unintended artifacts and weakens the explicit release contract.

### Bound retries at the job level

Each copy uses a finite request timeout and the shell retry loop remains small. A failed object terminates the step instead of allowing four 30-minute attempts. A manually dispatched repair is the supported resume path and reuses published assets.

## Risks / Trade-offs

- [The official binary download is unavailable] → Pin the URL and checksum; fail before accessing credentials or changing OSS.
- [CLI flags or behavior change upstream] → Pin an exact version and update it only through a reviewed change.
- [A partial multipart upload remains after failure] → Let `ossutil` resume/overwrite the same key; final public reachability and metadata checks remain mandatory.
- [The endpoint secret lacks a region derivable by the client] → Set the known `cn-heyuan` region explicitly while retaining the configured endpoint.

## Migration Plan

1. Update and validate the workflow and release runbook on `main`.
2. Dispatch `release.yml` with `mirror_tag=v0.3.10`; the repair job downloads existing GitHub assets.
3. Verify all required OSS objects, versions, URLs, and signature parity.
4. Roll back the workflow commit if the official client cannot operate with the existing credentials; the public tag and GitHub Release remain unchanged.
