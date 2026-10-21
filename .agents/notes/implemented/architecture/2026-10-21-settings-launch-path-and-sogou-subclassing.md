# Agent Note: 设置程序定位按本 DLL 反查

Status: implemented

[中文](2026-10-21-settings-launch-path-and-sogou-subclassing.zh.md) | English

## Problem

用户报告：右键点击竹叶输入法的托盘图标，在菜单里点「打开设置」，弹出的却是「符号大全」而不是设置窗口。

排查中发现两个彼此独立的问题：

1. **「符号大全」不是我们的窗口。** 该字串实测位于 `C:\WINDOWS\System32\SogouPY.ime` 内，与搜狗拼音的菜单项标识同处一张表：`IDB_MENU_SUPER_SYMBOL`（符号大全 `&F`）、`IDB_MENU_SUPER_REPAIR`（修复输入法 `&R`）、`IDB_MENU_SUPER_MORE`（更多工具 `&O`）、`IDB_MENU_SUPER_UPDATE`（检查更新）。我们的托盘菜单只有四项（模式/工具箱/打开设置/退出），不含这些。

   > **后续修正（2026-10-21）**：本条把「符号大全」判成搜狗的界面，是**误判**。用户提供的截图里含「点击输入关键词过滤（拼音别名/字符）」「点击格子上屏 · 呼出热键 Ctrl+Alt+E」等**本项目自己的**文案 —— 「符号大全」实为我们的工具箱面板（窗口类 `ZhuYeSettingsPanel`）。字串搜索命中搜狗只是巧合。真正的成因是面板（`WS_EX_TOPMOST` 浮层）压住了设置窗口；见 `2026-10-21-panel-yields-to-app-window`。

2. ~~**搜狗的全局注入劫持了我们的托盘窗口。**~~ **该归因不成立。** 后续核查表明：搜狗的全局注入确为全系统范围（实测 328 个进程被注入 `SogouPY.ime`，含 `lsass`/`csrss`/`services`），但**与我们的 DLL 无关** —— 我们的导入表里没有任何搜狗 DLL，且每个加载我们 DLL 的宿主进程里搜狗 DLL 的加载位置**都更靠前**（如 chrome：搜狗 59 / 我们 145），因果方向是「搜狗注入 → 我们的 DLL 后加载进这个已被污染的进程」。托盘窗口类名上那串乱码后缀在搜狗**未注入**时同样出现（`ZhuYeTrayHostWindowOLEAUT`、`...ARE_ACCOU鈻...`），故**不是搜狗改写**。

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

### 3. ~~与第三方输入法的子类化共存~~ → 归因修正

> **后续修正（2026-10-21）**：本节原先的结论（「搜狗子类化了我们的托盘窗口并接管菜单」）**不成立**。真实成因是：工具箱面板（`WS_EX_TOPMOST` 浮层）压住了设置窗口，用户点「打开设置」看到的其实是面板。修复见 `2026-10-21-panel-yields-to-app-window`。

保留仍然成立的事实：搜狗的全局注入是全系统范围的（328 个进程），但**与我们的 DLL 无因果关系**（导入表无搜狗 DLL；宿主进程里搜狗的加载位置一律更靠前）。本项目不为此做任何对抗性处理。

同一场景下另一处真实缺陷已独立修复：托盘此前不监听 explorer 重建任务栏时广播的 `TaskbarCreated`，且启动时 `Shell_NotifyIcon` 失败会让进程直接退出，explorer 崩溃/重启后托盘图标便永久消失（见 T-176 的托盘修复）。

## Alternatives considered

**按「宿主主模块」推测的 `launch_settings` 修法。** 曾考虑把候选目录扩到宿主主模块目录以覆盖更多布局。否决：那等于让输入法去宿主程序目录找设置程序，既不可靠也不安全。

**用 `ShellExecuteW` 取代 `CreateProcessW` 以自动处理引号。** 两者都可行，`ShellExecuteW` 还能顺带处理「已存在实例则激活」。否决：本轮只修「路径找不到」这一个真缺陷，不顺带改启动语义（激活已开实例属于另一项行为变更，应单独评估）。

## Consequences

- `ITfFnConfigure::Show`（系统输入法设置里的「选项」入口）此前因路径解析错误而**完全打不开设置窗口**；现按本 DLL 定位后可用。验证方式为直接取得实现的 `ITfFnConfigure` 并调用 `Show`，观察到 `zhu-ye-settings.exe` 启动、窗口标题为「竹叶输入法」。
- 新增单元测试 `find设置可执行文件路径_版本目录布局`，覆盖 `<root>\tsf\versions\<id>\` → `<root>\bin\` 的定位与同目录优先。
- 「符号大全」现象的真实成因与修复见 `2026-10-21-panel-yields-to-app-window`；本笔记的设置程序定位修复是那条链路上**独立成立**的一环（`ITfFnConfigure::Show` 此前确实打不开设置窗口）。
