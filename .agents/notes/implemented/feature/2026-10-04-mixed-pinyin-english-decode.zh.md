# Agent Note: 中英混合整句解码 `python代码`（T-086）

Status: implemented

English | [中文](2026-10-04-mixed-pinyin-english-decode.zh.md)

## Problem

FR-050（M13）覆盖场景 6"混合串"：用户在同一组合串里混打代码词/缩写/产品名与
拼音，如 `python代码`/`API接口`/`iPhone价格`。TSF 路径上组合串**永远是纯
ASCII**（`pythondaima`、`APIjiekou`），因为只有字母/数字/格式键进入组合串；
验收示例里的汉字是**解码结果形态**而非输入。既有独占路径（全拼 T-007、缩写
FR-017、纯英文 FR-030、前缀候选 T-029）各自假设整串属于单一层次，因此
`pythondaima` 今天退化为围绕 `py…` 的前缀候选。同时判别规则不得劫持 `nihao`
（纯拼音）、`yyds`（缩写）、`python`（纯英文）或 `xie` 类双可切串
（风险表 §14.8）。

## Decision

在 `crates/zhu-ye-core/src/mixed.rs` 实现确定性、离线、无统计模型的判别门 +
解码器，接入 `input.rs refresh_candidates`：位于 `detect_format`（FR-031
`@`/`www.`/http）return **之后**、`generate_prefix_candidates`（T-029）
**之前**。D-56 保持：不引入统计/语言模型。

**触发**（`is_mixed_input`，纯函数；段 = `[A-Za-z]+` 游程与其余游程）：

- 英文/缩写成分：字母段长度 ≥2 且含大写字母（cased，`API`/`iPhone`），或整段
  不可拼音切分且完整匹配英文词（`en_is_word`：静态词表前缀查询 top1 小写化==
  探测串；`python`）。
- 拼音成分：非 ASCII 段（中文主体）或可完整拼音切分的字母段（`daima`）。
- 同段混合：`en_prefix_then_pinyin`——最长英文词前缀 + 可切后缀
  （`pythondaima`）。
- 触发 iff `strong_mixed || (pinyin_body && (cased || english)) ||
  (cased && pinyin_letters)`。纯 `nihao`、`yyds`、`python`、`代码`、`xie`、
  `xiedaima`、`python123`（数字段不算拼音段）、小写可切 `api接口` 均不触发；
  `daimapython`（同段拼音在英文前）为记录的放弃项（§14.3.5 + 风险表）。

**解码**（`mixed_candidates`，按字母段顺序）：① 整段完整英文词 → 该词（原形
大小写）；② 最长英文词前缀 + 可拼音切分后缀 → 英文候选 + 拼音段 top1
（`generate_candidates`，T-007）；③ 最长拼音前缀（可尾随完整英文词）；
④ 英文词前缀 + 剩余递归（`pythonxyz` → `python` + `xyz` 字面）；⑤ 字面保底——
段永不丢失输入文本。"命中"恒指**完整英文词**：§14.3.2"未命中逐前缀减一重查"
的递减目标是词，所以 `pyx` 不会回退到非词前缀 `py`。英文候选在 `en.zyen`
已加载时来自文件（T-085），否则回退 M9 静态表（与 FR-030 双源口径一致）。

**输出**：`[整句候选] + 各段最优候选`（段出现顺序）。整句候选 = 各段最优解码
的**拼接**（`pythondaima` → `python代码`；`APIjiekou` → `API接口`），
`source = Mixed`、`score = i64::MAX` 置首；分段候选随后（`python` / `代码`），
每段取 top1（"最优候选组合"，§14.3.3）。确定性：无随机、无模型，同输入同输出。

**引擎集成**（`input.rs`）：混合分支清 `cached_translation_candidates` /
`selected_on_page`、clamp 页码后返回（镜像格式分支）；上屏按候选 `text`
提交，整句候选未命中时上屏=用户所键入文本。FR-031 因 `detect_format` 先
return 而保持优先级。

**工具**：`zhu-ye-dict mixed-bench`（22 组样本 ×10 万次，预热不计）2026-10-04
release 实测：中位 3.9µs、P99 40.7µs、最大 329.7µs——验收 ≤1ms（§14.2）已回填。

## Alternatives considered

- **含大写字母段整体按英文处理**：丢失拼音后缀（`nihaoAPI` → 仅 `API`），且
  误解 `XPdiannao` 类段；否决，采用有序 ①–⑤ 策略。
- **对称支持拼音在英文前（`daimapython`）**：搜索空间翻倍且不在验收集内；
  记为放弃项。
- **英文段每段多候选**：§14.3.3 明确"各段最优候选组合"（每段 top1）；top1
  让候选行保持安静且确定。
- **整句统计/语言模型重排**：D-56 否决（离线、确定性、无模型）；顺序=段序。
- **非 ASCII 触发输入**：core 级接受（中文主体=拼音段），但 TSF 不可达；
  ASCII 形态才是真实路径。

## Consequences

- `CandidateSource` 新增 `Mixed` 变体；代码库无 exhaustive match（仅
  host-e2e 有 `==` 比较），UI 按 vec 顺序渲染、不加新标签。
- 大小写有意敏感：`api接口`（小写可切）留守常态路径，`API接口` 触发；该
  不对称记录在验收测试与 §14.3 风险表。
- `daimapython` 与 `pythonxyz` 尾部经字面保底而不丢输入，但不出拼接式中段。
- 非触发输入下 T-029 前缀路径与其余层次零改动（eval 复跑一致：Top1 84.7% /
  Top3 97.2% / 整句 21.0%，2026-10-04）。
- 确定性/段全覆盖的属性测试后置 T-089（proptest），见下方链接。

相关记录：[M9 英文候选 + 邮箱/网址格式（FR-031 优先级与静态表回退）](../../implemented/feature/2026-10-02-english-candidates-and-email-url-formats.md)、
[T-085 en.zyen 文件（双源）](../../implemented/feature/2026-10-04-en-wordbook-zyen-v1.md)、
[全拼切分（T-007，拼音尾部复用）](../../implemented/feature/2026-09-19-full-pinyin-segmentation-core.md)、
[前缀候选（T-029，位于混合分支之后）](../../implemented/feature/2026-09-24-prefix-candidates-incomplete-segmentation.md)。
