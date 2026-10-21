# Agent Note: 托盘菜单须先夺取前台，点外部才关闭

Status: implemented

[中文](2026-10-21-tray-menu-outside-click-dismiss.zh.md) | English

## Problem

用户需求：右键竹叶输入法的托盘弹出菜单后，**点击非菜单区域应直接关闭菜单**。

真机症状：菜单弹出后点外部，菜单不消失（需按 Esc 才关）。

## Decision

### 机制：`TrackPopupMenu` 的「点外部关闭」依赖拥有者窗口在前台

弹出式菜单的「点外部关闭」由 `WM_CANCELMODE` 实现 —— 而该消息**只在拥有者窗口已在前台**时才会送到菜单窗口。拥有者窗口不在前台时，系统认为输入焦点在别的应用，菜单收不到取消消息，因而既不关闭也**收不到鼠标点击**。

托盘窗口原先是 `WINDOW_STYLE(0)` 创建的**零尺寸普通窗口**（无 `WS_POPUP`、无 `WS_VISIBLE`），从不激活 ⇒ 永远不在前台。

两处修复：

**① 窗口样式改为 `WS_POPUP`。** 托盘程序用它接收通知区回调已是通行做法；更关键的是 `WS_POPUP` 窗口**参与前台切换**，而零尺寸普通窗口不参与 —— `SetForegroundWindow` 对后者无效。实测加 `WS_POPUP` 后前台确实变成 `ZhuYeTrayHostWindow`。

**② 用 `AttachThreadInput` 绕过 Windows 的前台锁定。** 前台锁定的规则是：只有当前前台进程、或「最后输入接收者」进程才能设置前台。托盘进程在通知区图标被点击时通常算「最后输入接收者」，但**不保证**（例如由自动化/脚本触发、或前台刚切换时）—— 一旦 `SetForegroundWindow` 失败，菜单就拿不到输入焦点。

`AttachThreadInput` 把托盘线程与当前前台线程的输入队列「粘合」，使本线程在系统看来与前台线程等价，从而**可以合法地设置前台**。这是 Win32 社区处理该限制的标准手段。

实现见 `force_foreground()`：取当前前台窗口 → 取其线程 → `AttachThreadInput(current, foreground, true)` → `SetForegroundWindow(window)` → `AttachThreadInput(current, foreground, false)` 解粘。

**③ 菜单收起后把前台还给用户原来的窗口。** 记录弹菜单前的 `GetForegroundWindow()`，菜单返回后恢复 —— 托盘窗口是零尺寸的隐形窗口，让它占着前台会打断用户正在做的事。

## Alternatives considered

**用 `TrackPopupMenuEx` 或加 `TPM_` 标志。** 否决：问题不在菜单 API，而在**拥有者窗口没在前台**。`TrackPopupMenuEx` 不改变这一点。

**自绘菜单（像搜狗那样用独立窗口 + 自己处理外部点击）。** 否决：代价远大于收益（要自己做命中测试、键盘导航、焦点管理、主题适配）。`WM_CANCELMODE` 这条路本来就是 Windows 提供的标准机制，只是需要满足前置条件。

**只加 `SetForegroundWindow`，不处理前台锁定。** 否决：`SetForegroundWindow` 的成功率依赖「最后输入接收者」这一不稳定前提，失败时症状**完全相同**（菜单不关、菜单项也点不动）—— 属于把问题留在概率上。`AttachThreadInput` 让它成为确定行为。

**保留 `WINDOW_STYLE(0)`，只靠 `SetForegroundWindow`。** 否决：零尺寸普通窗口不参与前台切换，`SetForegroundWindow` 对它直接失败。必须配合 `WS_POPUP`。

## Consequences

- 托盘菜单的「点外部关闭」恢复为标准行为。
- 菜单项点击恢复可用 —— 这两者是**同一个根因**：菜单拿不到输入焦点时，点外部不关、点菜单项也无反应。
- 弹菜单不再「抢走」用户的前台窗口，菜单收起后焦点回到原应用。
- 托盘窗口样式从 `WINDOW_STYLE(0)` 变为 `WS_POPUP`。`WS_POPUP` 窗口不参与任务栏与 Alt-Tab，与托盘程序的既有行为一致。
- 真机验证（程序化，`WM_TRAY` 触发 + 坐标点击）三项全过：点外部 ⇒ 菜单关闭；点「打开设置」⇒ 设置进程启动；点「退出」⇒ 托盘进程退出。
- **与 T-169 笔记的关联**：`2026-10-21-tray-menu-command-dispatch` 修的是「菜单命令取回方式」，本条修的是「菜单能否拿到输入焦点」—— 两者是菜单派发链路上相邻但独立的两环。上一条笔记中「点菜单项也点不动」的现象，其根因在本条。
