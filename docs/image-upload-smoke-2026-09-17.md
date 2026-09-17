# Image upload smoke evidence — 2026-09-17

Environment: Windows 10.0.26200 x64 (`D:\Coding\EditorProjects\markion`).

| Check | Result | Evidence |
| --- | --- | --- |
| PicGo GUI HTTP, real installation | Unavailable | `Get-Command picgo,picgo.exe` returned no command; no configured PicGo loopback server was available. |
| PicGo Core, real installation | Unavailable | PicGo CLI was not installed. Node was available at `C:\Applications\nodejs\node26\node.exe`, but no PicGo entry script or real host configuration was present. |
| Custom uploader protocol | Automated fixture passed | The external-command and custom-adapter tests exercise explicit `pwsh.exe`/fixture programs, literal Unicode/metacharacter paths, one-line URL parsing, nonzero exit, output bounds, timeout, cancellation, and owned child cleanup. PowerShell 7 was available at `C:\Users\willm\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe`. No cloud object was created because repository validation must not require credentials. |
| Windows hidden-console implementation | Automated verification only | The runner sets `CREATE_NO_WINDOW`, owns the process tree, and its Windows fixture tests passed. A packaged NSIS application with a real PicGo/custom uploader was not available for visual confirmation. |
| macOS PicGo HTTP/Core/custom | Unavailable on this host | No connected macOS runner or installation. |
| Linux PicGo HTTP/Core/custom | Unavailable on this host | No connected Linux runner or installation. |

Task 8.3 remains incomplete until the unavailable real-tool and cross-platform checks are performed. This evidence must not be interpreted as a passed manual release matrix.
