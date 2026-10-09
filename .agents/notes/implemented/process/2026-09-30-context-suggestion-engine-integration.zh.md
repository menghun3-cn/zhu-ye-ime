# Agent Note: 上下文联想：引擎承接

状态：已实现

[English](2026-09-30-context-suggestion-engine-integration.md) | 中文

相关笔记：
[context-suggestion-bigram-successors](../process/2026-09-30-context-suggestion-bigram-successors.md)
（本笔记引擎状态消费的检索层：同一份 `suggestion_candidates` 契约、同一列表形态——整词在前、短语在后）与
[ime-experience-optimization](../process/2026-10-02-ime-experience-optimization.md)
（所属 M7 功能族；交互沿用候选窗既有约定）。

## Problem

检索层（T-058）已能产出后继联想，但引擎与 TSF 没有**空闲候选窗**状态：候选窗只在组合串
非空时显示，上屏一清空即隐藏。用户场景 5（上下文联想）的要求是：上屏一个词后候选窗
留在屏幕上展示 bigram 后继；按数字键直接把所选联想上屏；输入任意字母退出联想态、
回到正常拼音路径。

## Decision

在 `InputEngine` 内新增 `suggestion` 状态，让既有候选窗/路径机制把它当普通
（空组合）视图渲染：

1. **引擎状态**：`InputEngine.suggestion: Vec<String>`。`commit_candidate` 在更新
   `previous_word` 后立即刷新——每次上屏都重算（落实检索层笔记"每次提交后必须重算"）。
   `commit_raw` 与 `handle_enter` 清空（`previous_word = None`）。
2. **`suggestion_active()`**：组合串为空**且**列表非空。任何字母输入（`push_composing`）
   清空列表，联想态在下一词第一个字母自然退出。
3. **从空闲窗上屏**：`commit_suggestion(text)` 把所选词作为**新前词**上屏（连续联想链：
   明天→早上→…）并重算列表；刻意不调用 `record_user_word`——联想候选没有可靠音节
   映射，记录会污染用户词库。
4. **视图/TSF**：联想态 `candidate_ui_view` 返回组合串 `""` + items=联想列表
   （`CandidateSource::Suggestion`，UI 不新增标签）；`refresh_candidate_window` 只在
   组合串**且** items 都为空时隐藏，空闲窗因此保持显示；`candidate_window.update`
   同步放宽该条件。空组合的定位走 `selection_placement`（`GetSelection` 取文档插入点、
   用 range 顶端），替代组成区坐标。
5. **联想态键语义**：数字 1..N → `Select` 对应下标联想条目，越界数字放行宿主
   （不吞键、不上屏空串）；空格上屏当前选中行（默认第 0 行）；Esc 清列表并关闭
   窗口（保留 `previous_word`）；Enter / Backspace / 翻页 / 上下键放行宿主——
   联想窗是被动辅助，不得接管换行、删除等宿主语义。

## Alternatives considered

- **为空闲窗造假组合模式**（占位组合串）。拒绝：把联想拖进组合生命周期——TSF 会对
  幻影 range 跑 `SetText`/`EndComposition` 与编辑会话；且联想语义（数字选、字母退出）
  与组合语义（数字追加或选候选）本就不同，两者必须分离。
- **联想上屏走 `commit_raw`**（会清掉 `previous_word`）。拒绝：连续联想链要求已上屏
  的联想词成为下一个前词；清掉会断链。
- **联想态吞掉 Enter/Backspace**。拒绝：宿主语义（换行/光标前删除）优先；窗口只
  响应数字、空格、Esc 与字母输入。
- **把空闲窗做成 TSF 临时组合承载**。拒绝：为零收益增加组合生命周期——引擎状态 +
  空组合视图已覆盖渲染、定位与上屏，无需幻影组合。

## Consequences

- `zhu-ye-ime` 104 测试全绿（联想 +5：上屏联想+数字选择续联、字母退出、越界数字与
  Enter 放行+Esc 关闭、空格选选中行、无 bigram 引擎退化为不联想）。workspace 387
  全绿；fmt / clippy `-D warnings` / diff 检查干净。
- `host-e2e --m8` 真实词典 7/7：今天→的 首条、整词在前短语在后、数字选择以"的"续联、
  Esc 关闭、字母退出。
- 命中率基准契约仍不受影响：联想从不参与排序（显示时拼音为空），T-057 评测无需复跑。
- **VM 交互验收通过**（2026-09-30，vm-accept-sop 流程，Windows Server 2019 验收 VM）：
  TSF 日志实证上屏「今天」→ `cand-show items=8 first=的 (Suggestion)`；按数字 1 →
  `Select(0)` → `commit-no-comp 的`（无组合直接插入）；续联「的」→ `items=8 first=人`；
  输入 `n` 退出联想回组合（`cand-show items=0`）。三张截图 MD5 互异，蓝底面板像素
  （s1=2444 / s2=2532，对照 s3=959 组合页眉条）区分完整联想窗与仅页眉状态。

## 联想行显示带调音标（T-135，用户反馈"联想词没显示音标"）

用户 2026-10-09 反馈：音标模式下，词汇上屏后联想出来的词没有显示音标。
根因：`candidate_ui_view` 联想分支构建 `CandidateUiItem` 时 `pinyin` 与
`pinyin_tone` 固定置空（联想只存 `Vec<String>` 词文本）→ 候选窗 `show_pin`
判定失败，每个联想行都没有拼音行。

修复（T-135）：联想分支改填 `pinyin_tone: self.tone_spaced(word)`——
ToneMap 词级带调优先、字级逐字拼合兜底（字级表覆盖常用字，两词短语也能
拼全）；含缺音字时留空，候选窗回退"无拼音行"，不硬造拼音。`pinyin`
（无调）联想项保持空：`pinyin_tone` 命中时 paint 直接用带调（T-112 批四
逻辑）、未命中时拼注回退原文本 → 拼音行隐藏，两条路径都正确。新测试覆盖
无带调表全空基线、词级/字级/短语拼合、缺音字留空。

## Supersession 检查

无既有活跃笔记被取代：本笔记是消费检索层契约的引擎层（上方已交叉引用）；
hit-rate-eval-baseline、ime-experience-optimization 保持活跃。检索层笔记的后果
（"引擎可缓存空闲窗结果，但每次上屏后必须重算"）在这里按原文实现。
