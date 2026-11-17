# Screenshot sources · 截图来源

The README images are **actual Windows application captures**, saved directly from the official Computer Use window screenshot output. They are not mockups or generated illustrations. No screenshot compositing or retouching was applied.

README 中的图片均为 **Windows 实机窗口截图**，直接保存自官方 Computer Use 工具，未拼接或修饰，也未使用生成式图片。

| Image | Captured surface |
| --- | --- |
| [markion-workspace-windows.png](markion-workspace-windows.png) | Visual Edit with left Files and an independent right Outline; a callout, task list and table |
| [markion-block-menu-windows.png](markion-block-menu-windows.png) | The same workspace with the paragraph context menu: block types, actions, inline formatting and insertion |
| [markion-korean-workspace-windows.png](markion-korean-workspace-windows.png) | Current-source Korean menus, Files, Outline and status bar; authored English Markdown stays unchanged |
| [markion-korean-preferences-windows.png](markion-korean-preferences-windows.png) | Korean General preferences and the active 한국어 language choice |
| [markion-korean-ai-settings-windows.png](markion-korean-ai-settings-windows.png) | Korean AI settings, with AI off and no account credentials entered |

## English capture record

- **Date:** 2026-11-17, Asia/Shanghai.
- **Platform:** Windows, native GPUI window; 1,122 × 790 capture pixels.
- **Build:** v0.6.0 development executable reused from `target/debug/markion.exe`, not an installed release-package screenshot. Binary SHA-256: `FBFBB2C330468F58B668FA25957D8073A95A4CD918456650DEE00BEDB4A7981C`.
- **Repository at capture:** `174c1e821cd12faa8fa9980e394f257ca6d3d26d`. The reused executable is identified by its hash; this is not a claim that it was rebuilt at that exact commit.
- **Document:** [sample-workspace/Welcome.md](sample-workspace/Welcome.md); the adjacent meeting note and project plan populate Files. All document content is an authored demo, without personal files or credentials.
- **Appearance:** Paper theme, English UI, 16 px reading text, Consolas code/source font. The local test-enabled build requires an installed code font, so the capture configuration explicitly selected Consolas.
- **Layout:** 1,120 × 760 requested window content area; approximately 190 px Files and 205 px Outline. Window chrome adds the remaining capture area.
- **Isolation:** a dedicated executable copy, `APPDATA` and `LOCALAPPDATA` under `target/tmp/readme-refresh`; auto-save and startup update checks disabled. No personal preferences or documents were edited. This process was stopped after capture.

## Korean capture record

- **Date/platform:** 2026-11-17, Asia/Shanghai; Windows native GPUI, 1,122 × 790 pixels.
- **Build:** current source with Korean UI and localized startup status, after the published v0.6.0 release. The package version remains 0.6.0; these screenshots do not describe the already-published binaries.
- **Binary SHA-256:** `454B74DBB1C8B07A1F6FD92E4784D3D036375DEA879BC0AD1FDC6D1A657A7DEB`.
- **Configuration:** Paper, 16 px reading text, Consolas source/code; dedicated `APPDATA`/`LOCALAPPDATA` under `target/tmp/readme-refresh/korean-native`. The sample workspace was copied to a newly created temporary folder outside the repository. Auto-save and update checks were disabled.
- **Verification:** English → 한국어 through the actual Preferences control immediately relabeled menus, General and AI settings. `language = "ko"` was saved and survived restart. A startup-only English Ready label discovered during the first restart was corrected; the final screenshot shows `준비됨`.
- **Document integrity:** the copied `Welcome.md` and checked-in sample both retained SHA-256 `EAF59227D9DE23980A0D1448399AAE618512E007A8ACA7C650D911D543EE38DC` throughout switching and restarts.
- **Additional evidence:** [Korean native File menu](../verification/korean-native-menu-windows-2026-11-17.png). No AI requests were sent and no credentials were entered. The isolated app was closed after capture.

## Recreate the scene

1. Build Markion (`cargo build --bin markion`) or use the same development executable if available.
2. Copy the complete [sample workspace](sample-workspace) to a scratch directory outside your normal notes. Open that directory through **File → Open Folder**, then open `Welcome.md`.
3. In **Preferences → Appearance**, select Paper, a 16 px reading font size, and an installed code font (Consolas on this Windows machine).
4. In **Preferences → General**, put Outline on the right and keep both Files and Outline visible. Resize to the dimensions above.
5. Enter Visual Edit; place the caret in the plain introductory paragraph so heading markers remain hidden. Capture the window. Right-click that paragraph and capture the block menu for the second image.

The images demonstrate this Windows build and sample document. Other platforms and layouts can look different. Performance measurements have separate provenance in [performance.md](../performance.md).

The README layout takes inspiration from [quiet-md's README](https://github.com/s1dashu/quiet-md/blob/main/README.md). All product screenshots and measurements linked here are Markion's own.
