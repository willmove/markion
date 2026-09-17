# Core/PDF size baseline

This record freezes the same-runner evidence used by the plugin extraction work. Byte counts are taken from the successful `pdf-size-spike` run at the candidate revision below; they are not reconstructed from a later local build.

## Revisions and build identity

- PDF-disabled control: `546aeed1a2cc205150430b87410565b7ccd620a3`
- Built-in-PDF candidate: `d99624aa560454820fa2443f626cd40cc8217446`
- GitHub Actions run: `34669535100` (`https://github.com/willmove/markion/actions/runs/34669535100`)
- Workflow conclusion: `success`
- Toolchain recorded by every report: Rust/Cargo `1.98.1`, cargo-packager `0.11.8`, PowerShell `7.6.5`
- Unit: bytes; MiB means 1,048,576 bytes

## Immutable artifact and installed-payload manifest

| Format | Target | Control package | PDF candidate package | Package delta | Control installed | PDF candidate installed | Installed delta | Control/candidate files |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| NSIS | `x86_64-pc-windows-msvc` | 21,096,018 | 24,061,282 | 2,965,264 | 65,628,362 | 73,662,510 | 8,034,148 | 52 / 53 |
| DMG/app | `aarch64-apple-darwin` | 26,689,989 | 30,299,071 | 3,609,082 | 52,109,944 | 60,562,200 | 8,452,256 | 46 / 47 |
| DEB | `x86_64-unknown-linux-gnu` | 31,321,712 | 35,092,460 | 3,770,748 | 66,596,361 | 75,122,857 | 8,526,496 | 46 / 47 |
| AppImage | `x86_64-unknown-linux-gnu` | 30,140,920 | 33,532,408 | 3,391,488 | 67,317,095 | 75,863,575 | 8,546,480 | 56 / 57 |

All four original reports passed the 6 MiB compressed and 10 MiB installed PDF gates. Their largest changes also establish what extraction should remove from the core package:

| Format | PDFium/runtime bytes | Root executable growth | Notice growth |
|---|---:|---:|---:|
| NSIS | 7,211,520 | 821,760 | 1,279 |
| DMG/app | 7,732,336 | 718,672 | 1,248 |
| DEB | 7,645,184 | 880,064 | 1,248 |
| AppImage | 7,664,592 | 880,640 | 1,248 |

## Downloaded report integrity

The reports downloaded from run `34669535100` are retained locally under the ignored `target/ci-size-report-d99624a/` directory. These hashes identify the exact evidence used above:

| Report | Length | SHA-256 |
|---|---:|---|
| `nsis.json` | 1,800 | `627fd92a244d887d17f7e5984d5c134e3ef80f8afa46bcd8964d85ea2898126f` |
| `dmg.json` | 1,707 | `16ccefd08eae96bba2c6bbb40f1f9a49995a7b5a341ad4a8c2ca593d8e471d8d` |
| `deb.json` | 1,675 | `f075af41141df7665f351eb658fe6b9c71baf384d8a7451390fea863e857e069` |
| `appimage.json` | 1,710 | `3f2f147898e8c8b28d6b95e40e01fd1d5359e92a5d889969669db7ff4de5f5b7` |

## Acceptance budgets for this change

- Complete plugin host in the core package: at most 1,048,576 compressed bytes and 2,097,152 installed bytes above a same-runner PDF-disabled control.
- Official PDF plugin: at most 6,291,456 compressed bytes and 10,485,760 extracted bytes per target.
- The final decision uses new same-toolchain control/candidate reports; this baseline only fixes the comparison point and expected recoverable payload.
