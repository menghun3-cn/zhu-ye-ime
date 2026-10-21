# Agent Note: 设置程序定位按本 DLL 反查，与第三方输入法子类化共存

Status: implemented

[中文](2026-10-21-settings-launch-path-and-sogou-subclassing.zh.md) | English

## Problem

用户报告：右键点击竹叶输入法的托盘图标，在菜单里点「打开设置」，弹出的却是「符号大全」而不是设置窗口。

排查中发现两个彼此独立的问题：

1. **「符号大全」不是我们的窗口。** 该字串实测位于 `C:\WINDOWS\System32\SogouPY.ime` 内，与搜狗拼音的菜单项标识同处一张表：`IDB_MENU_SUPER_SYMBOL`（符号大全 `&F`）、`IDB_MENU_SUPER_REPAIR`（修复输入法 `&R`）、`IDB_MENU_SUPER_MORE`（更多工具 `&O`）、`IDB_MENU_SUPER_UPDATE`（检查更新）。我们的托盘菜单只有四项（模式/工具箱/打开设置/退出），不含这些。

2. **搜狗的全局注入劫持了我们的托盘窗口。** 搜狗的 `SGTool.exe -daemon` 守护进程把 `SogouPY.ime` 等 DLL 注入所有 GUI 进程。我们托盘进程的窗口类名实测被改写：注册时是 `ZhuYeTrayHostWindow`，运行时变成 `ZhuYeTrayHostWindowtools\And???耀\sdk`（后半段是搜狗追加的自身标记）。这说明搜狗对该窗口做了**子类化**（subclass），接管了窗口过程。子类化之后，我们用 `TrackPopupMenu` 弹出的菜单被搜狗的菜单体系替换，其返回值映射到搜狗的菜单 ID，「打开设置」因此执行了搜狗的「符号大全」。

对照实验（同一台机器、同一托盘二进制，只切换搜狗注入守护进程的在否）：

| 条件 | 点「打开设置」的结果 |
| --- | --- |
| 搜狗守护进程已停，托盘未注入搜狗 DLL | 正确打开「竹叶输入法」设置窗口 |
| 搜狗守护进程运行，托盘进程已注入 `SogouPY.ime` | 弹出搜狗的「符号大全」 |

窗口类名也随之变化：未注入时为 `ZhuYeTrayHostWindowOLEAUT`（干净），注入后为 `ZhuYeTrayHostWindowtools\And???耀\sdk`（被改写）。

## Decision

### 1. 设置程序按「本 DLL」反查，不再按宿主进程主模块

`lang_bar::launch_settings` 原先用 `GetModuleFileNameW(None)` 取路径。该 API 返回的是**宿主进程主模块**（`C:\Windows\explorer.exe` 等），不是本 DLL——于是它到宿主目录去找设置程序，永远找不到。同 crate 的 `tsf::installed_module_file` 早已用「模块锚点」模式取到正确结果，本次沿用同一模式：

- 新增 `lang_bar_module_anchor()` 空函数，取其地址经 `GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS)` 反查本 DLL 所在目录；
- 目录反查与 DLL 文件名解耦（部署脚本用版本化文件名，不能按名查找）。

候选目录顺序覆盖两种部署形态：

- 实现 DLL 与设置程序同目录，或其下的 `bin\`（T-176 之前）；
- 实现 DLL 在 `<root>\tsf\versions\<id>\`、设置程序在 `<root>\bin\`（T-176 壳架构）。

向上查找**仅在 DLL 的祖父目录名为 `versions` 时启用**，并最多向上三级。这一限制是必要的：无条件的「上级目录」候选会命中与本 DLL 无关的目录，可能把设置窗口启动成别的东西。原有单元测试「均缺失 → None」正是被这条无约束候选破坏的。

### 2. 托盘的命令行加引号

`zhu-ye-tray` 的 `open_settings` 把**不带引号**的完整路径作为 `CreateProcessW` 的 `lpCommandLine`。安装路径含空格（`C:\Program Files\...`），命令行按空格切分时程序名会被截断。改为 `"<完整路径>"`。

实测记录：加引号与否 `CreateProcessW` 都能成功——Windows 在 `lpApplicationName` 为 `NULL` 时会先尝试把整串当作路径。但加引号是文档明确要求的写法，且不依赖该回退行为，故仍按规范加引号。

### 3. 与第三方输入法的子类化共存

搜狗的全局注入对任何使用弹出式菜单的进程都有影响，我们不打算与它对抗。托盘的窗口类名是注册时固定的，注入方对它的改写不影响我们自身的菜单项定义；真正的失效发生在菜单弹出阶段，由注入方接管。

因此这一项**不是我们代码的缺陷**，处理方式是：

- 记录事实与对照实验，便于将来区分「我们的缺陷」与「共存冲突」；
- 不为对抗第三方注入而引入窗口子类化防护（那会与 TSF 的窗口过程交互叠加，复杂度和风险都更高）。

同一场景下另一处真实缺陷已独立修复：托盘此前不监听 explorer 重建任务栏时广播的 `TaskbarCreated`，且启动时 `Shell_NotifyIcon` 失败会让进程直接退出，explorer 崩溃/重启后托盘图标便永久消失（见 T-176 的托盘修复）。

## Alternatives considered

**按「宿主主模块」推测的 `launch_settings` 修法。** 曾考虑把候选目录扩到宿主主模块目录以覆盖更多布局。否决：那等于让输入法去宿主程序目录找设置程序，既不可靠也不安全。

**用 `ShellExecuteW` 取代 `CreateProcessW` 以自动处理引号。** 两者都可行，`ShellExecuteW` 还能顺带处理「已存在实例则激活」。否决：本轮只修「路径找不到」这一个真缺陷，不顺带改启动语义（激活已开实例属于另一项行为变更，应单独评估）。

**给托盘窗口加 `WS_EX_NOACTIVATE` 或反子类化防护以抵御搜狗注入。** 否决：代价高、收益不确定，且与 TSF 窗口过程交互叠加后难以验证；共存冲突记录在案即可。

## Consequences

- `ITfFnConfigure::Show`（系统输入法设置里的「选项」入口）此前因路径解析错误而**完全打不开设置窗口**；现按本 DLL 定位后可用。验证方式为直接取得实现的 `ITfFnConfigure` 并调用 `Show`，观察到 `zhu-ye-settings.exe` 启动、窗口标题为「竹叶输入法」。
- 新增单元测试 `find设置可执行文件路径_版本目录布局`，覆盖 `<root>\tsf\versions\<id>\` → `<root>\bin\` 的定位与同目录优先。
- 「符号大全」这一现象**未由我们的代码修复**：它需要搜狗停止注入才会消失。已如实记录对照实验与类名改写证据，未做无法验证的断言。
- 用户若仍看到「符号大全」，可临时结束 `SGTool.exe`（搜狗全局注入守护进程）复现对照；这会同时影响搜狗自身的注入行为。
