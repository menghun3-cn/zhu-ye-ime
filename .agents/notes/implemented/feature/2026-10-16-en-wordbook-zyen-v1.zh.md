# Agent Note: 英文词表文件 en.zyen（ZYEN v1）——ECDICT 全量英文词典（T-085）

Status: implemented

[English](2026-10-16-en-wordbook-zyen-v1.md) | 中文

## Problem

FR-046（M13）要求以 ECDICT 全量（770,611 行）作为英文词典，取代 [M9 英文候选层](../../implemented/feature/2026-10-02-english-candidates-and-email-url-formats.zh.md)
的内嵌静态表（约 15,534 条）：用户期望 `pytho`→Python、`iphon`→iPhone、`ap`→API/Apple
覆盖全部词汇而非顶部切片。D-51a 明确否决按词频截断（"目的是全"），并把加载时间（≤50ms）、
查询延迟（≤0.5ms 中位数）与产物体积（≤20MB 预算）转为硬性验收指标。77 万条规模的 crate
内嵌 Rust 静态切片不可维护（源码数十 MB、编译时间爆炸）；v2 `base.zyct` 词典格式按拼音键
的中文词条与翻译索引组织，与"按 rank 排序的英文前缀查询"形态不符。

## Decision

英文词典以独立 mmap 文件 `en.zyen`（ZYEN v1）随发行包分发（与 `base.zyct` 并列，T-078），
由 IME 引擎加载，作为静态表的无缝升级路径。

**数据管线**（`zhu-ye-dict en-build`，输入经 `data/pins/ecdict.json` + `data/cache/ecdict-full.csv`
SHA-256 锁定）：

- 主表：ECDICT 行清洗至 `[A-Za-z'\- ]`、至少 1 字母、长度 2..=40（源内"词:释义"污染行剔除）；
  770,611 行入 → 760,951 保留；小写键去重取 `(frq, bnc, word)` 三元组最大者。
- 排序/rank 键：`(frq>0, frq, bnc)` 降序，norm 字典序（二分搜索依赖）；rank = 排序后下标
  （越小越常用）。frq 覆盖保留行 100%、bnc 75.4%——无空档，确定性稳定。
- 补充：FrequencyWords（D-018）前 1 万补 ECDICT 未收录（79 条，以词频占位 rank 输入）；
  CC-CEDICT 英文侧（D-001）补纯单词（实测 0 条——ECDICT 已覆盖）。
- 补丁：`data/patches/en-capitals.tsv` 复原官方写法/强推 rank（应用 92）；`data/patches/en-exclude.tsv`
  剔除噪声（51）。两者沿用 M9 清单，仍然权威。
- 结果：760,987 词条，产物 19,462,460 字节（18.56 MiB ≤ 20MB）。

**ZYEN v1 二进制布局**（定稿，加载时校验）：

- 头部 96B：magic `"ZYEN"`、version u32=1、count u64、reserved u64、
  records_off/pool_off/anchors_off u64、pool_len u64、anchor_stride u32=1024、
  anchor_count u32、content_hash [u8;32]=第 96 字节之后全部的 SHA-256。
- 记录 11B/条：`norm_off u24, word_off u24, norm_len u8, word_len u8, rank u24`；
  `word_len==0` 复用 norm 池段（显示形==norm）。记录按 norm 字节序升序（二分依赖；
  编译断言 ASCII 小写、非空、≤255）。
- 池：单段 UTF-8 承载全部 norm/word 文本（相同时去重）。
- 锚：每 1024 条一个 12B 桶（`key_pfx [u8;8]`、`idx u32`），短前缀查询不必从头扫描。
- 查询：小写化 → 按 key_pfx 二分定位桶首 → 记录二分找首个 norm ≥ 前缀 → 前缀区间扫描 →
  最大堆保顶 `limit` 个最小 rank（避免 "a"/"b"/"c" 十万级命中全排序）；结果语义与 M9 静态表
  （`sort_by rank → truncate`）一致。

**装载**（`crates/zhu-ye-core/src/en_lexicon.rs`）：

- `EnLexicon::open/from_bytes`：mmap（或堆）+ 主体全量 SHA-256 + `validate_layout`
  （边界、norm 全小写 ASCII、严格升序、rank<count、锚单调且桶首一致）；布局校验
  用 `std::thread::scope` 分块并行（无 rayon、无 Windows API——core 保持可移植），
  跨块升序衔接链顺序复检。
- 引擎装配解析顺序（`tsf.rs`）：DLL 同目录 `en.zyen`，否则 `%APPDATA%\ai-zhu-ye-ime\en.zyen`；
  加载失败 `debug_log` 并回退 M9 静态表；两者皆无 → 静默 `None`（FR-030 与 T-085 前行为一致）。
- `input.rs` FR-030 分支：有词典走 `en_word_candidates_from(lexicon, ...)`，否则
  `en_word_candidates(...)`（静态表）——同一映射（`score=-(rank as i64)`、`pinyin=None`、
  置主候选之后、上限与静态表一致）。

**工具**：`zhu-ye-dict` 子命令 `en-build` / `en-bench` / `en-inspect`；
`scripts/build-en-wordbook.ps1`（PS 5.1）核对缓存 CSV 与 pin、构建、实测，加载 >50ms、
查询中位 >500µs 或产物 >20MB 即失败退出。`package-portable.ps1` / `install.ps1` 将
`bin/en.zyen` 复制到 DLL 同目录；文件缺失退化为静态表回退而非安装失败。

**实测（2026-10-16，release 构建）**：加载+校验 28.08ms；20 万次查询中位 43.2µs、
P99 2.2ms；产物 19,462,460B；校验和稳定。

## Alternatives considered

- **静态表扩到 77 万**：生成 Rust 源码数十 MB，编译成本与表体积不可维护；否决。
- **扩展 v2 `.zyct` 格式加英文区**：英文前缀查询被捆绑在拼音键的中文布局与翻译索引上，
  版本与按词频增量更新都要拖着基础词典走；否决，改独立文件（独立 mmap 分页、可单独按词频
  更新、与 `base.zyct` 版本解耦）。
- **按词频截断 10–20 万**：D-51a 否决——目的是全量；性能转硬验收指标。
- **加载时不做完整性校验（信任发行 manifest）**：发行清单虽已对产物哈希，但加载期全量
  SHA-256 仅约 12.7ms（sha2 硬件加速），可在任何查询前捕获篡改/损坏；保留并尽量并行。
- **命中区间全排序**：短前缀 10 万+ 命中使 P99 达 7.9ms；最大堆只保 `limit` 个最小 rank
  （limit 通常 ≤6）把中位压到 43µs 且保持 `sort_by rank → truncate` 语义。
- **引入 CSV crate 解析 ECDICT**：保留手写解析器（引号字段、`""` 转义对、尾列 audio
  频繁为空的尾部空字段规则）——零新增运行时依赖，符合项目自研可控立场。

## Consequences

- `en_words.rs`（15,534 条）保留为 `en.zyen` 缺失/损坏时的回退；两条路径共享同一公开
  语义，任意部署状态下 FR-030 行为不变。
- 产物在发布期构建（源码树不携带大二进制）；`data/cache/ecdict-full.csv` gitignored，
  `data/pins/ecdict.json` 入库。
- `EN_WORDBOOK_FILE_NAME` 双处维护（`identity.rs` + `scripts/ime-identity.ps1`），沿用
  既有 D-42 双份模式；`verify-tsf-identity.ps1` 不覆盖它（文件名非注册表身份）。
- 大小写相异的词保留独立显示段（大写原形 86,783 条约占文本池 10.9MB）——精确大小写显示
  （D-09）的代价。
- 发行包在文件缺失时静默退化为更小的静态表；验收门禁（加载 ≤50ms、查询中位 ≤0.5ms、
  ≤20MB）在文件路径上实测。
