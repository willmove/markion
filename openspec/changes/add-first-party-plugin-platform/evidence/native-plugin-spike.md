# Native plugin feasibility evidence

## Accepted run

- Workflow: `plugin-platform-spike`
- GitHub Actions run: `35181072438`
- Candidate revision: `7d6cd0e8e6dc47686df54e2ff7a9cb4d3d388cf2`
- Immutable host control: `d99624aa560454820fa2443f626cd40cc8217446`
- Toolchain: Rust 1.98.1, cargo-packager 0.11.8, PowerShell 7.6.5
- Result: Windows x86_64, macOS arm64, and Linux x86_64 jobs all passed.

The fixture archive was signed, authenticated, defensively extracted, launched
directly from the extracted directory, handshaken over the framed protocol, and
terminated with its child-process cleanup check on every target.

| Target | Direct launch | Handshake | Forced child cleanup | Platform-policy result |
|---|---:|---:|---:|---|
| Windows x86_64 | yes | yes | yes | A copied worker with MOTW launched directly; the Zone.Identifier stream remained present. |
| macOS arm64 | yes | yes | yes | The app-managed, unquarantined extracted worker launched. An explicitly quarantined unsigned copy was rejected by Gatekeeper and the quarantine xattr remained present. Markion must surface that rejection and must not strip the xattr. |
| Linux x86_64 | yes | yes | yes | Extracted executable mode was exactly `755`. |

The macOS result establishes a reliable app-managed first-party launch path for
the project's currently unsigned distribution model, while preserving the OS
decision for externally quarantined packages. A future signed/notarized app can
sign/notarize the plugin worker independently without changing the package or
protocol design.

## Minimal host overhead

All values are byte-exact same-runner candidate-minus-control deltas. The hard
limits are 1,048,576 compressed bytes and 2,097,152 installed bytes.

| Format | Package delta | Installed delta | Package headroom | Installed headroom | Result |
|---|---:|---:|---:|---:|---|
| NSIS | 75,681 B | 215,675 B | 972,895 B | 1,881,477 B | pass |
| DMG | 80,141 B | 171,188 B | 968,435 B | 1,925,964 B | pass |
| DEB | 85,336 B | 209,396 B | 963,240 B | 1,887,756 B | pass |
| AppImage | 135,168 B | 211,796 B | 913,408 B | 1,885,356 B | pass |

The installed deltas consist of a 168,288–212,480 byte executable increase plus
the 2,900–2,995 byte bootstrap catalog. No plugin worker or PDFium runtime was
included in these host candidates.

## PDF payload prototype

The accepted run packaged the native smoke worker plus each target's pinned
PDFium runtime to validate the archive/signature/extraction and payload budget.
The real protocol PDF worker is measured separately below and replaces this
smoke executable in the next run.

| Target | Archive bytes | Installed bytes | Archive limit | Installed limit | Result |
|---|---:|---:|---:|---:|---|
| Windows x86_64 | 3,929,766 B | 8,039,787 B | 6,291,456 B | 10,485,760 B | pass |
| macOS arm64 | 3,815,994 B | 8,713,271 B | 6,291,456 B | 10,485,760 B | pass |
| Linux x86_64 | 3,955,016 B | 8,762,516 B | 6,291,456 B | 10,485,760 B | pass |

The 2026-09-17 Windows local composition with the real
`markion-plugin-pdf` protocol worker produced a reproducible 4,047,124-byte
archive and an 8,297,772-byte extracted payload. Its 1,082,880-byte worker,
7,211,520-byte PDFium DLL, manifest, signature, and notices all remain inside
the 6 MiB / 10 MiB gates. The exact signed archive was extracted beneath a
space-and-non-ASCII path, launched, and used to open and rasterize the one-page
fixture before clean shutdown. Native CI remains the final authority for the
real worker on every target.

## Interpretation for the product decision

The reusable plugin host adds only about 0.07–0.13 MiB to installers and
0.16–0.21 MiB after installation in this measured skeleton—well below the
planning estimate of 0.25–0.75 MiB / 0.6–1.5 MiB and the hard 1 MiB / 2 MiB
gates. The optional PDF plugin is about 3.64–3.77 MiB to download and
7.67–8.36 MiB installed in the accepted prototype matrix. Therefore users who
do not install PDF support avoid essentially the entire PDFium payload, while
the core pays only a few hundred KiB for the reusable platform.
