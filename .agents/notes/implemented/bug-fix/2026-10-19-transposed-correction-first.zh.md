# Agent Note: 错序输入的校正词置顶与校正显示（T-172）

Status: implemented

[English](2026-10-19-transposed-correction-first.zh.md) | 中文

## Problem

快速输入常把两个相邻字母打反（`zhegnque` 而非 `zhengque`）。用户期望输入法
识别该错位并把校正后的候选优先显示（「输入zhegn'que也默认校正优先预判显示候选词
zheng'que，这种通常是用户输入太快导致的，应该可以智能识别该输入」）。错序容错
能力本身存在（T-115），但实际不可见：

1. 校正组的触发条件是「主候选不足一页」。真实词典下 `zhegnque` 先命中前缀补全
   路径（`zhe` → 这/者/遮…）把一页填满，于是该条件不成立，**整组校正候选根本
   不生成**——用户看不到「正确」。
2. 即使生成了，校正组也是**追加**在主候选之后，校正词被挤到第一页之外。
3. `transposed_candidates` 内部按位置顺序枚举换位变体，候选额度被靠前的无用
   变体吃光：`zhegnque` 只产出「则/咋/责/泽…」（这些变体不可切分），根本没轮到
   真正可切分的 `zhengque`。

## Decision

三处协同改动让校正词可见且置顶：

1. **core 变体顺序**（`candidate.rs`）：换位变体拆成**可完整切分**（真正的修正
   目标，如 `zhengque`）与**不可切分**（只能用于前缀补全）两类；可切分变体先
   枚举，额度不会再被噪声抢先占满。
2. **触发条件**（`input.rs` `append_transposed_group`）：校正组在输入**不可完整
   切分**（`segment_all` 为空）时即生成——页数门被取消。生成后用 `prepend_group`
   **置顶**，校正词成为首候选（对齐搜狗）。
3. **校正显示**（`input.rs` `composing_display`）：当首候选是
   `CandidateSource::Corrected`、其正确拼音是当前输入的**字母重排**（等长且字符
   多重集相同）且可切分时，页眉显示校正后的拼音并按音节加分隔符
   （`zhegnque` → `zheng’que`）。重排判定避免退格后残留旧候选劫持页眉
   （`nihao` 退格成 `niha` 不应显示 `ni’hao`）。

## Alternatives considered

- **保留页数门、只调整变体顺序**：前缀路径在判定前已把页面填满，需要校正的
  输入恰好是被挡掉的那批。原来的门衡量错了对象——应看输入是否可切分，而非页面
  是否填满。
- **校正组仍追加、给校正词单独快捷键**：校正词仍不在屏上；搜狗是直接置顶。
- **对任何校正候选都显示校正拼音、不做重排校验**：退格后旧候选的拼音与输入不
  对应，页眉会显示用户没打过的词。改用字符多重集校验兜住。
- **仅对没有前缀命中的输入生成校正**：用户输入 `zhegnque` 本来就想要校正结果，
  即使存在 `zhe` 前缀词。否决。

## Consequences

- `zhegnque`（以及带分隔符的 `zhegn'que`，其组合键是去噪后的 `zhegnque`）现在
  首候选即「正确」，页眉显示 `zheng’que`。
- 校正组对所有不可切分输入都会生成，类错序输入的候选列表以校正词开头而非前缀
  补全词；可完整切分的输入行为不变。
- 校正输入的页眉文本会变化；底层组合键仍是去噪拼音，上屏学习与退格仍按用户
  实际敲入的字母工作。
- `transposed_candidates` 优先可切分变体，既有错序输入的校正组顺序随之改变
  （校正词在前，而非首个枚举到的换位）。

## Verification

- 单测「错位串校正词置顶且页眉显示校正形式」：首候选为「正确」且来源为
  `CandidateSource::Corrected`；`composing_display()` 为 `zheng’que`。
- 单测「带分隔符错位串zhegnque出正确」：带分隔符输入同样产出「正确」校正候选
  （组合键为去噪后的 `zhegnque`）。
- 真实词典端到端测试「真实词典下错位串zhegnque出正确并显示校正形式」
  （`#[ignore]`，需部署位词典
  `C:\Program Files\zhu-ye-ime\tsf\dictionary.zyct`）：首候选「正确」、页眉
  `zheng’que`。
- 全部门禁绿：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
  `cargo test --workspace`、`git diff --check`。
