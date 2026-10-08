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
  写入前自动建目录（干净机器无 `%APPDATA%\zhu-ye-ime`，VM 验收实测
  os error 3）、tmp 文件带 pid 后缀（`tray-state.<pid>.tmp`，多宿主
  并发不互相覆盖 tmp，rename 保证后写者胜）；
- 引擎接入点（三处，模式变化全路径）：
  1. `sync_engine` 的 `ToggleMode` 分支——锁内切换、锁外通知，与语言栏
     通知同域（覆盖 Shift 弹起 / 设置回调等所有经统一入口的切换）；
  2. 语言栏左键点击 handler（`guard.engine.toggle_mode()` 后取 mode，
     锁外写）；
  3. `create_text_service`——新会话确定起始模式即同步一次，让托盘从
     启动就反映"最近激活会话"的模式（多宿主并存时取最后一次写者）；
- 托盘进程：白"中"（资源 101）/白"英"（资源 102）两枚内嵌图标
  （`winresource` `set_icon_with_id`，assets/tray-zh.ico 与 tray-en.ico，
  橙底白字与 DLL 品牌同风格）；**普通隐藏顶层窗口**（无 WS_VISIBLE
  不进任务栏）+ `WM_TIMER` 500ms 轮询状态文件（mtime 判变，原子写
  保证内容完整）——初版 message-only 窗口在 VM 验收中被否决：
  `Shell_NotifyIcon` `NIM_ADD` 成功但 Server 2019 通知区不渲染
  （截图槽位 0 彩色像素）；单实例互斥
  （`Local\ZhuYeImeTraySingleton`，重复启动静默退出）——互斥句柄必须
  **保活到进程退出**：初版创建后立即 `CloseHandle` 销毁互斥对象，第二个
  进程会新建同名对象并照常运行（用户实测托盘出现两个图标），现以
  `_mutex` 绑定保持在 main 作用域（HANDLE 无 Drop，随进程退出回收）；
  右键菜单
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

## VM 验收修订（2026-10-08，vm-accept-sop 首次实战）

初版引擎完成于开发机（目录预存、窗口可见性无从目视），VM 交互会话
验收实测暴露三处，均已修复并复验：

- **状态桥干净机器写失败**：`%APPDATA%\zhu-ye-ime` 不存在 → os error 3
  → `tray_state::write_tray_state` 增 `create_dir_all` + pid 后缀 tmp
  （4 项单测重写通过）；
- **lone-shift 永不触发（引擎 bug）**：`OnTestKeyUp` 恒返 `BOOL(0)`，
  TSF 只在 `OnTestKeyUp` 返回 TRUE 时才把弹起事件转发给 `OnKeyUp`；
  实测日志只有 `shift-down pending`、没有 `shift-up ToggleMode`。
  修复：`OnTestKeyUp` 对 `VK_SHIFT` 返回 TRUE，其余键放行宿主。复验
  日志两条 `shift-up ToggleMode (lone shift)`、状态文件
  Chinese→English→Chinese 往返 ✓；
- **message-only 窗口在 Server 2019 不渲染托盘图标**（详见决策节），
  改普通隐藏顶层窗口；
- **单实例互斥被提前 CloseHandle 击穿（用户实测双托盘图标）**：初版
  创建互斥后立即关闭句柄，互斥对象随之销毁，第二进程新建同名对象照常
  运行 → 两进程各自 NIM_ADD 出两个图标。修复：`_mutex` 绑定保持句柄
  到 main 退出；本地验证第二个实例 exit 0、进程数恒为 1，VM 部署位
  同步（SHA256 35FF8091E553）。
- **发布链缺托盘整合（v0.1.2 补发发现）**：批六新增 crate 后 release
  链未跟进（T-078 早于批六）——构建/便携包/安装/卸载均不含
  zhu-ye-tray.exe，产出的 zip 会缺托盘。修复（fix/T-112-release-tray-packaging）：
  assemble-release 纳入构建与产物校验 + zip 四件套完整性守卫（缺一即中止发布），
  stage/package-portable 复制托盘，install 装三 exe 并注册 HKCU Run
  `竹叶输入法托盘`，uninstall 停进程删自启。同一修复批次把批四的带调拼音
  旁挂表 `dictionary.zyct.tones`（`<基础包路径>.tones` 约定）也纳入发布链：
  stage 复制、install 与词典同目录安装、assemble zip 守卫六件套校验。
  **v0.1.2 同版本补发**：CI 重跑（run #37723763247，upload=true）产出
  22.65MB zip（六件套守卫验证通过，含托盘与 tones）并上传既有 v0.1.2
  Release + feed-v0.1.2 镜像；部署所用 DLL 为 CI 构建 f143c14e
  （SHA256 F143C14E…，与本地重跑 8fb21f0a 同尺寸 2,195,456B 不同哈希，
  可复现体积、哈希差异为构建时间戳所致，部署以 CI 哈希为准）。

验收证据（截图存 VM `C:\zhu-ye-vm\shots-b6\`）：溢出窗格三态截图
`s1-zh/s2-en/s3-zh2` MD5 两两不同，zh↔en 像素差异 bbox
(765,744)-(773,752) 恰好 8×8 px 落在托盘槽 (758..782, 728..768)；
端到端 `s4-shift-en/s5-shift-zh` 记事本激活→Shift×2 状态桥往返正常。
**补发后基线（2026-10-08 整体验收）**：用户机
`C:\Program Files\zhu-ye-ime\tsf\zhu_ye_ime_F143C14E.dll`（CLSID/IconFile
指针已切换）+ VM `tsf-b6\zhu-ye-ime-f143c14e.dll` 注册生效，
托盘进程两环境单实例复核通过（第二实例 HasExited、进程数恒 1）；
VM 整体 UI 验收 v4 共 57 项指标全绿——托盘单实例/三态图标、
空格上屏「你好」、Enter 提交拼音原文、日期/金额/v/emoji/错位/混输
候选电池、Shift 双翻转 + 英文态直通上屏、设置入口窗口、
T-091 哨兵日志（shift-up ToggleMode ×2 + Activate 行）实测。

## Related

- 批五回退（单档案语义——托盘出现前系统指示器"恒竹"的边界记录）：
  [2026-10-07-single-profile-revert.md](2026-10-07-single-profile-revert.md)
- 批四（d 方向"无常驻进程不可行"的原始论证，本方案补充常驻进程）：
  [2026-10-07-acceptance-fix-batch-4.md](2026-10-07-acceptance-fix-batch-4.md)
- 任务栏图标机制（IconFile/DLL 内嵌，与托盘图标同风格）：
  [2026-10-06-taskbar-icon-zhu.md](2026-10-06-taskbar-icon-zhu.md)
