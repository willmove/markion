# 测试素材、生成与字节比较

这些是仓库自有合成内容，供[62 组用例](../README.md)使用，不含真实笔记、账户或云凭据。`sources/` 是只读原件；先生成工作副本，不在此目录直接练习。

## 准备与核对

Python 3.10+、标准库；Git 2.28+ 用于本地双端同步素材。仓库根目录运行：

```powershell
python docs/testing/fixtures/prepare.py check
python docs/testing/fixtures/prepare.py prepare --output target/tmp/release-qa/run-001
python docs/testing/fixtures/prepare.py verify --run target/tmp/release-qa/run-001 --working
```

无 Git 可用时新 run 使用 `--without-git`，G 组记 BLOCKED。prepare 拒绝任何已存在的目标；中途失败也保留现场，不覆盖重试。verify 默认仅核对 baseline；`--working` 还核对编辑前的工作副本，编辑后不应再用它要求整个 workspace 未变。

输出包含：

| 路径 | 内容／用途 |
| --- | --- |
| `workspace/01-editing.md` | hello、中文、emoji、组合字符和格式边界；D/E |
| `workspace/02-structures.md` | 独立段落、嵌套列表＋代码、引用、五类callout、空标题；E04/E07/E11 |
| `workspace/03-tables.md` | 双表、空表头／连续空行、不齐行、折行、HTML合并表；R01–R04 |
| `workspace/04-rendering.md` | YAML、TOC、中文/重复锚点、脚注、定义列表、代码、公式、Mermaid、HTML；R05–R07 |
| `workspace/05-images.md` | 本地／远程／引用／重复／失效／HTML／约2.7MiB data URI；R08/X07 |
| `workspace/06-search.md` | literal needle：大小写敏感4处、不敏感6处；正则 needle-\d 两处；Read可见label不含隐藏URL；E13 |
| `workspace/07-exports.md` | 导出格式对照；X01/X02/X05 |
| `workspace/line-endings/` | LF／CRLF／无EOF换行，结构和表格额外字节变体；D09/E/R |
| `workspace/empty.md` | 真正0字节空文档；E11 |
| `workspace/images/` | 程序生成128×128棋盘PNG、SVG、有效但含9MiB辅助chunk的大PNG；图片阈值/离线/HiDPI观察 |
| `workspace/data/` | 纯文本、JSON、YAML、TOML；D10 |
| `workspace/tree/` | 深目录、空目录、120个中文长名文件、隐藏／排除目录；W |
| `workspace/clipboard.html` | 用实际浏览器打开后选择正文复制，包含格式／列表／表格／代码；E06 |
| `workspace/ai/` | a.md为Alpha 中文😀，b.md为Beta；作为单独工作区授权，适配已有mock的固定路径；A |
| `workspace/imports/` | 确定性synthetic.docx、语义正文预期、坏ZIP；X03/X04 |
| `baseline/` | workspace独立完整副本，测试期间不编辑 |
| `fixture-manifest.json` | 每个输入的相对路径、大小、SHA-256；不含机器绝对路径 |
| `git/` | 本地bare remote和local/peer两clone；与workspace分开 |
| `profile/` | Windows及XDG两套合成配置，关闭启动更新与自动保存 |
| `evidence/` | expected/、截图、日志等本轮产物；初始为空 |
| run根报告/CSV | 从模板复制，默认NOT_RUN，执行前展开变体 |

图片的体积由有效PNG辅助chunk构造，不等于大像素／重型解码基准；data URI也主要测试长源和对象交互。失效图片/外链是刻意素材。需要重型实际照片或近上限DOCX时提供合法样本并登记，不从这些小像素图推断任意图片内存表现。

## 资源服务与上传 stub

只在需要R08/X07/X08时另开终端，从仓库根运行以下本地HTTP服务；其URL用于明确的测试资源，不代表生产图床：

```powershell
python -m http.server 18115 --bind 127.0.0.1 --directory target/tmp/release-qa/run-001/workspace
```

X08自定义上传程序选当前 `python` 的绝对路径；参数逐行填写：工作副本 `uploader.py` 的绝对路径、可选 `--mode` 与 `fail`/`multi`/`slow`、最后完整 `{file}`。默认成功将输入复制为工作区 images/uploaded.png，输出单一loopback URL；multi输出两URL，fail返回3，slow等待10秒（测试超时设为5秒）。程序需访问本地测试服务才有可预览图片。只停止此终端服务，不影响其他进程。stub不证明PicGo/Core或云provider兼容，真实服务另建行。

## Git 双端操作

打开run/git/local作为Markion工作区；已有origin指向run/git/remote.git。peer使用临时仓库内的QA用户身份，不改全局Git配置；没有真实网络和凭据。外部编辑peer/note.md或other.md后执行（替换run绝对路径）：

```powershell
git -C 'C:/QA/run-001/git/peer' add note.md
git -C 'C:/QA/run-001/git/peer' commit -m QA-peer
git -C 'C:/QA/run-001/git/peer' push
```

同步后peer用 `git pull --ff-only`；冲突用例让两端同时修改 `BASE note` 同一行。删除/重命名分别用测试clone中的 `git rm note.md`／`git mv note.md renamed.md` 再提交推送。所有命令只针对run目录，禁止用日常repo替换路径。

## DOCX corpus 与独立预期

synthetic.docx由标准库生成，包含两段、bold片段和2×2表；synthetic.expected.txt是**语义行序**，不是导入Markdown逐字节oracle。它没有真实Word/WPS/LibreOffice producer身份，不覆盖修订、图片或完整OMML。真实20–30份样本使用[corpus模板](../templates/docx-corpus.csv)记录来源、版本、许可／脱敏步骤、预期结构与资源哈希；缺少时仍标未资格验证。可另用07-exports导出样本登记Markion/MarkNice往返，不冒充第三方producer。

字节比较（0相同、1不同、2读取或参数错误）：

```powershell
python docs/testing/fixtures/prepare.py compare --expected target/tmp/release-qa/run-001/baseline/01-editing.md --actual target/tmp/release-qa/run-001/workspace/01-editing.md
```

工具输出使用UTF-8。compare输出大小、SHA-256及首个差异偏移，不输出正文。不要用应用实际结果覆盖expected。只有渲染或导航的操作应与原baseline相同；编辑后用独立构造expected；一次Undo后再与baseline相同。Windows hidden属性分支可在工作副本 hidden-attribute.md 上设置/清除属性后操作，单纯dotfile不代表已测试Windows属性。
