# Agent Note: 任务栏指示器"竹"图标——DLL 内嵌品牌图标资源（T-112 修订）

Status: implemented

[English](2026-10-06-taskbar-icon-zhu.md) | 中文 | 中英镜像

## Problem

用户反馈 Win11 任务栏输入法指示器在竹叶输入法激活时显示语言回退
"简体"，而非品牌标识。期望与现役输入法一致：微软拼音显示"拼"、
搜狗拼音显示"S"（其 DLL 图标），竹叶应显示单字"竹"；同时输入法
**名称**保持"竹叶输入法"完整不变。早前 T-112 曾把语言栏项
`ITfLangBarItem::GetInfo` 的 `szDescription` 改为单字"竹"——方向错误：
那改的是名称，且 [T-046 语言栏图标记录](../../implemented/feature/2026-09-28-language-bar-mode-icon.md)
已写明 Win11 任务栏指示器不渲染第三方语言栏项目。

## Decision

Win11 任务栏指示器按 TIP 语言档注册的 `IconFile` 加载图标资源
（`ExtractIconEx` 语义，`IconIndex=0` 取第一个图标），与语言栏项目
`GetIcon`（T-046，"中"/"英"模式图标）是两套互不干扰的机制。因此：

- **图标**：`crates/zhu-ye-ime/assets/zhu.ico`（32×32，品牌橙底
  `#E5881E` 白字"竹"，System.Drawing 生成，766B）经 `build.rs` 用
  `winresource`（build-dependencies）嵌入 `zhu_ye_ime.dll`；语言档
  `IconFile` 指向 DLL、`IconIndex=0`。`ExtractIconEx` 验证：提取得
  非空 HICON。
- **名称**：`GetInfo` `szDescription` 与 LanguageProfile
  `Description`/`Display Description` 一律还原"竹叶输入法（中英切换）"；
  单字"竹"只存在于图标，不再占用名称字段。
- **注册完备性（排障顺带固化）**：TSF 枚举 TIP 依赖
  `LanguageProfile` 的 `Enable=1`(DWord)、`IconFile`、`IconIndex` 与
  `Category\Item\{tip}\{cat}` 关联；字段缺失时 TIP 不出现在
  Win+Space/输入法列表。补齐后 `ITfInputProcessorProfiles::
  EnumLanguageProfiles(0x0804)` 立刻列出竹叶
  （`{E54D6682-8650-40E7-A9EE-6FD1137849AE}` 在列）。HKLM TSF 子树
  普通权限不可读（ACL deny，连系统 TIP 也读不到），读写一律走提权
  通道（见部署记录 2026-10-06 排障链路）。

## Alternatives considered

**语言栏项 `szDescription="竹"`（T-112 原方向）。** 否决：修改的是
名称字段；Win11 任务栏指示器不读第三方语言栏项描述，且会破坏
"竹叶输入法"完整名称。

**`IconFile` 指向独立 `.ico` 文件。** 否决（中文档案）：TSF 图标按
DLL/exe 资源路径解析（微软拼音 `ResourceDll.dll`、搜狗
`SogouTSF.ime` 均内嵌），独立 ico 文件的加载路径在当时无官方先例
保证。**批四修订**（[2026-10-07-acceptance-fix-batch-4.zh.md](2026-10-07-acceptance-fix-batch-4.zh.md)）：
英文态档案 `{EF42481A-…}` 采用独立 `ying.ico` 作为其 IconFile（Rime
weasel 以 weasel.ico 文件为档案图标的成熟先例，任务栏指示器可正常
提取）；中文档案维持 DLL 内嵌（随版本化 DLL 指针切换，零文件依赖）。

**语言栏项目 `GetIcon` 动态绘"竹"。** 否决：T-046 记录已证 Win11
任务栏不渲染第三方语言栏项目；该机制只影响经典语言栏。

**构建后脚本注入 DLL 资源。** 否决：增量构建会覆盖注入结果，不可
复现；`build.rs` 内嵌与源码同生命周期。

## Consequences

- DLL 体积增加约 1.5KB（图标资源），无运行时依赖与网络成本；每构
  建一次即重新内嵌，门禁（fmt/clippy/test）涵盖改动路径。
- 任务栏图标为静态"竹"字，不随中/英模式变化——模式仍由语言栏项目
  的"中"/"英"图标表达（T-046）。
- 版本化部署语义不变：`IconFile` 指向 `Program Files\zhu-ye-ime\tsf\
  zhu_ye_ime.dll`，随 DLL 更新即换图标。
- tests 更新：`getinfo身份与描述正确` 断言完整名称逐字与空终止。
- **批四扩展（2026-10-07）**：同 TIP 树注册英文态第二语言档案
  `{EF42481A-233D-4035-A80A-7F416BE0E6CA}`（IconFile=独立
  `ying.ico`），Win+Space 切换中/英档案使任务栏指示器显示竹/英；
  中文档案仍走 DLL 内嵌 + IconFile=DLL 指针。见
  [2026-10-07-acceptance-fix-batch-4.zh.md](2026-10-07-acceptance-fix-batch-4.zh.md)。
