# Agent Note: 格式候选：数字格式 / v 模式符号 / emoji 队尾追加 (T-061)

Status: implemented

[English](2026-09-30-format-symbol-emoji-candidates.md) | 中文

## Problem

用户四项增强第③项：输入连续的原始数字、符号或 emoji 时，输入法不提供任何格式化候选项——数字串没有日期/金额/电话排版、没有符号快捷入口、没有 emoji 快捷输入。该功能必须与既有候选管线共存：不得回退 T-057 命中率基准（Top1 84.7% / Top3 97.2% / 整句 21.0%），不得干扰正常拼音排序路径。

## Decision

三个独立候选源挂在空闲候选窗之后（`suggestion > digit > v`，FR-029 D-05），一律不参与拼音排序。

**core**（`zhu-ye-core`）：`format.rs` — `format_candidates` 把数字串映射为排版文本（8 位日期 4 式、6 位年月 2 式、4 位年份 1 式、金额千分位+`cn_numeral` 中文读数、11 位电话 2 式、≥5 位纯千分位；超长/非法返回空）；`symbols.rs` — `symbol_group('1'..='9')` 序号 ①-⑨、`x` 数学 ±×÷≈≠≤≥∞％、`h` 标点 ，。！？、；：""——每类一页 9 个；`emoji.rs` — 静态 109 条别名表按别名字节序、`emoji_for` 二分查找；`CandidateSource` 增 `NumberFormat`/`Symbol`/`Emoji` 三变体。

**引擎**（`input.rs`）：
- `digit_buffer`：空闲态数字键进入数字模式并把该字符立即写入文档（digit_append 接受 ASCII 数字与 `.`）；不足 5 位无候选；`digit_active` 期间数字/退格/空格/Esc 全按数字模式语义；任何字母/拼音输入先退出数字模式（`push_composing` 开头 `exit_digit`）。`commit_digit` 清 buffer 并把 `previous_word` 设为格式文本——随后的空闲窗展示该文本的 bigram 联想（D-05）。
- `v_buffer`：空闲态 `v` 冷启动 v 模式（`v_start` 在组合/联想/数字/v 任一活跃时拒绝——`v` 是 nv/lv 合法拼音字符）；`v_code` 输类型码；非法字母（`vi`、`vv`）经 `v_consume` 回退拼音；`v_backspace` 回退类型码、Esc 退出；v 模式内数字=选择符号（越界放行、不吞键），空格=选第 1 个符号。
- emoji：拼音整串==别名时，在俚语组之后队尾追加 `Candidate{score: i64::MIN, source: Emoji}`——永不参与排序、与普通词并存。
- 候选窗：`candidate_ui_view` 同一时刻只显示一个来源——联想（拼音空）> 数字（pinyin_hint=数字 buffer）> v（pinyin_hint=类型码）；拼音候选仅在组合非空时出现。

**TSF**（`tsf.rs`）：
- `KeyAction` 扩 8 变体：`Dot`、`BufferDigit(char)`、`DigitBackspace`、`SelectAndReplace(usize)`、`VStart`、`VCode(char)`、`VConsume(char)`、`VBackspace`。
- `plan_action` 开头自动退出守卫：`digit_active` 且按键不属于数字模式保留键时 `exit_digit`；v 模式同理。数字模式内：空格=选当前行（`SelectAndReplace`）、退格=`DigitBackspace`、Esc 退出；v 模式内：空格=选符号 0、退格=`VBackspace`、Esc 退出；Enter 等非保留键先退出模式再放行。`plan_digit` 把数字键分派到 v 模式/数字模式/联想/缩写组合/数字模式各语义。
- `BufferDigit` 经 `finish_composition` 直接提交单字符（无组合）；`SelectAndReplace` 走 `replace_last_chars`——`GetSelection` → clone range → `ShiftStart(-len)` → `SetText` → `Collapse(TfAnchor(1))` → `SetSelection`（`TF_SELECTION { range: ManuallyDrop::new(Some(range)), style: TF_SELECTIONSTYLE { ase: TF_AE_END, fInterimChar: BOOL(0) } }`）——恰好替换已追加数字 buffer 的 UTF-16 长度；`DigitBackspace`=`replace_last_chars(context, ec, 1, "")`；`VConsume(c)` 开组合 `"v{c}"` 回拼音路径；`V*`/`Dot` 为纯引擎状态动作（sync + refresh）。
- `.` 由 `VK_OEM_PERIOD`/`VK_DECIMAL` 映射为数字模式内的 `BufferDigit('.')`（金额小数）；数字模式外放行宿主。

**host-e2e** `--m9`：15/15 断言（验收标准 9.1-1/2/4-11：日期 4 式、替换长度 8、无残留、不足 5 位无候选、金额、电话、v1/vx/vh、vi 回退、emoji 队尾、无别名不追加）。

## Alternatives considered

**先暂存后一次插入（备选 A）。** 数字先攒 buffer、选中时一次插入替换文本。落选为主路径：用户确认的"边输边上屏"对齐主流输入法，且 TSF 替换链（GetSelection → ShiftStart → SetText → Collapse → SetSelection）整体运行在单个编辑会话内，与既有 commit-no-comp 路径同模式。备选 A 保留在方案文档中，作为 VM 实测替换链失败时的切回方案。

**数字/emoji 候选参与拼音排序。** 落选：排序列内任何数字候选都会挤掉词条并回退 T-057 基准；emoji 以 i64::MIN 保证"可见但不可排序"的并存。

**v 模式在任何状态启动。** 落选：`v` 是 nv/lv 拼音的合法尾字符；只有空闲态（组合空、无联想、无数字模式）才拦截。

**v1..v9 作为缩写键进组合（u1s1 同款）。** 落选：与 D-02 v 模式编号冲突；沿用 v 模式编号语义。

## Consequences

- 数字与 emoji 候选不进入任何拼音排序路径——本次变更后 T-057 复跑与基准完全一致（Top1 84.7% / Top3 97.2% / 整句 21.0%）。
- 提交格式数字文本会把 `previous_word` 置为该文本，空闲窗随之切到该文本的 bigram 联想（联想 > 数字优先级）。
- 数字模式内 `.` 键被吞作小数点；模式外按键原样放行——数字模式活跃时用户无法用句号结束句子（先 Esc/字母退出）。
- 组合态数字键语义不变（含 996/u1s1 等数字缩写键）。
- emoji 表首批 109 条、静态二分；纯数据批次 T-062 铺至 300+。
- shui 候选覆盖项（选项 A）保持待办，与场景7 无交集。

## Deferred

VM 交互验收（验收标准 9.1，vm-accept-sop）：各用例的引擎侧已由 host-e2e `--m9` 断言；VM 上的替换链与组合实测在本 PR 合入后执行，结果回填验收标准 §9.5。

## Related

- 设计：[docs/格式候选设计.md](../../../docs/格式候选设计.md)（T-060）
- 需求：[docs/需求规格说明书.md §13](../../../docs/需求规格说明书.md)（T-060）
- 验收：[docs/验收标准.md §9](../../../docs/验收标准.md)（T-060）
- 任务：[docs/todos-list.md](../../../docs/todos-list.md) T-061 / T-062
