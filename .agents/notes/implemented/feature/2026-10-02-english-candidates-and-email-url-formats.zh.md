# Agent Note: 英文候选层与邮箱/网址格式（场景 6，M9）

Status: implemented

[English](2026-10-02-english-candidates-and-email-url-formats.md) | 中文

## Problem

场景 6（中英混输，FR-030/FR-031）需要英文候选层（拼写纠正：`pytho`→python、`iphon`→iPhone）
与邮箱/网址格式补全（`me@163`→`.com/.cn/.net`、`www.exa`→`www.exa.com`）。原 D-08 方案假设
人工编写 10,000 词种子表，手工编审不可行；D-10 要求英文候选不得挤占中文候选，也不得让
守护 `yyds` 的缩写（网络语）路径回退。

## Decision

自 T-085（M13）起，英文词主数据源为 ECDICT 全量的词表文件 `en.zyen`
（[英文词表文件 ZYEN v1](../../implemented/feature/2026-10-16-en-wordbook-zyen-v1.zh.md)）；
本文所述静态表保留为文件缺失或加载失败时的内嵌回退，公开语义一致（按 rank 排序→截断、
置主候选之后、`pinyin = None`）。英文词表位于 `zhu-ye-core/src/en_words.rs`，由
`scripts/build-en-words.ps1` 生成
（原始缓存 + pin 哈希锁定；生成文件入库、绝不手改）：

- 基座：FrequencyWords 英文词频（D-018，CC BY-SA 4.0），ASCII 清洗（a–z 与撇号、
  长度 2–32）后按频次取前 10,000。
- 大小写：人工补丁表 `data/patches/en-capitals.tsv`（D-019）恢复官方原形
  （`API`/`PHP`/`iPhone`/`QQ`）；命令语境小写两可词（`python`/`java`/`git`）保留小写。
- 噪声排除：`data/patches/en-exclude.tsv` 移除英文语料高频的中文人名音译
  （`zhang`/`wang`/`chen`…）；否则 `zh`（每位中文用户高频输入的拼音声母前缀）会弹出
  "zhang" 候选。排除同时作用于频率段与 CEDICT 补充段。
- 补充：CC-CEDICT 英文侧（D-001）仅补纯英文单词词目（过滤括号注释与空格短语），
  排名在基座之后。

表项存 `norm`（小写 ASCII 查键，字节序升序供 `partition_point`）、显示原形 `word` 与
组内排序 `freq_rank`。`en_words_with_prefix` 大小写不敏感并截断结果。

引擎（`crates/zhu-ye-ime/src/input.rs`）仅在整串**无法按拼音切分**（`segment_all` 空、
长度 ≥2）时把英文组追加到主候选**之后**：中文命中永不被挤占，`wo` 行为不变；未命中
自然落入既有缩写路径，`yyds` 不回退。英文候选 `pinyin = None`，不进用户词学习。

邮箱/网址为确定性的 core 模块 `email_url.rs`
（`detect_format`/`email_candidates`/`url_candidates`）：`@` 前至少有 1 字符 → 邮箱分支
（`.com/.cn/.net`，至多 3 条）；`www.`/`http://`/`https://` 前缀（大小写不敏感）→ 网址
分支（`.com/.cn/.org`，至多 3 条）；串已含 `.` 视为完整，单候选直通。`@` 尾空（`me@`）
同样补全——组合态按 `@` 后第一帧就要出候选（T-066）。引擎在拼音路径之前先按格式类别
路由。`handle_format_char` 在组合态接收 `@ . / :` 进串、空闲态返回 `false`——`@` 从不
冷启动组合，由宿主放行。

TSF 层新增统一动作 `KeyAction::FormatChar(char)`：`classify_key` 产出 `@`（Shift+2）、
`:`（Shift+OEM_1）、`/`（OEM_2），既有 `Dot`（VK_OEM_PERIOD）由 `plan_action` 重定向；
`plan_action` 查询引擎 `is_format_key`——仅在组合态且处于邮箱/网址上下文或网址意图演进
（`www`/`http`/`https` 及其 `:`/`/` 中间态，如 `http:`→`http:/`→`http://` 逐键）时吃键。
普通拼音组合（`nihao`+`.`）、空闲态与英文模式返回 `false` 放行宿主，因此"拼音组合中
`.` 直出标点"的既有行为不回退。

## Alternatives considered

- **人工把关 10,000 词种子**：体量不适合人工编审；改为 FrequencyWords 派生基座 +
  少量人工补丁。
- **触发长度门槛（如 ≥3 字符）替代排除清单**：会牺牲有用的两字母词（保留 `ok`/`me`
  支持）；`zh` 等拼音声母前缀仍需真正过滤，只有排除清单能做到。
- **大小写猜测**：否决（D-09）——展示表内原形，永不猜测。
- **英文候选参与主排序**：否决（D-10）——置后追加，绝不干预中文排序。
- **扩展 v2 词典格式承载英文词**：否决——表格量级适合 crate 内嵌静态切片，无格式成本。
- **邮箱/网址候选复用拼音管线**：否决——独立格式分支保证普通中文输入路径零改动。

## Consequences

- EN_WORDS 共 15,534 条；约 1.1 MB 生成 Rust 源码带来少量编译成本与受表切片约束的
  前缀扫描开销。
- 可切分英文词（`me`、`wo`）按 D-10 刻意不进英文路径（拼音优先）；用户想要两字母
  英文词须接受中文音节命中或用未切割串。
- 表维护是加法的、低风险：往大小写/排除补丁加条目 → 重跑脚本 → 提交重新生成的表。
- 生成文件必须保持 rustfmt 干净（脚本自带格式化），`norm` 字节序绝不可破坏，否则
  `partition_point` 查询会悄悄退化。
