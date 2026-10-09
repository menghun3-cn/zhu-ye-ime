# Agent Note: 日常高频短语扩充——种子表 + 系词枚举（T-145）

Status: implemented

## Problem

用户询问（2026-10-09）：输入 `ni shi` 应该优先候选「你是」这类日常高频短语
（对齐搜狗），而不是「逆势」；并询问是否需要把日常打出的组合单独汇总成词汇。

对已部署词典（`C:\Program Files\zhu-ye-ime\tsf\dictionary.zyct`）的实测证据：

- `rank nishi` 只有 5 个整词：逆势（2850）、泥湿（2477）、睨视（2477）、
  逆事（2477）、逆施（2477）。「你是」**根本不在基础词典中**。
- 自学习词库（`user_words.json`）救不了：它只记录用户**真正以整词上屏**的词，
  而候选里永远不出现的词也永远学不到（先有鸡还是先有蛋）。

「你是」为何缺失？基础词库三条来源都把这类语法级短语拆散了：CEDICT 是词典、
jieba 分词把「你是」切成 你/是、xdhyc 是字表。整条管线没有任何 n-gram 语料。

## Decision

`build_base` 在 jieba 扩充之后新增**日常高频短语扩充步（T-145，m6.rs）**，
候选来自两个来源：

1. **种子表** `data/patches/daily-phrases.tsv`——人工维护的纯 CJK 短语
   （2-5 字），每行一条，`#` 注释与空行忽略，重复保留首条。覆盖任何语料都
   无法从现有数据生成的对话句式（你是/我是/你是谁/你是不是/怎么办/什么时候/
   没错/好吧/…）。词频 `DAILY_PHRASE_FREQ` = 4200。
2. **系词枚举**——`COPULA_PREFIXES` 白名单（人称/指示/疑问代词与常用副词）×
   「是」，生成 你是/我是/其实是/反正是/…，最长 4 字。词频
   `COPULA_PHRASE_FREQ` = 4000。

两类候选统一用现有 `tables.annotate` 注音（kTGHZ 字级，不手写拼音、无拼写
漂移）。已在 merged 中的词**跳过并保留原词频**（如「是不是」保留 5320）——
绝不覆盖已有词条。频率档高于「逆势」(2850) 等常见短语拼音组首候选，低于
绝对高频单字（是 7160 / 你 6690），故 `shi` 组不受影响。

`BaseStats` 增加 `daily_seed` / `daily_copula` / `daily_skipped`；新增
`zhu-ye-dict freq <词...>` 子命令用于探查 wordfreq 数据源。

## Alternatives considered

**纯 wordfreq 子集。** 否决：`freq 你是` 实测「你是」根本不在 wordfreq 中文
语料（被空格分词拆成 你/是），「我是」等语法短语同样不在，无数据可取。

**语料 bigram 统计（globalvoices / 社交语料）。** 实测后否决：
`globalvoices_zhs.tok.gz` 是按词分词的新闻语料，整份 8MB 语料中相邻单字
对全量仅 97 个，「你是」「我是」一次都没共现；社交源是词表而非成篇文本。
没有任何 n-gram 数据可用于推导短语频率。

**仅种子表、不做枚举。** 否决：覆盖面停留在人工广度；枚举让「X是」族系统化
（免费获得 90+ 条同频率档的系词短语），种子表则对不规则句式保持权威。

**运行期聚合组合而非构建期入词。** 否决：那正是 `user_words.json` 已有的
机制，也正是无法解决首次出现的先有鸡机制；必须先由基础词库承载这些短语，
用户首次上屏后自学习再接手做个人化。

## Consequences

- `nishi` 现在首候选为「你是」(4200)、「逆势」(2850) 第二；`woshi` 首候选
  「我是」(4200) 高于「卧室」(4020)；单字组不变（shi 仍是「是」7160）。
- 基础词库按新增短语增长（首次构建 287,233 → 287,342：种子 22 + 枚举 87
  入包，43 条已在库跳过）。
- S-1 `audit-coverage` 指标与改动前基线完全一致（出候选率 100.00%、首候选
  正确率 89.60%）——扩充不扰动单字排序。
- 运行期 IME 代码零改动：整个修复在数据 + 构建；部署走既有 `copy-data`
  词典链（dictionary.zyct + .tones），无需换 DLL。
- 种子表是后续"日常短语"报告的维护面；扩充它只需重跑 `build-base` +
  `copy-data`。

## Related

- [Candidate ranking static model](2026-09-19-candidate-ranking-static-model.md)：
  运行期排序在拼音组内按词频排序；本 note 是补缺失的高频词条而非改排序模型。
- [User dict persistence](2026-09-19-user-dict-persistence.md)：用户整词
  自学习与提权（D-70/D-71）；本扩充让这些短语可被选中，自学习才能启动。
- [Real dictionary import](../../architecture/2026-09-21-real-dictionary-import.md)
  与 [Dictionary source pins and fetch script](../process/2026-09-28-dictionary-source-pins-and-fetch-script.md)：
  本步加入的构建管线；短语表属于 patches 级输入，不新增 pin 源。
