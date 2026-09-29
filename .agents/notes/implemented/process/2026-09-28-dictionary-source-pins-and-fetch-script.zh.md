# Agent Note: Dictionary data source pins and fetch script

Status: implemented

[English](2026-09-28-dictionary-source-pins-and-fetch-script.md) | 中文

## 问题

M6（词典体系）在第一期三个数据源之上新增十余个数据源——jieba、THUOCL、官方词表、wordfreq、MDN 术语表。每个源都需要锁定 URL、SHA-256、许可证与获取流程，构建才能保持可复现、许可记录保持准确。此前数据源只在 `docs/数据清单.md` 里人工登记：没有机器可读的记录，也没有能下载并校验数据源的脚本。人工维护必然漂移（URL、版本、哈希），R4 数据源定稿后三个源的体量靠手工已不可行。

## 决策

`data/pins/*.json` 是所有进入词典管线的数据源的机器可读锁。每个 pin 携带 `id`、`name`、`role`、`license`、`license_note`，以及二选一：

- `kind: "url"` —— 可下载文件：`url`、`cache_file`（或指向 `data/raw/` 既有文件的 `cache_rel`），以及 `sha256`/`size`/`fetched_at`（由 `scripts/fetch-sources.ps1 -WritePins` 首次获取时回填）；或
- `kind: "snapshot"` —— 提交入库的小型快照产物，其哈希锁定构建输入，用于不稳定或分页的源。MDN zh-cn 术语表 slug 列表以 `data/pins/mdn-glossary-zh.snapshot.json` 入库（601 个 slug，2026-09-28 抓取自 `mdn/translated-content` 的 GitHub contents API）；M6-P 按 slug 逐条抓取页面正文。

`scripts/fetch-sources.ps1` 将缺失文件下载到 `data/cache/`（gitignore），逐一对已锁定的 SHA-256 校验，任何哈希不匹配都作为硬失败处理并保留旧缓存——绝不静默降级。`-WritePins` 在首次运行回填 `sha256`/`size`/`fetched_at`，锁来自实际下载；此后哈希锁定版本，源端任何变化都会显式失败，直到人工审查并重新锁定 pin。`-Force` 在缓存一致时也强制重下。`-DryRun` 只报告每个 pin 的预期动作，不碰网络。脚本面向 Windows PowerShell 5.1，并携带 portable-scripts 约定的 UTF-8 BOM。

既有三源 pin（`cedict`、`frequencywords-zh`、`globalvoices-zhs`）以已登记哈希引用 `data/raw/` 既有文件，脚本只校验不重下；M6-P 将新获取统一迁入 `data/cache/`。

## 曾考虑的替代方案

**继续在 `docs/数据清单.md` 里人工登记数据源。** 否决：三源时已开始漂移；十余个带版本哈希的源靠人工不可能保持准确，也没有可执行的途径证明登记的哈希仍然描述着已下载文件。

**把全部原始数据提交进 git。** 否决：仓库"大文件与构建产物不入库"的规则正是为数十 MB 的词典数据源而设（仅 wordfreq wheel 就有 57 MB）；入库的只有 pins 与一个很小的待提交快照。

**用 CI（GitHub Actions）抓取，而不是本地脚本。** 否决：项目至今没有面向 CI 的抓取路径，构建都在本地执行；本地 PowerShell 脚本对维护者与 VM 验收路径（技能已适配 Rust/PowerShell，无 Node 工具链）保持一致。

**用站点 sitemap 锁定 MDN 术语表。** 否决：`https://developer.mozilla.org/zh-CN/sitemap.xml` 返回 404。`mdn/translated-content/files/zh-cn/glossary` 的 GitHub contents API（601 项，两次分页请求）提供同一 slug 集合；slug 列表足够小，可以提交入库。

## 后果

- 收益：一条命令即可复现下载与校验；版本被锁定；哈希漂移显式失败；快照提交让不稳定、分页的源在构建上稳定。
- 代价：上游内容变化后刷新 pin 是人工、可审哈希的步骤；snapshot 类需要人工刷新（低频——MDN 术语表页面极少变更）。
- 与第一期管线的关系：`zhu-ye-dict import`（见 [真实词典导入](../../architecture/2026-09-21-real-dictionary-import.md)）保持不变；M6-P 的 `zhu-ye-dict` 构建子命令消费 `data/cache/`，并以 pins 为 `docs/数据清单.md` 登记的权威来源扩展管线。
- BOM 要求沿用 [portable-scripts PS 5.1 编码修复](../../bug-fix/2026-09-22-portable-scripts-windows-powershell-5-1-encoding.md)。
