# Agent Note: Win32 名称参数的 NUL 终止，用类型封死该类缺陷

Status: implemented

[中文](2026-10-21-wide-string-nul-termination.zh.md) | English

## Problem

用户报告（第二次）：托盘右键菜单的「工具箱(T)…」「打开设置(S)…」「退出(X)」三项，**点哪一个都弹出「emoji… 符号大全」**。

上一轮（`2026-10-21-panel-yields-to-app-window`）已修好「面板压住设置窗口」并做了真机复验，但复验的是「面板开着 → 另开设置窗口 → 面板让位」这条路径，**不是**用户实际操作的「点托盘菜单」。用户实际操作仍未修复，说明上一轮的**第二道防线**（托盘 `open_settings()` 里先关面板）从未生效。

### 取证

「符号大全」是本项目的工具箱面板（窗口类 `ZhuYeSettingsPanel`）—— 上一轮已确认。

本轮先验证第二道防线的机制：用 `FindWindowW("ZhuYeSettingsPanel", None)` 找面板。**面板存在时返回 `Err`（找不到）**，改用带尾部 NUL 的字符串则返回 `Ok(HWND)`。两者不一致。

### 根因：类名参数没有 NUL 终止

`FindWindowW`、`CreateMutexW`、`RegisterClassW` 等 Win32 API 都要求**以 NUL 结尾**的 UTF-16 序列。而 `str::encode_utf16()` 得到的序列**没有**尾部 NUL。传进去之后 Win32 会继续往后读内存直到碰到一个 0：

- `FindWindowW` —— **根本找不到**窗口；
- `RegisterClassW` —— 注册出**带内存垃圾**的类名。

后者顺带解释了长期以来的一个谜团：托盘窗口类名实测为 `ZhuYeTrayHostWindowOLEAUT`、`ZhuYeTrayHostWindowtools\And???耀\sdk`。此前被误判为「搜狗子类化改写了我们的类名」（已在 `2026-10-21-settings-launch-path-and-sogou-subclassing` 中更正）。**真相是：我们自己的类名就没终止符，后缀是内存里的垃圾。**

全仓排查后确认有 **4 处**真缺陷（其余 `encode_utf16()` 用途是 GDI 文本绘制等**不需要**终止符的场景，已逐个甄别）：

| 位置 | API | 后果 |
| --- | --- | --- |
| `zhu-ye-tray/src/main.rs` `close_panel_window` | `FindWindowW` | **本缺陷的直接成因**：永远找不到面板 ⇒ 第二道防线完全失效 |
| `zhu-ye-tray/src/task.rs` `panel_exists` | `FindWindowW` | 「面板已在别处打开」的互斥判断恒为假 |
| `zhu-ye-tray/src/main.rs` 单实例 | `CreateMutexW` | 单实例判定失效（可能同时跑起两个托盘） |
| `zhu-ye-tray/src/main.rs` 注册窗口类 | `RegisterClassW` | 类名带内存垃圾 |

### 为什么此前反复中招

**同一个陷阱在本项目已踩三次**：`launch_settings` 的路径比较、`class_name_matches` 的类名比较、这次的 `FindWindowW`。三处的共同特征是「**能编译、能测试、真机却不工作**」—— 单元测试测的是纯函数，不会发现传给 Win32 的字符串没终止。

根因不是运气，是**约定没有固化**：`to_utf16`（带 NUL）与 `encode_utf16`（不带 NUL）在类型上完全等价，编译器无从区分。

## Decision

### 在类型层面封死这类缺陷

新增 `zhu-ye-core::wide::WideString`：

- **只能由 `&str` 构造**，构造时自动追加尾部 NUL；
- 提供 `as_ptr()` 供 `PCWSTR` 使用；
- 提供 `units()` 供确需原始序列的场景（GDI 绘制等**明确不需要**终止符处）。

于是「忘了补 NUL」在**类型层面不可能发生**——想要一个不带 NUL 的序列，必须显式调用 `units()` 并在代码里注明用途。

该模块放在 `zhu-ye-core`（无 Windows API 依赖，符合 `AGENTS.md` 对 core 的约束），托盘与设置程序均可使用。

### 4 处改为使用 `WideString`

### 关闭面板改为同步

`close_panel_window` 原本用 `PostMessageW(WM_CLOSE)`，是**异步**的：函数返回后设置程序可能已经启动，而面板尚未销毁，用户仍会先看到面板。改为 `SendMessageW`（同步等面板处理完 `WM_CLOSE` → `DestroyWindow`），并轮询 `IsWindow` 确认窗口真的消失（上限 500ms）。

无死锁风险：面板的 `WM_DESTROY` 只向宿主发**异步** `PostMessageW(WM_PANEL_CLOSED)`，不会回等托盘主线程。

> **后续修正（2026-10-21）**：本笔记修的是**旁支缺陷**，不是用户报告现象的根因。根因是 T-169 把托盘菜单命令的取回方式从 WM_COMMAND 改成了 TrackPopupMenu 的返回值，却漏加 TPM_RETURNCMD（返回值只是「成功与否」）⇒ 点任何菜单项都被判成第一个菜单项「工具箱」⇒ 弹出符号大全面板。见 2026-10-21-tray-menu-command-dispatch。

## Alternatives considered

**只修 `close_panel_window` 一处。** 否决：另外三处是同一根因，且各自有独立危害（类名乱码、单实例失效、面板互斥失效）。修一处留三处，等于下一次再踩。

**加 clippy lint 或代码审查清单来拦截。** 否决：无法覆盖所有 Win32 字符串参数（有的 API 确实不要终止符），清单式防护必然漏。类型约束是**结构性**的。

**把 `WideString` 放在 `zhu-ye-settings` crate。** 否决：托盘要用它，但托盘对 settings 的依赖方向是「用它的面板窗口」，把通用字符串工具挂在业务 crate 下会让 core 之外出现第二个来源。放 core 更合适。

**用 `PCWSTR::from_raw` 之类的方式继续传裸指针。** 否决：那正是缺陷的成因。

## Consequences

- 用户报告的三个菜单项问题闭环：点「打开设置」时面板被**同步**关闭，设置窗口独占可见。
- 顺带消除托盘窗口类名的乱码后缀（实测从 `ZhuYeTrayHostWindowOLEAUT` 变为干净的 `ZhuYeTrayHostWindow`）。
- 顺带修复单实例判定与面板互斥判定两个潜伏缺陷。
- 新增测试：`zhu-ye-core::wide` 4 个（终止符、空串、往返、非 BMP），托盘 1 个（三个名称类参数均以 NUL 结尾）。
- **`zhu-ye-core` 禁止依赖 Windows 专有 API 的约束得到遵守**：`WideString` 是纯 Rust，不碰任何 Win32 类型。
- 上一轮笔记里「搜狗子类化」的错误归因已更正为「我们自己的类名缺终止符」。
- 与 2026-10-21-panel-yields-to-app-window 是**部分重叠**（那条记录让位机制，本条记录 NUL 约定），两条互相链接。
