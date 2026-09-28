# Agent Note: 领域词包构建与 manifest 管线（M6-P）

Status: implemented

[English](2026-09-28-dictionary-pack-build-and-manifest.md) | 中文

## Problem

M6 需要发布多个词典包（基础包、领域包、网络语包）供运行时组合。THUOCL 一类的词表只含"词 + 文档频率"，没有拼音——构建期必须先注音才能编译为 ZYDT v2 二进制格式。管线还需要一种可机检的方式发布整组包（manifest），并在构建前证明缓存输入仍是锁定版本（source-check）。一期没有覆盖这些：`zhu-ye-dict` 只有 `build/import/inspect/verify`，各包也没有可复现性门禁。

## Decision

`zhu-ye-dict` 新增四个 M6 子命令，全部实现在 `m6.rs`：

- `source-check` — 读取全部 `data/pins/*.json`，对每个 pin 指向的缓存/快照文件做哈希校验；任一锁定哈希不匹配即构建失败。未锁定（缺 `sha256`）的 pin 只报告不失败。`build-pack` 先执行 `source-check`，源漂移永远不会静默进入构建。
- `build-pack <it|med>` — 解析 `THUOCL_*.txt`（`词<TAB>DF`），仅保留纯 CJK 词形，注音，按 (词, 拼音) 去重，用 `build_v2` 编译并写入 `data/artifacts/<id>.zyct`。词频取 THUOCL DF（封顶 u32::MAX）；领域包不带译文（现有管线本就过滤空译文）。
- `build-manifest [目录] [--version V] [--min-engine V]` — 扫描产物目录内 `*.zyct`，写出 `manifest.json`（JSON schema 1），逐包含 `id/name/version/file/sha256/size/min_engine_version`，顶层含 `schema` 与 `published_at`。**本里程碑不签名；ed25519 签名在 M6-U 引入。** manifest 默认 `version` 为 UTC 构建日期，`min_engine_version` 默认 "0.1.0"。
- `verify-manifest <manifest.json>` — 逐包校验：文件存在、内容 SHA-256 与字节大小和 manifest 一致；任一不一致即失败。

### 拼音注音策略

注音分两层，两层都经引擎标准音节表（`zhu_ye_core::pinyin::SyllableTable`）校验：

1. 词级：CC-CEDICT（D-001）词→拼音表，沿用既有导入清洗规则（纯 CJK、完整标准音节、音节数=字数）。一词多音时**保留首个读音**（CC-CEDICT 条目按常用度排序，首读即常用读法）。
2. 单字级：Unihan `kTGHZ2013.txt`（D-014，新增 pin，MIT）为 8,105 个规范汉字提供首选拼音（带调输入归一化为无调 ASCII，`ü`→`v`）。词级未命中的词逐字注音并再次校验。

2026-09-28 实测：IT 包 16,000 行 → 12,853 词条（注音失败 1、去重 1），0.94 MB；医学包 18,749 行 → 18,675 词条（注音失败 74），1.37 MB。注音失败来自 8,105 字集之外的生僻字，按设计丢弃。

## Alternatives considered

**THUOCL 词不带拼音，交给引擎按字切分。** 否决：运行时组合的是整词词条；在运行时做字级回退等于把注音工作搬到热路径，还少了保证候选质量的标准音节校验。

**改从其它未锁定来源取单字拼音。** 为可复现性否决：kTGHZ2013 有 pin、MIT 许可，且正好覆盖官方 8,105 字集——与基础包同一基准，一个字集、一个权威。

**manifest 留到 M6-U 再生成。** 否决：在构建里程碑就要能用哈希复核产出（包创建后立即校验）；签名是 M6-U 纯增量步骤。

## Consequences

- 收益：每次构建全程哈希门禁（pins → 注音 → v2 → manifest → verify）；领域包当下即可复现且体积受控；manifest schema 是 M6-U 签名 manifest 与 M6-R 运行时复合词典的稳定输入。
- 成本：构建工具 crate 新增依赖（`serde`/`serde_json`/`sha2`），运行时 crate 不受影响；注音失败数需按源观察（医学包首建 74，见 `docs/数据清单.md` D-014 说明）。
- 剩余 M6-P 拆为后续任务：`build-base`（XDHCY 草案 PDF 文本抽取、wordfreq 3.1.1 数据抽取、词频合并、S-1 体积/覆盖实测）与 `build-slang` 及 MDN glossary 正文抓取。
- 不取代任何既有笔记；扩展 [数据源 pins 与获取脚本](2026-09-28-dictionary-source-pins-and-fetch-script.zh.md) 所述机制。v2 二进制格式不变（见 [v1 二进制格式](../../architecture/2026-09-19-dictionary-binary-format-v1.zh.md)）。
