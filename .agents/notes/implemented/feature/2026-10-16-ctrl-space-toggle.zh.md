# Agent Note: Ctrl+Space 中英模式切换（T-165）

Status: implemented

[English](2026-10-16-ctrl-space-toggle.md) | 中文

## Problem

输入法的中英切换此前只绑定「单独按下并弹起 Shift」（T-114 待定切换状态机）；
用户点名要求增补专属快捷键 Ctrl+Space（「ctrl+空格，改竹叶输入法为切换中英文」）。
注意这与系统级 Ctrl+Shift 输入法语言切换（T-164，已在系统热键配置修复）是
两回事。

## Decision

在 `tsf.rs` 中把 **Ctrl+Space** 视为中英切换，语义与 Shift 切换一致
（空闲/组合态均响应，组合内容保留）：

1. `ctrl_key_down()` 读 `GetKeyState(VK_CONTROL)`；判定抽为纯函数
   `is_ctrl_space(wparam, ctrl_pressed)`（空格 VK 且 Ctrl 按下）便于单测。
2. `OnTestKeyDown` 对 Ctrl+Space 显式返回 TRUE——修饰组合在 `plan_action`
   归为 `None`，不加此判定事件永远到不了 `OnKeyDown`。
3. `OnKeyDown` 在 Shift 待定切换记账之后拦截 Ctrl+Space（并清除待定 Shift），
   执行 `sync_engine(ToggleMode)` + `refresh_candidate_window` 并返回 TRUE
   吃键；无需 TSF 编辑会话（纯状态动作，与 Shift 弹起/语言栏点击 T-100 同路径）。
4. 语言栏提示文案更新为「按 Shift / Ctrl+Space 切换」
   （`lang_bar.rs` `GetTooltipString`）。

## Alternatives considered

- **依赖 plan_action 归类 Ctrl+Space**：带修饰的键在 `plan_action` 已直接
  返回 `None`（仅 Ctrl+数字译文快捷键被特判），事件会落到宿主；必须在
  `OnTestKeyDown` 显式接收。
- **仅在空闲态切换**：Shift 切换本就只在空闲态生效（组合中 Shift 是修饰键）；
  而 Ctrl+Space 用户期望组合进行中也显式切换——组合保留、直接切换，
  与语言栏点击语义（T-100）一致。

## Consequences

- Ctrl+Space 与 Shift 均可切换中英；两种情况组合内容均保留；候选窗与
  语言栏图标经既有模式同步路径刷新。
- 按键被消费（宿主收不到 Space），Ctrl+Space 不会误触发候选提交/空格输入。
- 提示与用户文档同步列出两个快捷键；引擎无新增状态（复用
  `toggle_mode`/`set_mode`）。

## Verification

- 单测「ctrl空格判定」：Ctrl+Space→true；无 Ctrl 的 Space→false；Ctrl+A→false。
- 全部门禁绿：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
  `cargo test --workspace`（ime lib 248、tsf 45、core 352、host_e2e 169）、
  `git diff --check`；verify-translation-pairs 与 verify-agent-notes 全绿。
- 文档同步：需求规格说明书 FR-004、验收标准 FR-004「Ctrl+Space 切换（T-165）」
  行、安装与使用 §5 快捷键表、CHANGELOG [Unreleased]、todos T-165。
- 部署与真机验收（随 T-163 二轮 DLL 一并交付）待进行。
