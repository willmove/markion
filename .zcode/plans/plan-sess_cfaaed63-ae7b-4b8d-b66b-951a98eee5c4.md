## 完善国际化：将"中文"改为"中文简体"并新增"中文繁体"

遵循 AGENTS.md 的 OpenSpec 工作流：先建 change 提案，再按 tasks 逐项实现。改动局限于 `ui-i18n` 能力，不动 Markdown 缓存等不变量。

### 现状摘要（已勘探）
- `src/i18n.rs`：零依赖手写 i18n。`Language` 枚举为 `En/Zh/Ja/Fr/De/Es`，每个语言一个穷尽 `match` 表 `t/lang`。`Zh` 的 `native_name()` 现为 `"中文"`。
- `from_code` 已接受简体别名 `zh/chs/zh-cn/zh-hans/chinese`；持久化层 `src/storage/preferences.rs` 与 `src/model.rs` 只存原始字符串、对未知值兜底为英文，**无需修改**。
- UI 接入点：`src/app/root_view.rs:2204` 用 `Language::all()` + `native_name()` 渲染 Preferences 语言选择器；`src/app/search.rs:222` 的 `apply_language` 把新语言切到原生菜单；`src/app/mod.rs:169` 的 `dropdown_left` 为每语言手调像素，**当前是穷尽 match，新增枚举必须显式补 arm 否则编译失败**（这是安全网）。
- `src/app/mod.rs` 外部仅 6 处 `Language::Zh` 引用（全在 `dropdown_left`）。

### 关键设计决策（已与用户确认）
1. **枚举命名**：把 `Language::Zh` 重命名为 `Language::ZhHans`（语义更准、与 `ZhHant` 对称）。`from_code` 继续接受所有旧简体别名，**向后兼容**旧 prefs 文件。
2. **新增 `Language::ZhHant`**（繁体）。新代号采用标准 `zh-hant`，同时接受 `zh-tw`/`cht`/`zh-Hant`/`traditional chinese` 等常见别名。
3. **原生显示名**：简体条目 `"简体中文"`、繁体条目 `"繁體中文"`（每种用自身字符显示，符合 OS 惯例）。
4. **繁体术语**：采用台湾惯用风格（資料夾、偏好設定、預設、剪貼簿、結束 等非大陆用语）。
5. **`all()` 顺序**：`[En, ZhHans, ZhHant, De, Es, Fr, Ja]`——两个中文相邻。

### OpenSpec 提案阶段（Task 1）—— `/openspec:propose`
创建 `openspec/changes/expand-chinese-variants/`：
- `proposal.md`：Why（现有"中文"语义模糊、缺繁体用户）/ What Changes（重命名 `Zh→ZhHans`，新增 `ZhHant`，扩展全部 match 分发点，`dropdown_left` 繁体归入 CJK 字符组，更新测试）/ Non-goals（不动持久化格式、不翻译文档内容、不加 RTL 或 ICU）/ Impact（向后兼容旧 prefs，`zh`/`zh-cn` 等仍映射简体）。
- `specs/ui-i18n/spec.md` delta：
  - 修改"Common Chinese aliases"需求 → 增加繁体别名场景（`zh-hant`/`zh-tw`/`cht` → 繁体）。
  - 修改"exhaustive match"需求 → 把测试用例语言集从"English + Simplified Chinese"扩到含 Traditional Chinese。
- `tasks.md`：见下方任务拆分。

### 实现阶段（Task 2–6）—— `/openspec:apply`

**2. 枚举与元数据**（`src/i18n.rs:16-71`）
- `Language`：`Zh` → `ZhHans`，新增 `ZhHant`。
- `code()`：`ZhHans => "zh-hans"`、`ZhHant => "zh-hant"`。
- `from_code`：简体组 `zh/chs/zh-cn/zh-hans/chinese` → `ZhHans`；新增 `zh-hant/zh-tw/cht/zh-hk/traditional chinese` → `ZhHant`。
- `native_name()`：`ZhHans => "简体中文"`、`ZhHant => "繁體中文"`。
- `all()`：`[En, ZhHans, ZhHant, De, Es, Fr, Ja]`。

**3. 繁体翻译表 `fn zh_hant(msg)`**：复制 `zh()` 为模板，全部转繁体并改台湾用语。`PREFERENCES_DETAIL_ZH_HANT` + `SHORTCUTS_ZH_HANT` + `SHORTCUTS_ZH_HANT_EXTENDED` 三个常量。

**4. 分发点扩展**：
- `t()`：新增 `Language::ZhHant => zh_hant(msg)` arm。
- `shortcut_reference()`：新增 `(Language::ZhHant, false)` → `SHORTCUTS_ZH_HANT`、`(true)` → `SHORTCUTS_ZH_HANT_EXTENDED`。
- `sidebar_tab_label()`：新增繁体 `檔案/大綱` arm。
- `src/app/mod.rs:169 dropdown_left`：把 `Language::Zh` 重命名为 `ZhHans`，并把 `ZhHant` 并入 CJK 字符组（文件/編輯/檢視/格式/匯出/說明 与简体字宽接近，沿用同一组像素值）。

**5. 测试更新**（`src/i18n.rs` 测试模块 + `src/app/tests.rs`）：
- `language_round_trips` / `every_message_returns_non_empty_text_for_every_language`：`Language::all()` 自动覆盖新枚举；为 `ZhHant` 加一行 `assert!(!t(...).is_empty())`。
- `language_from_code_accepts_common_aliases`：新增 `zh-hant`/`zh-tw` → `ZhHant` 断言。
- 新增 `traditional_chinese_shortcut_reference_is_translated` 测试（参考现有 `chinese_shortcut_reference_is_translated`）。
- 把 `app/tests.rs:470 language: None,` 之类的字段声明对照检查（应为 enum/struct 字段，无影响，编译期验证）。

**6. 验证**：`cargo build` → `cargo test`（重点跑 i18n 模块测试 + 穷尽性守卫）。`openspec validate expand-chinese-variants`。

### 验证完成标准
- `cargo build` 无警告通过；`cargo test` 全绿。
- Preferences 面板出现 7 个语言选项，简体为 `简体中文`、繁体为 `繁體中文`，二者均可切换并立即翻译原生菜单。
- 旧 prefs 文件 `language = "zh"`（或 `zh-cn`/`chs`）仍加载为简体；新繁体用户 `language = "zh-hant"` 正常持久化。
- `dropdown_left` 对繁体菜单（檢視/格式/匯出 等）下拉面板对齐正确。

### 提交策略
按 task 粒度分提交（枚举/元数据、繁体翻译表、分发点、测试），不主动 push，完成后由用户决定是否 `/openspec:archive`。

### 待用户后续操作
实现完成后，是否归档（`/openspec:archive`）由用户决定——AGENTS.md 要求归档前 `openspec validate`。