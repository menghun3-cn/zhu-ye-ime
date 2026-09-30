# Agent Note: 命中率评测集与首轮基准

Status: implemented

[English](2026-09-30-hit-rate-eval-baseline.md) | 中文

关联笔记：[ime-experience-optimization](../process/2026-10-02-ime-experience-optimization.md)
（描述本评测所衡量的 M7 功能与已接受的 Top1/Top3 目标）、
[cli-benchmark-metrics-and-acceptance](../testing/2026-09-21-cli-benchmark-metrics-and-acceptance.md)
（时延口径：本笔记管**准确率**，那篇管**速度**）、
[polyphone-gap-patch](../process/2026-09-30-polyphone-gap-patch.md)
（命中率的词典侧：其补读音在排序列中同等参与评分）。

## 背景

此前的排序类改动（T-044 静态排序、T-053 简拼、T-054 模糊音/纠错、T-055 整句 beam）
都靠单点用例（`rank`、host-e2e 断言）验证，没有**规模化、可复现、可对比**的命中率
口径——"这个改动到底有没有提升打字体验"一直停留在抽查层面；用户要求的四项增强
（上下文联想、数字/符号/emoji 候选、混输、领域自动）也没有可守住回退的基线。

## 决策

建立确定性命中率评测基础设施，并记录首轮基准：

1. **评测集（`data/eval/`，入库）**——词样本由生成器产出，整句样本人工把关：
   - `zhu-ye-dict eval-set <CC-CEDICT> <wordfreq> <out.tsv> [--top N]`：以与真实词典
     导入完全一致的清洗口径解析 CEDICT（纯 CJK、音节数==字数、调符/数字调归一化到
     引擎无调键域、同词多行保留 CEDICT 首读），与 wordfreq 取交集，按词频降序
     （同频按词形字典序，保证确定性）截取前 N。输出 `词<TAB>拼音<TAB>词频`。
   - `data/eval/sentences-100.tsv`：100 句人工编写的日常句，`期望句<TAB>无调连拼`，
     全部 ≥4 音节，避免被 beam 的短串防抖（最少 3 音节）挡掉。
2. **判定（`zhu-ye-cli eval <词典> <词样本> [整句样本] [--out-miss 文件]`）**——
   词样本走与引擎一致的主候选路径（`generate_candidates` + `StaticRankingModel`，
   无前词、用户词典置空）：Top1 = 首候选==目标词，Top3 = 目标词在前 3 位；
   Top1 另按词频分档（≥1M / ≥100k / ≥10k / 其余）报告，暴露低频退化。
   整句走 `sentence_candidates`（引擎只是把其首元素放到列表最前，整句路径的
   Top1 即评测的 Top1）。Top1 未命中与整句未命中写入 MISS 文件
   （`词`/`句` 类型行）。
3. **确定性是性质而非希望**：TSV 固定行序遍历、用户词典置空、与引擎同权重；
   两次连续运行输出一致且 MISS 文件 MD5 相同，实测验证。

## Alternatives considered

- **复用 host-e2e 断言当评测。** 否决：通过/失败计数无法优雅退化、没法按词频分档，
  且每加一条都是手写用例而不是数据驱动评测集。
- **直接从词典文件抽取词样本。** 否决（加载器无遍历 API）：loader 只暴露
  `lookup`/`lookup_prefix`，为开发期工具给 core 加枚举接口会改核心契约；
  而且 CEDICT∩wordfreq 比"词典自己排最高"更贴近用户真实输入。
- **用 kTGHZ 逐字自动转写整句拼音。** 否决：多音字需要句子语境（地/了/得），
  逐字表选不出；100 句人工拼音可审计、可订正，diff 里能逐行 review。
- **判定候选走引擎全路径**（简拼+纠错组）。否决（v1）：词样本几乎总有非空主候选
  组，整句路径直接判整句候选；把 M7 全部组混进指标会糊化特性归因。
  未来可加 `-all-groups` 模式。

## Consequences

- `real.zyct`（2026-09-30 构建、含 T-056 补丁）首轮基准：
  **Top1 84.7%、Top3 97.2%**（N=2000），分档 ≥1M 100% / ≥100k 91.0% /
  ≥10k 87.9% / <10k 83.0%；整句 **21.0%**（N=100）。两次运行字节级一致。
- 数字解读：Top3 已很强；Top1 84.7%，且低频档（83.0%）贡献大部分 MISS——这正是
  后续增强（上下文联想、数字/符号候选）该打的主战场。整句 21% 量化了 beam 对任意
  句子的首句短板（多为同音字/词选择：迟到→吃到、晚上→玩上），是联想/补全必须
  超越的基线。
- 规则：此后每个排序类改动（T-058 上下文联想、T-059 数字/符号候选、混输、领域
  自动）必须跑同一 eval，Top1/Top3 相对本表不得回退；数字回填
  `docs/命中率评测设计.md` §5。
- 新增测试：`zhu-ye-dict` +4（清洗/交集/同词去重/top 截断/TSV 渲染/音节数不符
  拒绝）、`zhu-ye-cli` +4（行解析、分档边界）。`data/eval/` 文件是数据不是代码，
  按真实使用样本 review。
- MISS 文件进 `target/`（不入库）：是证据，可复现的真相在 `data/eval/` 评测集。
