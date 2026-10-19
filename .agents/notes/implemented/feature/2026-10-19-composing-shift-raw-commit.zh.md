# Agent Note: 组合态 Shift 上屏拼音原文（T-172）

Status: implemented

[English](2026-10-19-composing-shift-raw-commit.md) | 中文

## Problem

搜狗输入法在候选框已出内容时按 Shift，会切到英文模式并**把拼音原文直接上屏**
（用户原话：「当输入时，候选框已经出来内容，此时按键shift时，应该直接把输入
的内容直接上屏，和搜狗的表现一样」）。竹叶输入法此前相反：组合态下 Shift 只是
修饰键（T-114 待定切换状态机），组合态的 Shift 完全没有绑定任何上屏动作。

## Decision

引擎处于组合态（`engine.is_active()`）时，Shift 按下被**改写为
`KeyAction::Enter`**，由常规编辑会话路径上屏**拼音原文**（`nihao` 上屏
`nihao`，不是首候选），随后切换到英文模式。

1. `shift_down_action(combinational, action)`（`tsf.rs`）是纯改写规则：组合态
   → `Some(KeyAction::Enter)`；空闲态 → 原动作不变。
2. `OnKeyDown` 施加改写后**继续走常规 `RequestEditSession` 流程**，文本经
   `commit_text` + `finish_composition` 写入文档，与回车键完全同路。
3. 编辑会话结束后执行 `sync_engine(ToggleMode)` 并刷新候选窗，同时联动系统
   键盘布局（T-168）。

## Alternatives considered

- **上屏首候选而非拼音原文**：用户明确纠正（「按space是第一个候选词上屏，但是
  输入例如nihao，用shift切换中英格式时，期望的是nihao拼音上屏，而不是'第一个
  候选词上屏'」）。搜狗的语义是回车语义（原文）。
- **在按键回调里直接调 `engine.handle_space()`**：引擎状态会变，但**文本不会
  写进文档**——本输入法只在 TSF 编辑会话内提交文本（`apply_action` →
  `commit_text` → `finish_composition`）。最初实现即如此，用户反馈「nihao并没有
  上屏」。基于该直接观测否决。
- **只切模式不上屏**：组合内容会悬空或丢失，且与搜狗行为不一致。
- **用 `KeyAction::Space`**（首候选）：除文本不入档外，与用户期望语义冲突。

## Consequences

- 组合态按 Shift 现在上屏拼音原文并切英文；该行为在同批次由用户确认。
- 空闲态 Shift 行为不变：仍进入待定切换、弹起时生效（T-114）。
- 组合态的 Shift 按下被消费，宿主不会收到这一次 Shift；之后用英文输入大写
  需要重新按一次 Shift。
- 引擎无新增状态：复用既有 `is_active()` / `handle_enter()` / `toggle_mode()`。

## Verification

- 单测「组合态shift改写为原文上屏空闲态不变」：组合态→`Enter`；空闲态→
  `ToggleMode`；空闲态无动作→`None`。
- 单测「回车提交拼音原文而非首候选」：组合 `nihao` 的 `preview_enter()` 为
  `nihao`（原文而非「你好」）。
- 全部门禁绿：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
  `cargo test --workspace`、`git diff --check`。
