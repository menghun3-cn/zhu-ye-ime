# Agent Note: Windows 系统托盘"中/英"状态图标（T-112 批次六）

Status: implemented

English | [中文](2026-10-07-system-tray-mode-icon.zh.md) | 中英镜像

## Problem

用户要求新增系统托盘图标，展示输入法当前状态：白"中"=中文输入模式、
白"英"=英文输入模式。输入法本体是 TSF DLL，被各宿主进程（notepad 等）
加载；批四已论证 DLL 内直接 `Shell_NotifyIcon` 不可行（图标挂在宿主
进程栈、宿主退出即消失，且多宿主各注册一个冲突）。

## Decision

**新增独立常驻托盘进程 `zhu-ye-tray`**（搜狗/QQ 拼音同款做法），
引擎与托盘之间用**极小状态文件桥**传模式：

- 桥梁：`zhu_ye_core::tray_state`——引擎在切换时原子写
  `%APPDATA%\zhu-ye-ime\tray-state`（tmp+rename，单行
  `Chinese`/`English`，解析只认两值）；写入失败只降级，绝不阻断输入；
- 引擎接入点（三处，模式变化全路径）：
  1. `sync_engine` 的 `ToggleMode` 分支——锁内切换、锁外通知，与语言栏
     通知同域（覆盖 Shift 弹起 / 设置回调等所有经统一入口的切换）；
  2. 语言栏左键点击 handler（`guard.engine.toggle_mode()` 后取 mode，
     锁外写）；
  3. `create_text_service`——新会话确定起始模式即同步一次，让托盘从
     启动就反映"最近激活会话"的模式（多宿主并存时取最后一次写者）；
- 托盘进程：白"中"（资源 101）/白"英"（资源 102）两枚内嵌图标
  （`winresource` `set_icon_with_id`，assets/tray-zh.ico 与 tray-en.ico，
  橙底白字与 DLL 品牌同风格）；message-only 窗口 + `WM_TIMER` 500ms
  轮询状态文件（mtime 判变，原子写保证内容完整）；单实例互斥
  （`Local\ZhuYeImeTraySingleton`，重复启动静默退出）；右键菜单
  （模式状态行 / 打开设置 / 退出），左键单击弹菜单、双击开设置；
  每次 `LoadImageW` 后立即 `DestroyIcon`（Shell 已取得副本）；
- 部署位与自启：`C:\Program Files\zhu-ye-ime\bin\zhu-ye-tray.exe`
  （worker `copy-exe`，target 需**完整文件名**），HKCU Run 值
  "竹叶输入法托盘"（HKCU 普通权限可写，不走提权通道）；
- 托盘与系统任务栏指示器**并存不冲突**：系统指示器恒显"竹"（单档案
  语义，平台边界），托盘专职会话中英状态；两者视觉同风格（橙底白字）。

## Alternatives considered

**DLL 内 `Shell_NotifyIcon`**（批四 d 方向）：否决——图标属宿主进程
栈，进程退出即消失；多宿主并发各自注册冲突。本方案不推翻该论证，
而是新增常驻进程承载。

**托盘轮询 ITfThreadMgr 查激活会话模式**：否决——跨进程查 ctfmon 的
激活状态没有稳定公开 API（`GetActiveLanguageProfile` 只回答档案级，
返回中文档案，回答不了会话内中英），状态路由必须由引擎自己写出来。

**命名管道 / WM_COPYDATA 实时推送**：否决——引擎侧为零成本轮询的
对称方案；文件桥已满足 500ms 量级指示需求且崩溃自愈（托盘重启即读
文件），管道还要处理对端生命周期。

## Consequences

- 用户可随时在托盘看到当前中英状态（白"中"/白"英"），Shift 切换
  500ms 内翻新图标；托盘退出不影响输入法（独立进程、只读状态文件）；
- 状态语义 = "最近一次模式变化"（多宿主并发编辑时以最后写态为准），
  与系统指示器"竹"（档案级）语义分层，已在 todos 记录；
- worker `copy-exe` 目标必须给完整文件名（`bin\zhu-ye-tray.exe`），
  给目录会落入 target 白名单拒绝（本批次实际踩过，字段名 source/target
  与 src/dst 也踩过一次——请求字段以 worker 源码为准）；
- `tray-state` 文件小、写频低（仅切换时），无热路径问题；文件桥格式
  是跨模块契约（core 常量 `TRAY_STATE_FILE_NAME`/`MODE_*`），改动需
  同步引擎与托盘；
- 新 crate 进入 workspace（`crates/zhu-ye-tray`），发布产物加
  `bin\zhu-ye-tray.exe`（261,632B）。

## Related

- 批五回退（单档案语义——托盘出现前系统指示器"恒竹"的边界记录）：
  [2026-10-07-single-profile-revert.md](2026-10-07-single-profile-revert.md)
- 批四（d 方向"无常驻进程不可行"的原始论证，本方案补充常驻进程）：
  [2026-10-07-acceptance-fix-batch-4.md](2026-10-07-acceptance-fix-batch-4.md)
- 任务栏图标机制（IconFile/DLL 内嵌，与托盘图标同风格）：
  [2026-10-06-taskbar-icon-zhu.md](2026-10-06-taskbar-icon-zhu.md)
