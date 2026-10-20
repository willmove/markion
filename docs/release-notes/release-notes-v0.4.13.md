# Markion v0.4.13

Packaging and distribution maintenance release — the application itself is unchanged from v0.4.11. This release makes the China download mirror reliable for large installers, adds the pipeline support needed for notarized macOS DMGs, and corrects the macOS first-launch documentation. (v0.4.12 was tagged but its packaging workflow failed before anything was published; no release or downloads ever existed for it, and v0.4.13 is the next installable version.)

## Highlights

### Reliable China mirror for large installers

- Files larger than 16 MiB — both macOS DMGs, the DEB/RPM packages, the Windows portable `.zip`, and the ~52 MB Linux AppImage — now upload to the Aliyun OSS mirror as multipart transfers with 8 MiB parts, each part on its own connection with independent retries. A single capped 30-minute upload previously stalled forever when the CI-to-OSS route degraded, which is exactly what delayed the v0.4.11 mirror and required a manual repair.
- Two signing fixes make those multipart uploads authenticate correctly: the OSS upload id is now percent-encoded in request URLs (kept raw in the string-to-sign), and the content type curl actually sends is included in the signature.
- The OSS `latest/` channel — used for faster downloads in China and by the in-app updater's install URLs — now updates completely on every release. The same code already repaired and fully re-served the v0.4.11 mirror (all seven packages and `update.json` verified byte-identical in size to the GitHub assets).

### Groundwork for notarized macOS DMGs

- The release pipeline can now sign both macOS builds with an Apple Developer ID (hardened runtime) and notarize and staple the `.app` and both DMGs, controlled by six repository secrets. A partially configured secret set fails the macOS jobs instead of silently degrading, and a tag build without credentials logs a clear warning. Each release's signing mode is verified in CI before upload.
- **The Apple credentials are not configured yet, so this release's DMGs remain ad-hoc and are not notarized** — see Compatibility.

### Corrected macOS first-launch documentation

- The README and FAQ no longer claim that Gatekeeper's *Open Anyway* reliably opens Markion's ad-hoc-signed DMGs. On current macOS, a browser-downloaded ad-hoc app is typically refused with a "damaged and can't be opened" message. The documentation now gives the dependable routes: download with `curl -LO` (no quarantine flag), or clear the flag once with `xattr -cr /Applications/Markion.app` after a browser download. All README translations and the FAQ were updated.

## Compatibility

- No migration is required for Markdown files, preferences, or workspace data — the application code is identical to v0.4.11, and the in-app update flow is unchanged.
- macOS DMGs in this release are ad-hoc signed and **not notarized**. After a browser download, Gatekeeper may report Markion as "damaged"; the file is intact and opens after `xattr -cr /Applications/Markion.app` (or when downloaded with `curl -LO`). Notarized DMGs — which will open directly — require the maintainer to provision Apple credentials as documented in `docs/macos-signing.md`.
- Windows installers remain unsigned; SmartScreen may require **More info → Run anyway**.

## Downloads

- Windows x64: NSIS installer and portable `.zip`.
- macOS Apple Silicon: DMG (`aarch64`).
- macOS Intel: DMG (`x64`).
- Linux x86_64: DEB, RPM, and AppImage.

## Verification

- Local workspace suite: all crates pass except known machine-specific failures on the release machine (macOS symlinked temp paths and APFS directory-identity semantics trip 19 AI-journaling/policy tests; two markion-pdf font tests fail locally as always). None are affected by this release, whose application code is unchanged from v0.4.11. The packaged MarkNice workspace bundle verification succeeds.
- Native CI builds and packaging pass on Windows, macOS (Apple Silicon and x86_64 cross-compiled), and Linux, including the no-GPU AppImage smoke test; the updater installers are signed and mirrored with a version-matched `update.json`.

**Full comparison**: https://github.com/willmove/markion/compare/v0.4.11...v0.4.13

---

# Markion v0.4.13（中文说明）

打包与分发维护版本——应用本身与 v0.4.11 完全一致。本版本让中国下载镜像对大型安装包变得可靠，加入了公证版 macOS DMG 所需的流水线支持，并更正了 macOS 首次启动文档。（v0.4.12 曾打标签但其打包工作流在任何内容发布之前即告失败；该版本从未存在 Release 或下载，v0.4.13 是下一个可安装版本。）

## 主要更新

### 大型安装包的中国镜像更可靠

- 大于 16 MiB 的文件——两个 macOS DMG、DEB/RPM 包、Windows 便携 `.zip` 以及约 52 MB 的 Linux AppImage——现在以 8 MiB 分片的多部分上传方式传送到阿里云 OSS 镜像，每个分片独立连接、独立重试。此前单次上传受 30 分钟上限约束，CI 到 OSS 的线路一旦降速就会永远卡死，v0.4.11 的镜像正是因此延误并需要手工修复。
- 两处签名修复让多部分上传能正确通过认证：OSS upload id 现在只在请求 URL 中做百分号编码（参与签名的字符串保持原值），签名也加入了 curl 实际发送的 Content-Type。
- 供中国用户加速下载、并被应用内更新器用作安装地址的 OSS `latest/` 通道，现在每次发布都会完整更新。同一套代码已经修复并完整重现了 v0.4.11 镜像（七个安装包与 `update.json` 均已核验，大小与 GitHub 资产逐字节一致）。

### 公证版 macOS DMG 的前置工作

- 发布流水线现在支持用 Apple Developer ID（加固运行时）签名两个 macOS 构建，并对 `.app` 与两个 DMG 做公证与装订（staple），由六个仓库密钥控制。密钥配置不完整会让 macOS 构建直接失败而不是悄悄降级，缺少凭据的标签构建会输出明确警告。每次发布的签名模式都会在上传前经 CI 校验。
- **Apple 凭据尚未配置，因此本版本的 DMG 仍是 ad-hoc 签名、未经公证**——见“兼容性”。

### 更正 macOS 首次启动文档

- README 与 FAQ 不再声称 Gatekeeper 的“仍要打开”总能放行 Markion 的 ad-hoc 签名 DMG。在当前的 macOS 上，浏览器下载的 ad-hoc 应用通常会被以“已损坏，无法打开”直接拒绝。文档现在给出可靠做法：用 `curl -LO` 下载（不会设置隔离标记），或在浏览器下载后执行一次 `xattr -cr /Applications/Markion.app` 清除隔离标记。全部 README 译文与 FAQ 均已更新。

## 兼容性

- Markdown 文件、偏好设置、工作区数据均无需迁移——应用代码与 v0.4.11 完全一致，应用内更新流程不变。
- 本版本的 macOS DMG 为 ad-hoc 签名且**未经公证**。浏览器下载后 Gatekeeper 可能报告 Markion“已损坏”；文件本身完好，执行 `xattr -cr /Applications/Markion.app` 即可打开（或改用 `curl -LO` 下载）。可直接双击打开的公证版 DMG 需要维护者按 `docs/macos-signing.md` 配置 Apple 凭据后才会提供。
- Windows 安装包仍未签名；SmartScreen 可能要求“更多信息 → 仍要运行”。

## 下载

- Windows x64：NSIS 安装程序与便携 `.zip`。
- macOS Apple Silicon：DMG（`aarch64`）。
- macOS Intel：DMG（`x64`）。
- Linux x86_64：DEB、RPM 与 AppImage。

## 验证

- 本地 workspace 测试：除发布机器上已知的环境性失败外全部通过（macOS 符号链接临时目录与 APFS 目录身份语义导致 19 个 AI 日志/策略测试失败；markion-pdf 的两个字体测试一如既往地在本地失败）。本版本应用代码与 v0.4.11 一致，不受这些失败影响。打包的 MarkNice 工作区 bundle 校验成功。
- Windows、macOS（Apple Silicon 与交叉编译的 x86_64）及 Linux 原生 CI 构建与打包通过，包括无 GPU 的 AppImage 冒烟测试；更新器安装包已完成签名，并与版本匹配的 `update.json` 一同镜像分发。

**完整变更对比**: https://github.com/willmove/markion/compare/v0.4.11...v0.4.13
