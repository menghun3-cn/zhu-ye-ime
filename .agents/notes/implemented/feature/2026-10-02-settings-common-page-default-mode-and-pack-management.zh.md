# Agent Note: 设置窗口——默认中英模式与词典包管理（T-075）

Status: implemented

[English](2026-10-02-settings-common-page-default-mode-and-pack-management.md) | 中文

## Problem

常用设置页（T-075）还剩两项交付。其一，中英模式条目原本计划做成"当前模式"只读展示
加即时切换的会话状态视图，但设置窗口是独立进程，读不到宿主输入法运行中的会话模式，
"当前模式"在跨进程语境下无意义（D-32）。其二，"添加词库"（FR-022/FR-042）需要一份
已安装领域包清单（带启用/停用开关）和一个本地 `.zyct` 导入流程，导入文件的安全语义
由 D-38 决定（不验签、不进在线更新的签名信任链）。

## Decision

**默认中英模式**是装配项而非会话状态。设置窗口把 `config.json` 的 `default_mode`
写为 `"chinese"` 或 `"english"`（`deserialize_default_mode` 宽松解析、缺省中文；
该选择不递增配置格式版本）。TSF DLL 在下次装配（Activate）时经
`configured_default_mode` 读取，作为新会话的起始模式。页面条目是"中文/英文"二选一
chip，选中当前已保存的默认值；点击即存并提示"重启输入法后新会话生效"——与主题条目
同样的点击即存模式（S-8 保存前重读）。运行中的 Shift 切换仍是会话状态：不持久化，
也刻意不在窗口里暴露；该决策记录见
[设置窗口进程与注册所有权 note](../../proposed/architecture/2026-10-01-settings-window-process-and-registration-ownership.md)。

**添加词库子视图**整体替换页面内容区（不做行内展开：常用设置页现有 12 条，已超过
482 px 内容列的可视高度）。顶部说明固定标注"导入包以本地文件为准，不参与在线更新的
签名信任链（不验签）"（D-38）。每行显示：包名、一句话简介、"N 词条 · M KB"与版本
（导入包显示"本地导入 · 未签名"），以及启用开关。勾选写入 `enabled_packs` 并保存
（装配项，下次装配输入法时生效）；基础包只标注"基础包"、不可停用。行数据来自
`inventory::list_packs`：先列已知包元数据，再按 UTF-8 字节序追加磁盘 `packs/*.zyct`
里的未知包，包名回退为文件主名。工具栏把"← 返回常用设置"和主按钮"导入本地 .zyct…"
固定在内容区底部；行数超过可视高度时截断、绝不压缩行高。

**导入流程**（`window::import_zyct`）：`GetOpenFileNameW` 系统对话框
（`OFN_FILEMUSTEXIST | OFN_HIDEREADONLY | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR`，
过滤器 `"词典包 (*.zyct)\0*.zyct\0全部文件 (*.*)\0*.*\0"`）→
`zhu_ye_core::DictionaryFile::open` 全量校验（魔数、版本、头部、内容 SHA-256、
分区布局）；任一失败直接拒绝且不动磁盘（"导入失败：…（文件未改动）"）→ 复制到
`packs/{主名}.zyct`（同 id 覆盖）→ `sha256_file` → `installed.json` upsert
（`source: Import`、无版本）→ 刷新列表（"已导入「…」（本地导入，不参与签名校验），
勾选启用后重启生效"）。复制成功但清单写入失败时如实提示（"导入完成但清单记录失败…"），
不假装已回滚。

窗口沿用应用其余部分的职责切分：`SettingsState`（`packs_view` 标记、
`default_mode`）承载瞬态 UI 状态，`WindowState.packs` 持有磁盘重扫结果；
`list_packs_now` 在每次进入/勾选/导入前按 S-8 重读 `config.json` 与
`installed.json`。`--packs`（配 `--shot`）用于出子视图的验收截图。

## Alternatives considered

- **行内展开"添加词典"条目**：否决——常用页已有 12 条，内容列 (~482 px) 在无滚动
  的前提下放不下；整内容区子视图复用既有页面机制（导航与返回按钮都能退出）。
- **只读"当前模式"展示 + 即时 Shift 式切换**（最初的会话状态方案）：D-32 下否决——
  跨进程设置窗口读不到宿主 IME 的当前模式，唯一有意义的语义是新会话起始模式；
  运行时切换留在输入法内且不持久化。
- **模式与包勾选的显式"保存"按钮**：否决——既不符合主题条目的点击即存，也不符合
  S-8 先重读后保存的纪律；显式保存是第三种交互模式，对用户没有额外价值。
- **扫描上游词典目录或 exe 目录找包**：否决——只扫 `packs/`；基础包
  `dictionary.zyct` 放在 exe 同目录（`shell::exe_dir`），与 `packs/` 导入命名空间
  永不冲突。
- **导入文件纳入在线签名链校验**：否决（D-38）——界面明确标注导入文件不验签；
  清单记录携带 `source: Import`，信任链仍可审计。

## Consequences

- `default_mode` 写入 `config.json` 但不递增格式版本；未知值回退中文且不损坏
  其他字段。
- 导入包在列表中标注"本地导入"、元信息行标注"未签名"；可启停，但永不进入在线更新
  签名链（[信任链 note](../../implemented/architecture/2026-09-29-dictionary-update-trust-chain.md)）。
- 导入器写入的 `installed.json` 记录与更新器记录兼容；清单按非权威处理，`packs/`
  重扫能兜底缺失或损坏的清单
  （[部署 note](../../implemented/architecture/2026-09-21-installed-dictionary-deployment.md)）。
- 导入对话框与词库子视图属于宿主交互；VM 自动化验收（T-080）覆盖对话框，宿主内截图
  覆盖模式页与词库子视图的布局。
