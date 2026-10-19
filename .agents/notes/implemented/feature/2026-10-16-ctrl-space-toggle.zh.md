# Agent Note: Ctrl+Space 中英模式切换（T-165）

Status: implemented

[English](2026-10-16-ctrl-space-toggle.md) | 中文

## Problem

输入法的中英切换此前只绑定「单独按下并弹起 Shift」（T-114 待定切换状态机）；
用户点名要求增补专属快捷键 Ctrl+Space（「ctrl+空格，改竹叶输入法为切换中英文」）。
注意这与系统级 Ctrl+Shift 输入法语言切换（T-164，已在系统热键配置修复）是
两回事。

## Decision

**Ctrl+Space** 切换中英模式。切换判定发生在 **Space 的弹起（KEYUP）事件**，
而非按下：

1. `OnKeyDown(VK_CONTROL)` 置位 `ctrl_down` 并清除「Ctrl 刚弹起」时间戳；
   `OnKeyUp(VK_CONTROL)` 清除 `ctrl_down` 并记录 `ctrl_up_at`。
2. `OnKeyUp(VK_SPACE)` 计算 Ctrl 最近一次弹起的时刻差，套用纯函数判定
   `is_ctrl_space_up(ctrl_age_ms, ctrl_down, intervened)`（`tsf.rs`）：当
   Ctrl 按住期间**没有其它键**、且「Ctrl 仍按住」或「Ctrl 在
   `CTRL_SPACE_UP_WINDOW_MS`（200 ms）内刚弹起」时判定成立。
3. 判定成立即执行 `sync_engine(ToggleMode)` + `refresh_candidate_window` 并
   吃掉按键。`ctrl_intervened` 标志（Ctrl 按住期间出现任何非 Space 键即置位）
   防止 `Ctrl+C` 之后按空格误切模式。
4. 200 ms 窗口是**快速连打**能生效的关键：用户反馈「有时候感觉这俩按键连着
   快一些的时候未触发」，而同速度在搜狗可切换——该窗口覆盖「Ctrl 先松、
   Space 后松」的时序。
5. 语言栏提示文案为「按 Shift / Ctrl+Space 切换」（`lang_bar.rs`
   `GetTooltipString`）。

## Alternatives considered

- **用 `GetKeyState` / `GetAsyncKeyState` 读修饰键状态**：哨兵日志实证二者在
  TSF 键回调内恒为「未按下」（IME 键回调与原始输入状态不同线程），无法判定
  组合键。基于直接观测否决。
- **`PreserveKey(Ctrl+Space)` + `OnPreservedKey`**：注册成功，但系统把该保留
  组合派发给先注册者——与搜狗共存时竹叶的回调永不触发。基于直接观测否决。
- **托盘侧 `RegisterHotKey(Ctrl+Space)` 桥接**：确实能触发，但属于全局独占
  占用，会破坏用户正在用的搜狗。否决。
- **托盘低层键盘钩子 + 合成 F24**：钩子从未观测到注入的按键，且引入特权组件。
  否决。
- **在 Space 按下时判定**：系统输入法切换层会吃掉 Ctrl+Space 的按下事件，
  IME 收不到（哨兵日志：Ctrl down、Space **up**、Ctrl up 都能到达，Space down
  不能）。只有弹起路径可观测。
- **要求 Space 弹起时 Ctrl 仍按住**（最初实现）：快速连按时用户先松 Ctrl 再松
  Space 即失效。200 ms 窗口是在不误伤后续空格的前提下兼顾连打的折中。

## Consequences

- Ctrl+Space 与 Shift 均可切换中英；两种情况组合内容均保留；候选窗与
  语言栏图标经既有模式同步路径刷新。
- Space 键在弹起时被消费，Ctrl+Space 不会误触发候选提交/空格输入。
- 引擎新增三个状态字段（`ctrl_down`、`ctrl_up_at`、`ctrl_intervened`）；模式
  切换本身复用 `toggle_mode`/`set_mode`。
- Ctrl 弹起后 200 ms 内（且期间无其它键）按空格也会切换模式——这是有意
  行为，且被窗口长度限定。

## Verification

- 单测「ctrl空格弹起判定」：Ctrl 按住→true；Ctrl 弹起 120 ms→true（连打）；
  弹起 300 ms→false；无 Ctrl→false；Ctrl 期间有干预键→false。
- 全部门禁绿：`cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、
  `cargo test --workspace`、`git diff --check`。
- 真机用户验证：「ctrl+space已经支持切换中英模式」（2026-10-19）。
- 哨兵实证（TSF 调试日志）：`zhu-ye: ctrl+space ToggleMode (space-up)`。
- 已部署：`zhu_ye_ime_C6ADEA46.dll`（连打窗口批次）、后续
  `zhu_ye_ime_596BD111.dll`（T-172 批次）均指针切换、ctfmon 重启。
- 文档同步：需求规格说明书 FR-004、验收标准 FR-004「Ctrl+Space 切换（T-165）」
  行、安装与使用 §5 快捷键表、CHANGELOG [Unreleased]、todos T-165。
