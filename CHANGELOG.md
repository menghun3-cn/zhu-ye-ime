# 变更记录

本项目版本号遵循语义化版本，变更记录遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 中文格式。
当前唯一版本主源为根 `Cargo.toml` 的 `workspace.package.version`。

## [Unreleased]

### 变更

- 命名规范（T-108/T-109）：便携 zip 与全部产品目录去除 `ai-` 前缀——发布 zip
  `zhu-ye-ime-<v>.zip`、安装目录 `Program Files\zhu-ye-ime`、数据目录 `%APPDATA%\zhu-ye-ime`、
  日志目录 `%LOCALAPPDATA%\zhu-ye-ime\logs`、TSF 安装目录 `zhu-ye-ime\tsf`、便携包内顶层目录
  `zhu-ye-ime-<v>-test/`；exe 文件名（`ai-zhu-ye-ime-setup-<v>.exe`）与产品名不变。**升级不兼容**：
  v0.1.2 安装用户需卸载重装，旧 `ai-zhu-ye-ime` 目录不迁移不保留；v0.1.2 release 旧名 zip 资产
  已删除（同日补发新名资产）

## [0.1.2] - 2026-10-05

### 新增

- 候选覆盖增强（M14，FR-059）：低频音节前缀组词展开补足首屏——输入 `shui` 时「睡觉/水/谁」等前缀词直接入候选；仅当候选不足一页时介入，前缀组以词频序独立追加、不改变既有候选相对顺序；eval 基准与 host-e2e 全量回归零回退（T-090，PR #59）
- 产品化日志目录（M14，FR-060）：分级文件日志落盘 `%LOCALAPPDATA%\ai-zhu-ye-ime\logs\ime.log`（默认 warn 级、热路径零 IO，1 MiB 写前轮转），`config.json` 新增 `log_level` 字段可调（宽松解析、不升配置版本）；TSF 53 调用点分级（8 error/7 warn/14 info/24 debug）+ 11 热路径守卫；调试哨兵双轨保留旧路径全量日志（`C:\zhu-ye-test\tsf-debug.enable` 存在时原样写 tsf-debug.log）（T-091，PR #60）
- 官网页面对齐 v0.1.2-alpha（T-092/T-093）：版本状态行统一 M1–M14 口径（S-17，六页 footer + agents.txt）、hero-note 与 llms.txt 版本号同步，AI 入口四件套口径一致（S-18）；todos-done 里程碑归档补全（G-016~G-078）
- 发布分发闭环（P-01 第十期）：更新器内置 ed25519 发布公钥，可分发的词典领域包经签名 manifest 通过 GitHub Releases 分发（it/med/slang，默认关闭、开启后验签下载原子替换）；新增发布密钥生成（keygen）、发布资产组装（assemble-release）与端到端验证（verify-release-e2e）工具链，发布操作手册见 `docs/发布流程.md`（T-094）
- 正式发布打包 CI 化（T-095）：正式发布密钥对配置 GitHub Actions（私钥入 secret `ZHU_YE_RELEASE_SECRET_KEY`、公钥入 variable `ZHU_YE_RELEASE_PUBLIC_KEY`，测试密钥作废）；`release.yml` 工作流在推送 `v*` tag 时自动全链执行（数据源 pins 锁定校验 → CC-CEDICT 解压 → 词典全量构建 en.zyen/base/it/med/slang → 公钥注入构建 + 私钥签名 manifest + 复核 → zip/SHA256SUMS → 端到端验证 4/4 → 上传 GitHub Release），`workflow_dispatch` 手动试跑默认不上传；CI 试跑全链通过（run 37242703873）
- emoji 彩色渲染（批三，FR-009）：候选窗 emoji 经 DirectWrite 分层字符着色渲染替代底色块占位（T-101，PR #83）
- 通讯录检索扩展（批三，FR-037）：联系人拼音检索支持分批替换与十进制数字键小节（T-102，PR #86）
- 简拼/模糊音开关（批三，FR-023/FR-024）：`config.json` 新增 `enable_abbreviation` / `enable_fuzzy`（缺省 `true`，宽松解析、不升配置版本），关闭后对应候选路径不介入（T-103，PR #90）
- v 模式单位换算全量表（批三，FR-028）：v 模式内置 9 类 23 键单位换算表（长度/面积/体积/质量/时间/温度/速度/数据量/角度，SI 精确 + 市制通认折算）（T-104，PR #92）
- 口语语料替换（批三，场景 5）：31 个高频日常前词整词联想替换为口语后继表（天气/今天/你/我/工作/心情等），未覆盖前词零漂移（T-105，PR #94）
- 更新源镜像分发（FR-062）：默认更新 URL 切换镜像仓库 `menghun3-cn/zhu-ye-updates` 的 feed release（latest 语义可用，绕开主仓 prerelease 拒绝缺陷），release 工作流新增 `publish-updates` 步骤发布 feed 资产（T-096/T-098）
- exe 安装包（FR-061）：Inno Setup 图形化安装包（`ai-zhu-ye-ime-setup-<v>.exe`）随 release 第 7 项资产提供，安装/卸载复用发行包脚本语义（版本化 DLL、TSF 注册、预置领域包、卸载保留 %APPDATA%）；载荷与便携 zip 同源 staging（T-097）
- 官网 Pages 正式上线与下载直链（FR-063）：官网六页真实部署于 GitHub Pages，下载页提供 exe/zip 直链（T-099）
- VM 安装/覆盖/卸载 e2e 工具（T-107）：`scripts/vm-side-install-e2e.ps1`（四阶段断言）+ `scripts/vm-install-e2e.ps1`（本机驱动）

### 变更

- 正式定版与状态口径：v0.1.2 去除 alpha 预发布标记（`releases/latest` 语义指向正式版），官网六页状态行/llms.txt/agents.txt 同步为正式版（T-099）

## [0.1.1-alpha] - 2026-10-04

### 新增

- 英文词典全量扩容（M13，FR-046）：英文词表从 1.5 万词扩展为 ECDICT 全量离线词表——ZYEN v1 独立 mmap 词表 `en.zyen`（760,987 词条 / 18.56 MiB，加载+校验 28ms、前缀查询中位 43µs，均低于预算），拼写/大小写原形补全覆盖大幅提升；TSF 装配优先从 DLL 同目录加载、失败静态回退（T-085）
- 中英混合整句解码（M13，FR-050）：输入 `python代码` 这类无分隔中英混合串按整句出候选（整句置首 + 分段候选）；纯拼音、缩写与邮箱/网址格式路径不受影响（T-086）
- 网络语词典扩充（M13，FR-047）：social-media-chinese-words 高频子集 1 万清洗并入 slang 领域包（9,900 词条，含人工种子表扩充 19 条；来源与许可证见 `docs/licenses.md` D-020/D-021）（T-087）
- 设置窗口五项增强（M13，FR-048）：用户词表导入导出、符号集扩充至 230 字符、自定义主题文件（深浅色 + 候选窗同源装配）、启动时检查更新（默认关闭、零网络请求）、通讯录 vcf 界面化导入（T-088）
- 竹叶输入法官网上线（第十期，FR-051~FR-058）：`site/` 零构建静态官网（首页/文档/下载/隐私/关于 + 404），手写 SVG 品牌资产与候选窗示例插图，AI 入口四件套（llms.txt 按 llmstxt.org v2、agents.txt、robots.txt、sitemap.xml），GitHub Pages 工作流与内链自检就绪；下载页以安装包为主、全部指向 GitHub Releases（T-083、T-084）
- 简拼/首字母输入：`nh`→你好、`wsm`→为什么；静态 22 字母简拼音节表 + 前序笛卡尔积展开，上限 32 条，仅主候选为空且不可切分时介入（T-053，FR-023）
- 模糊音与纠错：zh↔z 等 7 组映射、单处替换；`zongguo`→中国（模糊替换）、`niha`→你好（少字母补全），独立 Corrected 组追加主候选之后（T-054，FR-024）
- 整句/长句 Beam Search：`woxiangmingtianqubeijing`→我想明天去北京；跨音节整词匹配 + 词间转移评分（unigram 上限、bigram 缺失惩罚），整句组置主候选最前（T-055，FR-025）
- host-e2e 新增 `--m7` 输入体验优化断言组（真实词典 22 项，含确定性）与 `scripts/e2e.ps1` 集成（T-055）
- `zhu-ye-cli bench` 新增 M7 三路径延迟场景（简拼/纠错/整句，回填验收标准 8.5）（T-055）
- 多音缺读补丁管线：`shui`→谁、`shou`→熟；构建期只增不改补读音词条（kTGHZ2013 规范读音对照 + 人工把关补丁表 + `import --polyphone` + `audit-polyphone` 审计命令）；修复 `dict -r` 中文键反查越界 panic（T-056）
- host-e2e `--m7` 增 4 项多音/反查断言（26/26）（T-056）
- 命中率评测基础设施：`zhu-ye-dict eval-set` 生成词样本（CEDICT∩wordfreq，2000 条）+ `zhu-ye-cli eval` 判定 Top1/Top3/整句并按词频分档、MISS 清单；评测集入库 `data/eval/`，real.zyct 首轮基准 Top1 84.7% / Top3 97.2% / 整句首候选 21.0%（两次运行一致）（T-057）
- 上下文联想检索层：`BigramModel::successors` 前词后继检索（bigram 表按前词连续、下界二分+区段扫描，零格式改动）+ `suggestion_candidates` 联想候选（Top5 整词 + 至多 3 条"前词+后继"两词短语）；`zhu-ye-cli suggest` 抓手（T-058）
- 上下文联想引擎承接：上屏后空闲候选窗展示 bigram 后继联想（连续联想）；联想态数字键直接上屏、空格选词、Esc 关闭、输入字母即退出回主输入路径；联想候选标注 Suggestion 来源；host-e2e `--m8` 真实词典断言组 7/7（T-059）
- 场景7 格式候选：任意连续数字启发式识别（空闲态数字键直插上屏、空格/数字选择、文档侧替换链换入格式文本；8 位日期 4 式/6 位年月/4 位年份/金额千分位+中文读数/11 位电话分段/≥5 位千分位）＋ v 模式符号组（空闲态 `v` 冷启动、v1 序号/vx 数学/vh 标点各一页 9 项、`vi` 回退拼音组合）＋ 拼音整串命中 emoji 别名队尾追加（不参与排序）；候选源优先级 联想 > 数字 > v；host-e2e `--m9` 真实词典断言组 15/15，T-057 命中率基准不回退（T-061）
- emoji 别名表纯数据扩展：首批 109 条扩至 381 条（动物/食物/物品/天气/交通/运动/手势/符号等常用类别，字母序二分保持，规模断言 ≥300；顺带修正 `biye` 别名 emoji（👋→🎓）并新增 `soup`/`glasses` 等独立别名）（T-062）
- 领域自动提权（M10，FR-033~035）：领域包完整词命中时按包内词频插入基础候选之后、追加组之前（仅完整词提权、多领域按包 id 字典序取首个、提权可配置开关、未启用领域包不加载不占内存）；host-e2e `--m11` 断言组 7/7（T-070）
- 通讯录候选（M11，FR-036~038）：vCard(.vcf) 文件导入 + 内置规范注音表（kTGHZ2013 8105 字）+ 通讯录索引（全读形 + 简拼派生键）；`zhangsan`/`zs` 均命中，联系人候选与领域提权同位次（D-21）；host-e2e `--m12` 断言组（内存 6/6 + 真实 vcf 7/7）（T-071）
- 设置窗口（第八期，FR-039~045）：原生 Win32 零 GUI 框架 crate `zhu-ye-settings`——三页导航（常用设置/工具箱/关于）、深色主题跟随系统（候选窗仍固定浅色）、单实例、宿主内配置读写（T-073）
- 工具箱页：emoji 面板（342 个唯一字符/23 组/6 页）+ 符号大全面板 + 图片占位；WS_EX_NOACTIVATE 不夺焦点、SendInput UNICODE 投递 + UIPI 回退剪贴板（T-074）
- 管理输入法页：注册状态诊断（RegistryProbe 判据与 `ime-identity.ps1` 一致）与两级修复——一级重建 packs 目录、损坏文件改名 `.bak`（无需提权、不删用户文件），二级经提权子命令重注册两棵 HKLM 树并重启 ctfmon（重启前弹确认）（T-076）
- 关于与更新页：「启用在线更新」运行时开关（默认关闭、关闭时检查按钮禁用且零出站连接）+ 检查更新子视图（spawn `zhu-ye-updater` 子进程执行，设置窗口自身不发起网络请求，输出逐行如实展示）+ 版本与诊断信息子视图（全部本地读取：core 版本/配置路径/数据目录/日志目录/已装包列表）；应用更新需二次确认（T-077）
- 发行安装改造（FR-045）：`install.ps1` 从发行包自身取文件（`-PackageRoot`），版本化 DLL + 基础词典 + `zhu-ye-settings.exe`/`zhu-ye-updater.exe` 至 Program Files + 预置 it/med/slang 三领域包 + 开始菜单快捷方式；`uninstall.ps1` 清理但不删除 `%APPDATA%` 用户数据；便携包同布局（T-078）
- core 英文词候选表 EN_WORDS（FR-030 底座，场景 6）：FrequencyWords 英文词频（D-018，CC BY-SA 4.0）前 10000 词 + 人工大小写补丁表（D-019，`data/patches/en-capitals.tsv`，专名/缩写原形如 `iPhone`/`API`/`QQ`，命令两可词保留小写）+ 人工排除清单（`data/patches/en-exclude.tsv`，中文人名音译噪声词，防 `zh` 等拼音声母前缀被英文组污染）+ CC-CEDICT 英文侧纯单词补充（D-001），共 15534 条；小写 ASCII 查键有序二分 + 前缀区段扫描 `en_words_with_prefix`，按 freq_rank 组内排序、上限截断；生成脚本 `scripts/build-en-words.ps1` 可复现并 rustfmt（T-064）
- 中英混输引擎（场景 6，FR-030/FR-031）：整串不可按拼音切分时才查英文表，命中以 `EnWord` 来源组追加主候选之后（D-10：可切分串 `nihao`/`wo` 不介入、`pytho`→python、`iphon`→iPhone 保留原形、`api`→API；未命中回落缩写路径 `yyds` 不回退；候选 `pinyin=None` 不进用户词学习）；邮箱/网址格式态（core `email_url` 模块）：组合串含 `@`（前有字符）→ `.com/.cn/.net` 补全至多 3 条，`www.`/`http(s)://` 前缀（大小写不敏感）→ `.com/.cn/.org` 补全至多 3 条，已含 `.` 完整串直通；`handle_format_char` 组合态收 `@ . / :`、空闲态放行宿主（TSF 键路归 T-066）（T-065）
- 中英混输 TSF 键路与 host-e2e `--m10`（场景 6，FR-030/031/032）：TSF `KeyAction::FormatChar` 统一格式字符动作（Shift+2=`@`、Shift+`;`=`:`、VK_OEM_2=`/`、`.` 复用 VK_OEM_PERIOD），按引擎 `is_format_key` 吃键——组合态邮箱/网址上下文进组合串、`www`/`http`/`https` 网址意图演进（含 `http:`/`http:/` 中间态）吃 `:`/`/`、普通拼音组合与空闲态/英文模式放行宿主（`nihao.` 标点直出不回归）；`me@`（`@` 尾空）补全候选（组合态按 `@` 后第一帧出 `.com/.cn/.net`）；host-e2e `--m10` 断言组：拼写补全 `pytho`→python、大小写原形 `iphon`→iPhone、拼音不介入、缩写不回退、邮箱/网址补全与直通、选中上屏、Esc/退格退出、噪声串（12/12 PASS，与 seed/real-smoke/m7/m8/m9 全量不回退）（T-066）
- 领域词典包体系（M6，FR-015~019/FR-021）：基础包 base.zyct（约 28.7 万词条/23.1MB，出候选率 100%）＋ IT 术语包 it.zyct（13,144 词条，MDN 正文源）＋ 医学包 med.zyct ＋ 网络语包 slang.zyct（种子表 + 正负样例把关）；多包运行时复合加载与缩写路径（CompositeDictionary 去重、词频取 max、坏包跳过；`u1s1`/`996` 等数字缩写可输入）；数据源 pins 机器可读锁定 + 获取脚本 SHA-256 校验（T-041/T-044/T-045/T-048/T-049/T-050）
- 词典在线更新（M6-U，FR-020/FR-022）：ed25519 签名 manifest、原子替换 + `.bak` 回滚、`zhu-ye-updater` 唯一联网组件（默认关闭时零网络）、客户端内置公钥校验拒绝未签名更新（T-051）
- 候选窗序号样式与搜狗经典风色板：纯数字序号、选中行默认首项高亮（块状圆角）；候选窗固定浅色主题（深色系统下不回退）；蓝框蓝字红选中配色；四角黑点修复与序号紧凑、英文译文靠左；选中块贴面板左右边框（T-028/T-030/T-032/T-037/T-043）
- 候选窗交互增强：无候选但组合非空时保留页眉条；翻页键改为 `-`/`=`；候选字改为宋体并修复字体句柄从未生效的既有缺陷；上下键页内选择候选（空格上屏当前项）；页脚 m/n 翻页指示（T-031/T-033/T-034/T-039/T-040）
- 拼音未完整切分时显示前缀候选（混合列表）：`nih`→你好；前缀补全组置前、切分组合置后（T-029）
- 语言栏中英模式图标：ITfLangBarItemButton + ITfSource，运行时绘制中/英 16×16 图标（Win11 桌面语言栏需显式开启）（T-046）
- 设置窗口常用设置页（第八期 M12-3，FR-041/FR-042）：英文输入法默认中英模式**装配项**（D-32 改判：设置窗口跨进程读不到宿主 IME"当前模式"，条目改为二选一 chip、点击即存 `config.json` 的 `default_mode`（chinese|english 宽松解析）、TSF DLL 下次装配读取为新会话起始模式，提示"重启输入法后新会话生效"；运行中 Shift 切换仍是会话状态、不进窗口）+ 添加词库子视图（整内容区）：领域包列表（六字段 + 启用开关，基础包标注不可停用，勾选即存 `enabled_packs` 下次装配生效）+ 本地 `.zyct` 导入（`GetOpenFileNameW` 系统对话框 → `DictionaryFile::open` 全量校验任一失败拒绝且不动盘 → 复制入 `packs/` → `installed.json` upsert（source: Import、无版本、同 id 覆盖）→ 刷新列表；界面注明"本地导入 · 未签名、不参与在线更新签名信任链"（D-38））（T-075）

### 变更

- 候选窗与设置窗口共享 UI 原语抽为独立 crate `zhu-ye-ui`（颜色/主题/矩形/DPI/文本测量），设置窗口不再连带链接 TSF 侧代码；渲染行为不变（2026-10-04 候选窗三配置逐帧像素比对零回退，T-081）
- proptest 属性测试门禁：切分合法性与拼接守恒/候选排序全序确定性/bigram 后继/beam 边界四组属性测试纳入 `cargo test`（仅测试期依赖，运行时依赖图零变化，T-089，FR-049）
- 版本化 DLL 部署（T-026）：安装/卸载改为版本化命名 DLL + PE 头静态校验 + 注册表切换前快照与失败回滚 + 旧版延迟清理，支持无锁升级

## [0.1.0] - 2026-09-23

### 新增

- 基于 TSF 的 Windows 中文输入法 DLL 与安装/卸载闭环（T-010）
- 全拼按键、组合与上屏闭环（T-011），支持中英模式切换
- 标准全拼音节表与动态规划切分核心（T-007）
- v2 二进制词典格式、构建管线与 mmap 加载（T-006）
- CC-CEDICT 与 FrequencyWords 中文词频真实词库导入管线（T-006），清洗校验后生成 12 万级双语词典
- OPUS GlobalVoices 中文分词语料真实 bigram 统计与导入（T-006、T-008），真实词典含 820,368 个共现词对
- unigram + bigram 静态候选排序与用户词学习持久化（T-008、T-009）
- 候选窗 Win32 自绘，支持深浅色、DPI 与高对比度（T-012）
- 键位交互：Shift、Tab、翻页、数字选择与候选窗 TSF 联动（T-013）
- 本地双语翻译层与译文切换、英文反查（T-014）
- `zhu-ye-cli` 自检、候选演示、词典检查与性能基准（T-014、T-015）
- `zhu-ye-cli rank` 真实词典候选排序验证（T-008），候选生成统一下沉 zhu-ye-core
- 安装与便携包随带 v2 词典，TSF 运行时从 DLL 同目录加载（T-022）
- 性能基准与阈值验收脚本 `scripts/bench.ps1`（T-015）
- 主机侧端到端回归检查器与 `scripts/e2e.ps1` 验收入口，覆盖切分、排序、用户词、译文层与正反查（T-024）
- 便携测试包脚本 `scripts/package-portable.ps1`（T-010）
- Agent Notes 双语校验与归档脚本（T-018、T-020）

### 修复

- 修复 TSF 组合写入路径崩溃：改用只读查询加组合写入，规避 `ITfInsertAtSelection` 写入分支访问冲突（T-010、T-011）
- 修复候选窗翻页后因视图二次切片越界被误判为无候选而隐藏（T-013）
- 整词拼音存在直接词典条目时抑制多音节切分噪声候选（T-021）
- 便携包脚本适配 Windows PowerShell 5.1 的 UTF-8 BOM 输出（T-025）

### 变更

- 词典格式确定为 v2：128 字节头部、拼音索引、词条表、bigram、中文译文索引、英文反查索引与内容 SHA-256（T-014）
- `AiService.translate` 对齐方案设计，明确 `TranslationDirection` 参数；离线实现保持零网络空结果（T-023）
