# Agent Note: 多音缺读补丁接入产品链 base 包（shui→谁 等 31 条）

Status: implemented

[English](2026-10-08-polyphone-patch-into-base-pack.md) | 中文

## Problem

用户 2026-10-08 主诉："输入 shui，也要出'谁'字，类似问题的字都要出现"。
排查发现 T-056 的补丁机制只在 **import 管线**生效（`build_real_dictionary` →
`data/artifacts/real.zyct`），而发行/产品链的 `data/artifacts/base.zyct`
（`m6::build_base`：xdhyc 骨架 + wordfreq 词频 + jieba 扩充 + CEDICT 兜底）**未接入
补丁表**——发行包 `bin/dictionary.zyct` 即 base 的拷贝，因此用户机上「谁」仅有
`shei`、「熟」仅有 `shu` 读音，`shui`/`shou` 键不可达。T-056 当时在 VM 上验证的是
real.zyct（120,030 词条），没有覆盖 base 链，属管线覆盖盲区。

## Decision

1. **共享补丁应用函数**：`polyphone::apply_patch_entries(entries, patches, table)`
   从 import 内联逻辑提炼（只增不改、词频继承该字现有最高、字须已在词表、
   音节过标准表校验、跳过组合已存在/表内重复，返回 (应用数, 跳过数)）；
   `import` 与 `build_base` 双管线同口径，杜绝再次分叉。
2. **build_base 接入**：entries 构造后、`build_v2` 前无条件读
   `data/patches/polyphone.tsv` 应用；`BaseStats` 增
   `polyphone_applied`/`polyphone_skipped` 并在构建日志打印。
3. **base 侧缺读审计**：`audit-polyphone` 新增 `--base <zyct>`——枚举标准全拼音节
   （`SyllableTable::complete_syllables_with_prefix("")`）逐个 `lookup` 收集单字
   读音集，词频取词条该字最高值（无外部词频文件依赖）；审计产品链 base 而非
   CEDICT 的缺读。
4. **补丁表扩充 29 条**（共 31 条，见 `data/patches/polyphone.tsv`）：base 侧审计
   460 条缺读差集中**人工甄选口语/日常单字可打**读音——的di 了liao 都du 还huan
   着zhao/zhuo 地di 得dei 长zhang 觉jiao 绿lu 行hang 血xie 调diao 藏zang 乐yue
   吓xia 朝zhao 角jue 系ji 塞se 便pian 佛fu 似shi 俩liang 色shai 露lou 重chong
   陆lu。**排除**姓氏/文言/地名/争议音（区ou 万mo 大dai 说shui 会kuai 单chan/shan
   等）——组合词整词键已可命中，单字补读会污染候选键位。
5. **验收子命令**：`zhu-ye-dict lookup <文件> <拼音>` 按完整拼音列出词条
   （词频降序），构建/部署验收与补丁核对用。

## Alternatives considered

- **引擎特判 `shui`→谁**：只治一个症状、绕过排序确定性、污染热路径；T-056 已否决，
  维持。
- **460 条缺读全量补入**：大量姓氏/文言/地名读音（`verifying: 区ou 龟qiu 万mo
  单chan/shan`）会污染候选键（如 `qiu` 冒出「龟」）；词频继承让这些字压过本音
  常用词；否决。
- **在 CEDICT 源上加行**：数据源只读（pin 锁定），且 CEDICT 拼音字段语义是
  "该词条的读音"，不宜为单字口语音铺开；维持补丁表。
- **repo 内烧录缺读清单**：审计是过程不是产物，补丁表仍须人工把关；审计只为
  缩小挑选范围。

## Consequences

- base 重建 287,233 词条（+29），`多音补丁 31/跳过 0`；`shui` 键「谁」居首
  （5840 > 水 5290）——用户主诉达成；`liao`→了、`zhuo`→着、`yue`→乐 等效验。
- "谁"升到 `shui` 首位是有意取舍（词频继承"该字最高值"，谁 5840 高于水）：
  打 shui 优先出谁，水仍第 2 位首屏可达。若后续用户反馈排序不适，可考虑
  "补丁读音词频=原键读音最大词频（而非全字最大）"作修正——未做。
- 后续扩充补丁表后必须同步重建 base（发行包词典哈希变化）；
  real.zyct 与 base 现在应用同一补丁表，口径一致。
- 部分超驰 [2026-09-30-polyphone-gap-patch](2026-09-30-polyphone-gap-patch.md)：
  其"构建期应用 import --polyphone"口径扩展为"import 与 build-base 双管线共用
  apply_patch_entries"；补丁表机制/审计/人工把关原则不变，原 note 保留并交叉链接。

## Verification

- `polyphone.rs` 新单测：共享应用函数复制读音且词频继承该字最高 / 跳过已有组合与
  表内重复与词表外字 / 拒绝非标准音节。
- `audit-polyphone --base` 实际输出 460 条缺读（对照 base 实测读数）。
- base 重建后 `zhu-ye-dict lookup` 核对 31 条补丁中可判定的 30 个键**全部命中**
  （shui→谁 第 1、liao→了 第 1、zhuo→着 第 1、yue→乐 第 4 等）。
- 门禁：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
  `cargo test --workspace`（core 327/ime 215/dict 62/settings 107/ui 35 全绿）、
  `git diff --check` 全部通过。
- 部署：`copy-data`（dictionary.zyct rename-swap + .tones）到产品目录 + Roaming
  副本同步；`lookup` 对部署位复验，见 T-129 todos 行与部署记录。
