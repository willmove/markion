# Pop!_OS 24.04 / COSMIC / Wayland 输入法定位验证环境调研

调研日期：2026-09-10。本文只引用操作系统、协议、输入法项目和服务商自己的资料。

## 结论

目前在所核对的服务商官方目录中，没有找到“开通即得、版本可确认、带持久化和可验证 GPU 的当前稳定版 Pop!_OS 24.04 + COSMIC + Wayland”云桌面。DistroSea 的 Pop!_OS 24.04 页面同时列出了 `Main`、`Cosmic Alpha 6` 和 `COSMIC`，但其 COSMIC 实例通过 noVNC 提供浏览器演示，页面没有承诺具体 COSMIC 构建、持久化或 Vulkan/GPU 能力；Shells 的预装系统列表没有 Pop!_OS，只有“上传自定义操作系统”的泛化能力。因此，不应把这两者直接当作发布验收机。[DistroSea 的 Pop!_OS 页面](https://distrosea.com/select/popos/)；[DistroSea 的 COSMIC noVNC 会话页](https://distrosea.com/start/popos-24.04-cosmic/)；[Shells 的系统列表和自定义系统说明](https://www.shells.com/os)

推荐顺序如下：

1. **让原缺陷报告者在原机验证补丁**。这是唯一能同时保持 Pop!_OS、COSMIC、Wayland、内核、输入法版本、缩放和显卡完全不变的方案，证据价值最高。
2. **在一台普通 x86_64 PC 上用官方 ISO 实机启动**。优先使用 Live USB 的 `Try Demo Mode` 做一次性验证；需要长期复测时，将系统完整安装到独立/外置 SSD。这是自己可控且最接近报告环境的方案。
3. **若必须远程租真机，先向 Hetzner 确认显示链路，再租 GEX45 GPU 裸金属**。它有实体 NVIDIA GPU、自定义 ISO 所需的 KVM，但成本高，而且下单前必须确认 KVM 能看到由该 GPU 驱动的 COSMIC 会话，而不是只有服务器管理显卡或无显示头。
4. **Vultr 自定义 ISO 云 VM 用作快速协议回归**。安装最方便、按小时便宜，浏览器 VNC 能直接操作 guest 控制台；但虚拟显卡/Vulkan 可能是软件实现，只能作为中等可信度证据。
5. **本地 VirtualBox VM 用作开发回归**。官方有明确安装步骤，成本最低，但虚拟 GPU 和虚拟显示栈与报告者不同。

不建议用 AWS DCV、普通 xrdp/Xvnc 会话、WSL、容器或 DistroSea 完成最终验收。它们会改用 X11/独立虚拟桌面、缺少 COSMIC compositor，或版本太旧。AWS 官方更明确要求 Linux DCV 禁用 Wayland，因此与本缺陷的目标路径相冲突。[Amazon DCV Linux 前置条件](https://docs.aws.amazon.com/dcv/latest/adminguide/setting-up-installing-linux-prereq.html)

## 什么才算可信的验证

Pop!_OS 24.04 默认包含 COSMIC，System76 将 COSMIC 定义为 Wayland-native 桌面；24.04 初始发布组件包括 COSMIC Epoch 1。[Pop!_OS 24.04 默认应用与组件](https://system76.com/support/default-apps)；[COSMIC 官方介绍](https://system76.com/cosmic)

本问题验证的关键不是“屏幕看起来像 Linux”，而是保持这条链路：

`Markion 原生 Wayland client -> text-input-v3 cursor rectangle -> cosmic-comp -> guest 内的 Fcitx5/IBus -> 候选窗`

`text-input-v3` 的 `set_cursor_rectangle` 本来就是让 client 以 surface-local 坐标告诉 compositor 光标矩形，以便将候选/建议窗口放在附近。[Wayland text-input-v3 协议说明](https://wayland.app/protocols/xx-text-input-v3) Fcitx 官方也说明：原生 Wayland 的 winit 应用通常走 `text-input-v3`，候选窗的正确放置依赖 compositor/input-method 协议；Wayland 没有供普通 client 使用的全局坐标。[Fcitx 5 on Wayland](https://fcitx-im.org/wiki/Using_Fcitx_5_on_Wayland/en)

因此：

- **最高可信**：用户坐在实机前，用该机键盘和显示器操作。
- **可接受的远程真机**：硬件 KVM/IP-KVM 只负责传送 USB 键盘事件和捕获真实显示输出，COSMIC、输入法和候选窗均在远端主机内部运行。
- **有条件可接受的 VM**：guest 内确实是 COSMIC Wayland，Markion 是原生 Wayland，输入法也在 guest 内组合；VM 证据可以确认坐标协议修复，但不能代替显卡/多屏/缩放实机验收。
- **不可信**：xrdp/Xvnc/Amazon DCV 创建另一个 Xorg 会话；客户端本地输入法已经把汉字提交给远端；Markion 走 XWayland；只看到浏览器模拟的旧版桌面。

COSMIC 官方调试文档给出了原生 Wayland 判定方法：运行 `xprop` 后若无法选中目标窗口，该窗口就是原生 Wayland；也可用 `WAYLAND_DEBUG=1` 检查协议调用。[COSMIC 调试文档](https://github.com/pop-os/cosmic-epoch/blob/master/docs/DEBUGGING.md)

Markion 使用 GPUI，因此图形能力还必须单独过关。作为同一 GPUI 技术栈的官方参考，Zed Linux 要求 Vulkan 1.3，并指出没有兼容 GPU 时窗口可能无法创建。[Zed 安装要求](https://zed.dev/docs/installation)；[Zed Linux 图形故障排查](https://zed.dev/docs/linux)

## 方案对比

| 方案 | 是真机吗 | 当前 Pop/COSMIC 可控 | Wayland/IME 定位可信度 | Vulkan/GPUI | 成本与限制 | 用途 |
| --- | --- | --- | --- | --- | --- | --- |
| 原报告者原机复验 | 是 | 完全一致 | 最高 | 已知能运行 | 通常仅沟通成本；开发者不一定能直接控制 | **首选最终验收** |
| 自有/借用 PC + Live USB 或外置 SSD | 是 | 高 | 最高 | 使用真实 Intel/AMD/NVIDIA GPU | 系统免费；需 U 盘/SSD 和一台可重启 PC | **首选自助最终验收** |
| Hetzner GEX45 GPU 裸金属 + KVM | 是 | 高 | KVM 显示链路确认后高 | NVIDIA RTX PRO 4000 | €214/月 + €209 一次性设置费，未含 VAT；远高于一次缺陷验证所需 | 需要远程实体 GPU 时 |
| Vultr Cloud Compute + 自定义 ISO + Web VNC | 否 | 高 | 中 | 必须实测；可能只有虚拟/软件 Vulkan | 2026-09-10 官方 Plans API 中 4 vCPU/8 GB 为 $0.055/小时、$40/月；关机仍计费，销毁才停止 | **最快的远程回归机** |
| 腾讯云 GPU CVM + 导入自定义镜像 | 否 | 高，但需先制作磁盘镜像 | 低到中 | GPU 可用，驱动和图形会话需自配 | 支持按量计费；需另付镜像快照、磁盘和流量成本 | 中国大陆网络低延迟备选 |
| 本地 VirtualBox | 否 | 高 | 中 | 必须实测虚拟 GPU/Vulkan | VirtualBox/Pop!_OS 免费；消耗本机 RAM/磁盘 | 日常开发回归 |
| Shells 自定义系统云桌面 | 否 | 需售前确认 | 低到中 | 官方页未承诺 Vulkan/GPU | 4 GB Plus 页面价约 $22.95/月；首购/续费 7 日内可申请按条款退款 | 只有确认 ISO、Wayland、Vulkan 后才试 |
| Google Cloud GPU workstation + 导入磁盘 | 否 | 复杂 | 低 | L4 等显示型 GPU | `g2-standard-4` 官方价示例约 $0.706832276/小时，另有磁盘/网络/远程软件成本 | 不经济的备选 |
| DistroSea Pop!_OS 24.04 COSMIC | 否 | 低（具体构建未注明） | 低 | 不可控 | 免费、排队、noVNC 浏览器会话 | 快速体验/冒烟，不作最终验收 |
| AWS EC2 + Amazon DCV | 否 | 复杂 | **不合格** | GPU 可用但 DCV 是 X session | DCV 官方要求禁用 Wayland | 排除 |

Vultr 的数字来自其[公开 Plans API](https://api.vultr.com/v2/plans)在调研日的返回；实例按小时收费，Cloud Compute 按 672 小时封顶，停止实例并不停止收费。[Vultr 计费规则](https://docs.vultr.com/support/platform/billing/how-am-i-billed-for-my-servers) Shells 的页面价格会随月付/年付选项变化，购买前应在订单页确认；其条款称首次购买或续费七日内可请求当期按月费率的比例退款。[Shells 定价页](https://www.shells.com/l/en-US/lp/cloud)；[Shells 条款](https://www.shells.com/page/terms)

## 方案实施

### A. 原报告者原机复验

给报告者一个可校验的 Linux AppImage/DEB、提交 SHA 和以下最短操作说明；不要要求其更换输入法或更新系统后再测：

1. 保存环境信息和 `fcitx5-diagnose`（见后文命令）。
2. 运行修复前版本录一段对照视频。
3. 在同一登录会话、相同窗口大小和缩放下运行修复版。
4. 分别关闭/打开打字机模式，连续键入 `n -> ni -> nih -> nihao`。
5. 录下候选框、光标和窗口左上角，并返回结果及环境文件。

这能直接回答“补丁是否修复报告者机器”，优先级高于另找一台相似机器。

### B. 自有或借用 PC：Live USB / 外置 SSD

System76 官方支持把 ISO 写入 U 盘，在安装器中选择 `Try Demo Mode` 进入完整 live 环境；官方也明确将 live disk 用于判断问题是否由硬件或软件引起。[安装指南](https://support.system76.com/support/install-pop/)；[Live Disk 指南](https://support.system76.com/articles/pop-live-disk)

1. 从[官方 Pop!_OS 下载页](https://system76.com/pop/download/)下载 24.04 x86_64 ISO；按机器选择通用版或 NVIDIA 版，并校验页面给出的 checksum。
2. 按官方指南写入 U 盘。Pop!_OS 要求关闭 Secure Boot；最低 4 GB RAM、20 GB 磁盘，推荐 8 GB RAM。[安装要求](https://support.system76.com/support/install-pop/)
3. 只做一次验证时选 `Try Demo Mode`。要长期复测时，安装到一块独立/外置 SSD；不要把 `Clean Install` 指向 Windows 系统盘。
4. 进入 COSMIC 后更新软件；若目标是逐字复现原报告，则先记录 ISO 自带版本，再决定是否更新到报告者的内核和 COSMIC/Fcitx 包版本。
5. 安装与报告者相同的输入法。Fcitx5 拼音所需包通常为 `fcitx5`、`fcitx5-chinese-addons`、`fcitx5-config-qt`；Fcitx 官方说明 `fcitx5-chinese-addons` 包含简体中文拼音，配置可用 `fcitx5-configtool`。[Fcitx 输入法引擎](https://fcitx-im.org/wiki/Input_method_engines/en)；[Fcitx 配置工具](https://fcitx-im.org/wiki/Configtool_%28Fcitx_5%29)
6. 注销并重新登录，再执行统一检查和用例。

注意：Pop!_OS 24.04 基于 Ubuntu Noble 仓库，Noble 提供 Fcitx5 和中文附加组件，但 clean install 上的实际版本、Pop 仓库更新和原报告者配置可能不同，必须用 `apt-cache policy` 记录，而不是只写“Fcitx5”。[Ubuntu Noble Fcitx5 包](https://packages.ubuntu.com/search?arch=any&keywords=fcitx&searchon=names&suite=noble)

### C. Vultr 自定义 ISO 云 VM

Vultr 官方支持从公开直链上传小于 10 GB 的 `.iso`（每账户最多两个），用 ISO 新建实例，通过 Web VNC 完成安装，随后卸载 ISO。[Vultr 自定义 ISO 指南](https://docs.vultr.com/how-to-upload-and-use-custom-isos-on-vultr)

1. 取得官方 Pop!_OS ISO 的最终公开直链。若官方地址包含重定向或查询参数，先放到自己控制的对象存储公开直链，并核对 SHA-256；不要使用来历不明的重打包镜像。
2. Vultr Console：`Orchestration -> ISOs -> Add ISO`，等待状态为 Complete。
3. 新建 Cloud Compute，建议至少 4 vCPU / 8 GB / 80 GB；Image 选择 Custom ISO。
4. 打开 `View Console`，安装 Pop!_OS，之后 `Settings -> Custom ISO -> Remove ISO`。
5. 在 guest 内安装输入法和 Markion。浏览器/本机输入法切回英文键盘，让 **guest 内的 Fcitx5/IBus** 接收 `n/i/h/a/o` 原始按键。
6. 只有在 `XDG_SESSION_TYPE=wayland`、Markion 原生 Wayland、候选框确由 guest 输入法绘制、`vulkaninfo` 通过时，才保留为中等可信度证据。
7. 完成后销毁实例；仅关机仍收费。

若 `vulkaninfo` 只显示 llvmpipe/lavapipe，仍可做坐标协议回归，但必须在结果中标为“软件 GPU VM”，不能替代真机。Vultr 另有 Cloud GPU，但官方文档只分别证明“GPU 实例存在”和“Cloud Compute 可用自定义 ISO”，没有证明两者能组合；购买前需向售前确认。[Vultr Cloud GPU 配置](https://docs.vultr.com/products/compute/instances/cloud-gpu/provisioning)

### D. Hetzner GPU 裸金属

Hetzner GEX45 是带 NVIDIA RTX PRO 4000 Blackwell SFF 的独立物理服务器；2026-09-01 官方价格为 €214/月、一次性设置费 €209（未含 VAT）。[GEX45 发布与价格](https://www.hetzner.com/pressroom/hetzner-expands-its-gpu-portfolio-with-the-gex45/) Hetzner 可为任意 dedicated root server 接 KVM，提供键盘、视频、鼠标和虚拟介质，自定义 ISO 安装；KVM 前三小时免费。[Hetzner KVM Console](https://docs.hetzner.com/robot/dedicated-server/maintenance/kvm-console/)

下单前向销售/支持书面确认三点：

- KVM 捕获的是能显示 COSMIC 会话的实际输出，且可提供有效 display head/EDID；
- Pop!_OS 24.04 NVIDIA ISO 可从 KVM 虚拟介质或技术员制作的 USB 启动；
- 安装后可继续用 KVM 操作桌面，而不是只能用 SSH。

确认后：订购 GEX45，预约 KVM，将 ISO 通过 KVM 支持的 SMB/CIFS 虚拟介质挂载（或请技术员写入 USB），安装到 NVMe，安装输入法和 Markion，并全程用 KVM 操作 guest 内输入法。若 KVM 只能看到管理显卡/文本控制台，就取消此路线；“有计算 GPU”不等于“有可远程捕获的 Wayland 桌面”。

Vultr 也支持 bare metal 自定义 ISO，但其普通 bare metal 计划没有被官方承诺为 Vulkan 图形工作站；GPU bare metal 对单次 UI 缺陷验证价格过高，故不优先。[Vultr bare metal 自定义 ISO](https://docs.vultr.com/custom-iso-on-bare-metal)

### E. 本地 VirtualBox

System76 官方 VirtualBox 指南要求选择 `Linux / Ubuntu (64 bit)`、至少 4096 MB RAM、至少 15 GB（建议 20 GB）磁盘，并启用 EFI；默认 legacy boot 可能导致安装失败。[System76 VirtualBox 指南](https://support.system76.com/support/install-in-vm)

实际建议分配 4 CPU、8 GB RAM、40 GB 动态磁盘，启用 EFI 和可用的 3D 加速。安装后先执行 `vulkaninfo --summary` 和 Markion 启动测试。若只能软件渲染，仍适合日常回归候选框坐标；最终结论仍需实机。

### F. 其他云桌面/GPU 路线为何降级

- **Shells**：官方预装列表没有 Pop!_OS，但宣称可上传自定义系统，适合先向支持询问“Pop!_OS ISO、UEFI、COSMIC Wayland、Vulkan 1.3、guest 内输入法”五项。任何一项无法确认就不要购买。[Shells 系统页](https://www.shells.com/os)
- **Google Cloud**：可以导入 VMDK/VHD 等虚拟磁盘并建立自定义 image，也有 L4/T4/P4/P100 显示型 GPU workstation；但官方教程基于 Ubuntu 22.04 + HP Anyware，不保证 Pop!_OS/COSMIC Wayland 会话。它不是直接上传 ISO，准备工作和成本都明显高于 Vultr。[导入虚拟磁盘](https://docs.cloud.google.com/compute/docs/import/importing-virtual-disks)；[GPU Linux workstation](https://docs.cloud.google.com/compute/docs/virtual-workstation/linux-gpu)；[G2 官方价格](https://cloud.google.com/products/compute/pricing/accelerator-optimized)
- **腾讯云 GPU CVM**：GPU 实例支持公共、自定义、共享和镜像市场四种镜像，也支持按量计费；可把本地系统盘以 RAW、VHD、QCOW2 或 VMDK 导入为自定义镜像。可先在本地 VM 安装 Pop!_OS 24.04 后转换并导入，选择 GPU 实例时自行安装驱动。它对中国大陆用户的网络延迟通常更友好，但官方没有承诺 COSMIC Wayland 的交互式远程桌面链路；是否能创建真实 GPU-backed display、控制台能否捕获候选窗，仍需实测，所以只列为低优先级备选。[腾讯云 GPU 实例快速入门](https://cloud.tencent.com/document/product/560/30211)；[导入自定义镜像](https://cloud.tencent.com/document/product/213/4945)
- **阿里云 ECS/EGS**：同样能导入 RAW、QCOW2、VHD 或 VMDK 自定义 Linux 镜像并用于 GPU 实例；ISO 不能直接导入，需先安装到虚拟磁盘并配置 cloud-init/virtio，再转换上传到 OSS。准备成本和远程显示不确定性与腾讯云相近。[阿里云导入自定义镜像](https://www.alibabacloud.com/help/en/ecs/user-guide/import-a-custom-image)；[GPU 实例概览](https://www.alibabacloud.com/help/en/egs/quick-reference)
- **AWS**：VM Import 支持 OVA/VMDK/VHD/raw，但官方支持 OS 表没有 Pop!_OS；更关键的是 Amazon DCV 的 Linux 图形会话基于 X server，并要求 GDM 禁用 Wayland，不能验本缺陷。[AWS VM Import 格式和支持 OS](https://docs.aws.amazon.com/vm-import/latest/userguide/prerequisites.html)；[Amazon DCV 前置条件](https://docs.aws.amazon.com/dcv/latest/adminguide/setting-up-installing-linux-prereq.html)
- **DistroSea**：服务页当前既列出 Pop!_OS 24.04 `COSMIC`，也保留 `Cosmic Alpha 6`；但 COSMIC 会话由 noVNC 提供，页面没有标明精确构建，也未承诺 GPU、持久化、输入法包或自定义 Markion 构建能力。因此它适合免费冒烟，不适合作为发布验收依据。[DistroSea Pop!_OS](https://distrosea.com/select/popos/)；[COSMIC 会话页](https://distrosea.com/start/popos-24.04-cosmic/)

## 统一环境采集

每台候选机器都先保存以下输出：

```bash
mkdir -p ~/markion-ime-evidence
{
  date --iso-8601=seconds
  cat /etc/os-release
  uname -a
  printf 'XDG_SESSION_TYPE=%s\n' "$XDG_SESSION_TYPE"
  printf 'XDG_CURRENT_DESKTOP=%s\n' "$XDG_CURRENT_DESKTOP"
  printf 'WAYLAND_DISPLAY=%s\n' "$WAYLAND_DISPLAY"
  loginctl show-session "$XDG_SESSION_ID" -p Type -p Desktop -p Remote 2>&1
  apt-cache policy cosmic-comp cosmic-session fcitx5 ibus 2>&1
  vulkaninfo --summary 2>&1
} | tee ~/markion-ime-evidence/environment.txt

fcitx5-diagnose > ~/markion-ime-evidence/fcitx5-diagnose.txt 2>&1 || true
```

然后验证 Markion 是否原生 Wayland：

```bash
xprop
```

点击 Markion 窗口；若 `xprop` 能读取窗口属性，则它是 X11/XWayland，本轮不算目标路径。必要时从终端用协议日志启动：

```bash
WAYLAND_DEBUG=1 ./markion 2>~/markion-ime-evidence/wayland.log
```

出现 `zwp_text_input_v3`/`set_cursor_rectangle` 相关调用可作为补充证据。Fcitx 官方建议使用 `fcitx5-diagnose` 收集发行版、桌面和 Wayland 连接状态。[fcitx5-diagnose 文档](https://fcitx-im.org/wiki/Fcitx5-diagnose/en)

## 验收用例

在同一台机器、同一输入法配置下，对修复前和修复后构建各执行一次：

1. 新建空白文档，窗口非最大化，光标分别放在左上、中间、右下附近。
2. **关闭打字机模式**，切到中文输入法，缓慢键入 `n`、`ni`、`nih`、`nihao`，每一步停一秒。
3. 候选框应锚定在预编辑起点附近，不随拼音字符串变长而持续左右漂移。
4. **打开打字机模式**，重复同一序列；候选框不得跳到窗口左上角，应位于居中的当前编辑行光标附近。
5. 输入多行并滚动，让打字机模式触发垂直重定位，再重复步骤 2–4。
6. 移动和缩放窗口后重复；实机再覆盖 100% 和一个分数缩放、多显示器/不同显示器（若有）。
7. 分别使用鼠标点击定位、方向键移动、换行后触发输入法。
8. 记录输入法名称/版本、Markion commit、是否 AppImage/DEB/源码、缩放、显示器布局，以及远程方式。

通过标准：两种打字机模式下均无左上角回退；`n -> ni -> nih -> nihao` 期间候选框锚点不因预编辑长度水平漂移；提交候选后文本和光标位置正确。最终至少要有一份“原报告者原机”或“本地键鼠操作的 Pop!_OS 实机”视频和环境采集文件；VM 结果只能补充，不能单独关闭缺陷。
