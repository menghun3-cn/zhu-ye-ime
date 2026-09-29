# Agent Note: 领域词包构建与 manifest 管线（M6-P）

Status: implemented

[English](2026-09-28-dictionary-pack-build-and-manifest.md) | 中文

## Problem

M6 需要发布多个词典包（基础包、领域包、网络语包）供运行时组合。THUOCL 一类的词表只含"词 + 文档频率"，没有拼音——构建期必须先注音才能编译为 ZYDT v2 二进制格式。管线还需要一种可机检的方式发布整组包（manifest），并在构建前证明缓存输入仍是锁定版本（source-check）。一期没有覆盖这些：`zhu-ye-dict` 只有 `build/import/inspect/verify`，各包也没有可复现性门禁。

## Decision

`zhu-ye-dict` 新增六个 M6 子命令；`build-slang` 在 `slang.rs`（见 [网络语包与内容把关](2026-09-28-slang-pack-and-content-gate.zh.md)），其余在 `m6.rs`：

- `source-check` — 读取全部 `data/pins/*.json`，对每个 pin 指向的缓存/快照文件做哈希校验；任一锁定哈希不匹配即构建失败。未锁定（缺 `sha256`）的 pin 只报告不失败。`build-pack` 先执行 `source-check`，源漂移永远不会静默进入构建。
- `build-pack <it|med>` — 解析 `THUOCL_*.txt`（`词<TAB>DF`），仅保留纯 CJK 词形，注音，按 (词, 拼音) 去重，用 `build_v2` 编译并写入 `data/artifacts/<id>.zyct`。词频取 THUOCL DF（封顶 u32::MAX）；领域包不带译文（现有管线本就过滤空译文）。`it` 包另并入 MDN zh-cn 术语表标题（D-015，`kind: bundle` 类 pin，由 `scripts/fetch-mdn-glossary.ps1` 按 slug 从锁定 commit 抓取）：`parse_mdn_titles` 取每页 front matter 的 `title:`，去引号与括注的英文/缩写，按 `、` 与 `/` 拆分，保留 ≥2 字纯汉字术语；统一词频 500（THUOCL_IT DF 中位数 441）。
- `build-base [--min-score N]` — 合并骨架（XDHCY 56,008）、CC-CEDICT 词条（带译文）与 jieba 扩充为 `base.zyct`，并记录实测统计（骨架数、wordfreq 命中率、扩充数、字集覆盖、体积、SHA-256）。输出确定：同输入两次构建字节级一致（2026-09-28 实测验证）。
- `build-manifest [目录] [--version V] [--min-engine V]` — 扫描产物目录内 `*.zyct`，写出 `manifest.json`（JSON schema 1），逐包含 `id/name/version/file/sha256/size/min_engine_version`，顶层含 `schema` 与 `published_at`。manifest 类型定义在 `zhu-ye-core`，构建侧与运行时侧共用同一传输格式；`sign-manifest` 追加 ed25519 签名块（见[更新信任链笔记](../architecture/2026-09-29-dictionary-update-trust-chain.zh.md)）。manifest 默认 `version` 为 UTC 构建日期，`min_engine_version` 默认 "0.1.0"。
- `verify-manifest <manifest.json>` — 逐包校验：文件存在、内容 SHA-256 与字节大小和 manifest 一致；任一不一致即失败。

### 基础包数据流

D-010 官方 XDHCY PDF 是扫描图片版（页面内容流为 `/Im0` 图像，无文本层），因此 `build-base` 消费转录文本镜像（`liuxilu/Proofread-Modern-Chinese-Common-Lexicon` 的 `现代汉语常用词表.txt`，行格式 `词<TAB>拼音<TAB>序号`，带调数字拼音）；pin（`data/pins/xdhyc-2008.json`）把官方 PDF 哈希保留在 `legacy` 供溯源。行格式归一（`load_xdhyc`）：

- 异形词并列行 `甲;乙`（如 `年轻;年青`）拆为两条词共享同一拼音；
- 儿化 `hua1'r` 归一为 `hua1'er`，**仅当 `r` 后不再跟字母**（`sui1'ran2` 的 `ran` 保持原样）；
- 顿号行（`宁为玉碎,不为瓦全`）与间隔号行（`一二·九运动`）去除分隔符，拼音侧 `sui4',bu4` 合并为 `sui4'bu4`；
- 全角外来字符行保留词形（`阿Ｑ`）；已知排除《卡拉ＯＫ》（音节 `kei1` 非普通话标准音节，1/56,008）。

词频合并（去重取 max）：wordfreq 为主源——wheel 内 `wordfreq/data/large_zh.msgpack.gz` 为 gzip → msgpack `[header, 0cB 词表, -1cB 词表, …]`（cBpack），第 k 桶放 −k cB 的词，`zipf = 9 − k/100`；存储为 `round(zipf×1000)`（u32）。wordfreq 未命中回退 jieba，标定到同尺度（`2000 + 1000·log10(频次)`，封顶 9000）；两源皆无按 1。注音：骨架用镜像自带官方拼音；CC-CEDICT 词级同时提供拼音与译文；jieba 扩充经 CC-CEDICT 词级 → kTGHZ 字级兜底注音。`--min-score` 为 jieba 扩充入口阈值（默认 2000），即 S-1 调参口。

2026-09-28 实测：56,008 行全部加载 → 56,062 词形；wordfreq 命中 52,070（92.9%，≥70% 内部门槛）；CEDICT 词级 75,156（含译文）；jieba 扩充 247,423；378,312 词条，29,559,095 字节（28.2 MB，≤60 MB）；kTGHZ 字集覆盖 95.5%（未覆盖字多为生僻字，部分由 jieba 单字词承担）。`base.zyct` 已纳入 manifest（2026.09.28-p3 含网络语包，`verify-manifest` 6/6）。新增构建工具依赖：`flate2`、`rmpv`、`zip`（运行时 crate 不受影响）。

### 拼音注音策略

注音分两层，两层都经引擎标准音节表（`zhu_ye_core::pinyin::SyllableTable`）校验：

1. 词级：CC-CEDICT（D-001）词→拼音表，沿用既有导入清洗规则（纯 CJK、完整标准音节、音节数=字数）。一词多音时**保留首个读音**（CC-CEDICT 条目按常用度排序，首读即常用读法）。
2. 单字级：Unihan `kTGHZ2013.txt`（D-014，新增 pin，MIT）为 8,105 个规范汉字提供首选拼音（带调输入归一化为无调 ASCII，`ü`→`v`）。词级未命中的词逐字注音并再次校验。

2026-09-28 实测：IT 包 THUOCL 16,000 行 + MDN 术语 380 → 13,144 词条（MDN 入包 291，其余 89 条与 THUOCL 重复），0.96 MB；医学包 18,749 行 → 18,675 词条（注音失败 74），1.37 MB。注音失败来自 8,105 字集之外的生僻字，按设计丢弃。

## Alternatives considered

**THUOCL 词不带拼音，交给引擎按字切分。** 否决：运行时组合的是整词词条；在运行时做字级回退等于把注音工作搬到热路径，还少了保证候选质量的标准音节校验。

**改从其它未锁定来源取单字拼音。** 为可复现性否决：kTGHZ2013 有 pin、MIT 许可，且正好覆盖官方 8,105 字集——与基础包同一基准，一个字集、一个权威。

**manifest 留到 M6-U 再生成。** 否决：在构建里程碑就要能用哈希复核产出（包创建后立即校验）；签名是 M6-U 纯增量步骤。

## Consequences

- 收益：每次构建全程哈希门禁（pins → 注音 → v2 → manifest → verify）；领域包当下即可复现且体积受控；manifest schema 是 M6-U 签名 manifest 与 M6-R 运行时复合词典的稳定输入。
- 成本：构建工具 crate 新增依赖（`serde`/`serde_json`/`sha2`，基础包另加 `flate2`/`rmpv`/`zip`），运行时 crate 不受影响；注音失败数需按源观察（医学包首建 74，见 `docs/数据清单.md` D-014 说明）。
- 剩余 M6-P：base 的 S-1 调参收敛（`--min-score` 扫描 + 首候选覆盖抽样，验收标准 7.5 回填数字正式定稿前执行）。
- 不取代任何既有笔记；扩展 [数据源 pins 与获取脚本](2026-09-28-dictionary-source-pins-and-fetch-script.zh.md) 所述机制。v2 二进制格式不变（见 [v1 二进制格式](../../architecture/2026-09-19-dictionary-binary-format-v1.zh.md)）。
