## 1. Supported OSS transfer path

- [ ] 1.1 Pin, download, and checksum-verify the official Linux amd64 `ossutil` binary in the mirror job; verify the workflow exposes no credential values and the binary reports its expected version.
- [ ] 1.2 Replace direct signed `curl` PUTs with explicit bounded `ossutil cp` uploads for the existing artifact allowlist; verify YAML parsing and shell syntax checks pass.

## 2. Release operations and validation

- [ ] 2.1 Update `docs/release-process.md` to describe the pinned official client, bounded failure behavior, and stable-tag repair path; verify documentation matches the workflow.
- [ ] 2.2 Validate the OpenSpec change and repository diff, then commit and push the fix to `main`; verify the branch workflow succeeds.
- [ ] 2.3 Dispatch the `mirror_tag=v0.3.10` repair, verify every required OSS object returns HTTP 200, both manifests report `0.3.10`, updater URLs/signatures are consistent, and the public tag/GitHub Release remain unchanged.
