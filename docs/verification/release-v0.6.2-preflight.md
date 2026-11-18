# Markion v0.6.2 release verification

The previous stable release is `v0.6.1`. The release includes mandatory Developer ID signing and Apple notarization, fixes for saving and draft recovery, GFM table boundaries, cross-platform PDF font fallback, block format indicators, and a refreshed icon. Existing documents and settings require no manual migration.

## Release-specific acceptance scope

On 2026-11-18 (Asia/Shanghai), the maintainer explicitly authorized an exception for the remaining manual acceptance matrix, with disclosure in the English and Simplified Chinese release notes.

未执行的人工验收包括各平台安装包的完整交互与 DPI 组合、真实输入法、浏览器剪贴板和 Word／微信互操作，以及上一稳定版 Windows／AppImage 的签名升级、失败回退与长时间运行。本次例外仅适用于 v0.6.2，这些项目仍记为未验证。

The macOS checks remain required. An Apple Silicon runner and an Intel runner must each assess and open the app installed from its final release DMG. GitHub and OSS publication depend on both launch jobs. Windows and Linux native packaging, AppImage smoke, updater signatures, final public asset verification, and bilingual notes also remain required.

## Earlier signing and startup rehearsals

- The [initial signing rehearsal](https://github.com/willmove/markion/actions/runs/44283630532) exercised real signing credentials. The [Intel installation rehearsal](https://github.com/willmove/markion/actions/runs/44307203859) verified the Intel DMG and its embedded app through Gatekeeper on x86_64 macOS 15.7.9. It also confirmed that the installed application opened a visible Markion window. Its DMG SHA-256 was `49544ae46079e56379ebefab1422f8b21c0a23669a1a3ce725fc352ebf0e536e`.
- The [Windows installer rehearsal](https://github.com/willmove/markion/actions/runs/44287236073) installed the NSIS package and extracted the portable ZIP into separate test locations. Both opened a Markion window.
- The [main packaging rehearsal](https://github.com/willmove/markion/actions/runs/44285479855) passed the native build matrix and Linux AppImage smoke with no system Vulkan driver.
- A synthetic [acceptance fixture run](../testing/fixtures/README.md) verified 160 source and working files. Its fixture manifest SHA-256 was `ed19e977d4f540a1ff60f4f32149985bf6a59bc2edea6f88db46a7708b886ab2`.

The signed rehearsal and native smoke tests above cover their recorded artifacts. Final release assets are verified separately and carry their own hashes.

## Apple Silicon browser installation before the final editor changes

A macOS 27.2.1 Apple Silicon host independently built v0.6.2 from rehearsal commit `585f174938aa`. The rehearsal preceded the additional list editing, rich import, and export fixes on main and does not qualify their installers. The application and DMG both passed Developer ID and Gatekeeper checks. Apple approved the DMG after the initial one-hour wait expired, so the original submission was resumed by ID and stapled successfully. Its finished SHA-256 was `0f8323481d05d76266407ae2dae9ec4cd50cad4a97a3da170ab9999002027a50`.

Safari downloaded the candidate DMG from a loopback test URL, retaining macOS quarantine. Finder mounted the image and copied Markion to an isolated Applications folder. macOS showed its normal first-open confirmation stating that Apple had checked the app. Confirming it opened `Markion - Untitled.md` with the isolated profile. Both signatures and both stapled tickets were also validated on disk.

The release workflow now allows up to three hours for an individual disk-image verdict. The longer wait only affects how long CI waits for Apple. It leaves the application sources, resources, and package configuration unchanged.

The same host also downloaded the [GitHub rehearsal's arm64 artifact](https://github.com/willmove/markion/actions/runs/44308180701), verified its archive checksum, assessed the exact DMG and installed application, and opened the Markion window. The rehearsal DMG SHA-256 was `ba9eac4355e4f1a94a9e2f52e786f5ef4f483182c1fb1191a842e121cbe8287d`.
