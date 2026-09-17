# Native plugin feasibility evidence

## Accepted run

- Workflow: `plugin-platform-spike`
- GitHub Actions run: `35196344240`
- Candidate revision: `cc4b3ea284f4c9226d772cf23381e8572b4b5bd0`
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
| NSIS | 274,133 B | 794,099 B | 774,443 B | 1,303,053 B | pass |
| DMG | 312,620 B | 367,422 B | 735,956 B | 1,729,730 B | pass |
| DEB | 319,664 B | 440,942 B | 728,912 B | 1,656,210 B | pass |
| AppImage | 339,968 B | 443,382 B | 708,608 B | 1,653,770 B | pass |

The installed deltas consist primarily of a 366,416–792,576 byte executable
increase plus the 2,254–2,258 byte bootstrap catalog/signature pair; the core
notice file became smaller, and NSIS also changed its uninstaller by 544 bytes.
No plugin worker, plugin archive, rollback payload, or PDFium runtime was
included in these host candidates.

## Real PDF plugin payload

The accepted `pdf-size-spike` run `35196344203` packaged the real
`markion-plugin-pdf` worker plus each target's pinned PDFium runtime. Every
signed archive was authenticated, defensively extracted beneath a path with
spaces and non-ASCII characters, launched from that exact extraction, and used
to open and rasterize the one-page fixture before clean shutdown. Release run
`35196344189` independently rebuilt all three plugin artifacts and the signed
catalog successfully.

| Target | Archive bytes | Installed bytes | Archive headroom | Installed headroom | Result |
|---|---:|---:|---:|---:|---|
| Windows x86_64 | 4,053,075 B | 8,311,115 B | 2,238,381 B | 2,174,645 B | pass |
| macOS arm64 | 3,922,436 B | 8,915,860 B | 2,369,020 B | 1,569,900 B | pass |
| Linux x86_64 | 4,073,003 B | 9,004,776 B | 2,218,453 B | 1,480,984 B | pass |

All three reports remain below the 6,291,456-byte archive and
10,485,760-byte extracted limits. The earlier local Windows result was
4,047,124 B / 8,297,772 B; the accepted native CI result is 5,951 B larger
compressed and 13,343 B larger extracted, reflecting the final CI
toolchain/revision rather than the earlier local composition.

## Interpretation for the product decision

The completed reusable plugin host adds about 0.26–0.32 MiB to installers and
0.35–0.76 MiB after installation—within the planning estimate of
0.25–0.75 MiB / 0.6–1.5 MiB (with macOS and Linux installed deltas even lower)
and the hard 1 MiB / 2 MiB gates. The optional PDF plugin is about
3.74–3.88 MiB to download and 7.93–8.59 MiB installed. Therefore users who do
not install PDF support avoid the entire PDFium/plugin payload, while the core
pays well under 1 MiB for the reusable platform.
