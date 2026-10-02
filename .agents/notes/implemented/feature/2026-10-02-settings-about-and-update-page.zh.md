# Agent Note：设置窗口关于与更新页（M12-5，T-077）

状态：implemented

[English](2026-10-02-settings-about-and-update-page.md) | 中文

## 问题

关于与更新页（FR-044）必须给用户可见的"检查更新"入口与诊断视图，同时让设置窗口自身
**完全不联网**（D-44）：一切网络连接必须来自独立的 `zhu-ye-updater.exe` 进程。
更新默认关闭（P-03）：`online_update=false` 时界面必须禁用检查/应用按钮并说明原因，
且窗口不得 spawn 任何进程——保证"关闭时零出站连接"可测（验收 13.1 FR-044）。

另有两条约束：应用更新会替换本机程序文件，需要二次确认；诊断信息（配置路径、数据目录、
日志目录、已装包）必须不经 updater 就展示（T-077 本地直读）。

## 决策

### 更新器桥 `crates/zhu-ye-settings/src/updater.rs`

唯一的联网抽象层：`updater_exe_path()` 在当前可执行文件同目录定位
`zhu-ye-updater.exe`（`std::env::current_exe()` 目录——安装脚本把两个 exe 都放进同一个
`bin\`，T-078）；`run(program, args)` 经 `Command::output` spawn 子进程、拼接 stdout+
stderr，非零退出码返回 `Err(text)` 以便原样展示失败；`output_lines(text)` 去掉空行供
结果区逐行绘制。本模块无 Win32 依赖（纯 `std::process`），可无 GUI 单测。

### 关闭时窗口不 spawn 任何进程

`can_run_updater` 对每条检查/应用路径设三重门禁：`online_update`（来自
`config.json`）、更新器存在、无任务在跑。禁用按钮用控件底 + 占位文字渲染，命中为
空操作；说明区写明原因（"更新关闭，D-44：请在列表页开启" / "更新器缺失：请安装完整
发行包"）。门禁位于**任何 spawn 之前**，因此 `online_update=false` ⇒ 零进程零连接，
可测。

### 「启用在线更新」是运行时开关，列于关于页首条

`ChipValue::OnlineUpdate(bool)` 复用二选一 chip 机制（"关闭"/"开启"，关闭在前，
P-03）。点击即经 `config::save_online_update` 持久化（先重读再写，S-8），与主题/
模式同口径。点条目行本身无动作。

### 检查更新子视图

布局：说明区（开关状态 + 禁用原因/更新器路径）、结果区（更新器输出逐行展示，
**错误如实展示**——例如"未内置发布公钥"失败或清单 URL 的 404，绝不当成"无更新"）、
底部按钮 [检查更新][应用更新][返回]。

### 后台任务线程 + 消息循环回执

`Command::output` 会阻塞，放在 UI 线程会冻结消息循环。`start_update_task` spawn
线程：运行更新器、经 `mpsc` 送回结果、**然后**投递 `WM_UPDATER_DONE`
（WM_USER+0x120）；窗口过程取回 receiver 后立即 `try_recv` 并重绘。因为 send 先于
post，消息到达时结果必在 channel 里。`WM_UPDATER_DONE` 的 `wparam` 携带任务序号，
被新任务顶替的旧任务迟到回执直接忽略。`HWND` 不是 `Send`，线程收到 `hwnd.0 as isize`
再重建。

### 应用更新需二次确认

应用按钮先弹 `MessageBox`（YESNO，D-44）再 spawn `zhu-ye-updater apply`。

### 诊断子视图只读本地

`build_diagnostics` 读取 `zhu_ye_core::core_version()`、`config.json` 路径、数据目录、
验收期日志目录与已装包列表（复用既有本地清单扫描 `list_packs_now`）——不 spawn、
不联网。格式行按管理/修复子视图的逐行样式绘制。

### 取证入口

`--shot --update` / `--shot --diag` 进入两个子视图出 BMP（与 `--packs/--manage/
--repair` 并列）。

## Alternatives considered

1. **设置窗口内嵌 HTTP 客户端**（如 reqwest 或 curl 子进程）：违反 D-44 的单一联网
   组件边界（应为更新器进程）；否决——窗口必须可证明离线。
2. **UI 线程同步调用更新器**：`Command::output` 会阻塞，直接调用会冻结整个检查/
   下载期间的窗口消息循环；否决——后台线程 + mpsc + `PostMessageW` 回执让窗口保持
   响应，更新器的长尾不占消息循环。
3. **应用更新不确认**：应用会替换本机程序文件；否决——每次 apply 前先弹 MessageBox
   二次确认（D-44）。
4. **诊断走 `zhu-ye-updater status`**：窗口本地已可读的数据（配置路径、核心版本、
   `installed.json` 清单）还要多挂一个进程依赖；否决——spawn 只限检查/应用。
5. **复用既有应用消息做回执**：面板关闭消息语义不同；否决——专用 `WM_USER+0x120`
   回执携带任务序号，被顶替任务的迟到回执被忽略。

## 后果

- `config.rs` 新增 `load_online_update`/`save_online_update`，含往返与字段保全测试
  （S-8 重读保留 `last_check`）。
- settings 库测试 95→97（model：关于页三条目 Ready + 开关默认关 + 子视图切换；
  layout：更新/诊断几何；updater 桥：行过滤、非零退出码与启动失败路径，用
  `cmd.exe` 实测）。
- 宿主取证（2026-10-02）：三张 BMP（诊断页、更新关闭态、更新开启态）——开/关两态
  隔行采样 3650 像素差异；真实 updater 行为确认：`online_update=false` ⇒ `check`
  零网络立即退出（"未发起任何网络请求"），`true` ⇒ 由更新器进程发起连接，清单 404
  失败原因由窗口如实展示。
- 欠账：「显式开启」「检查更新」「二次确认」交互与抓包条款按验收标准 13.5 挂起待 VM。

## 相关笔记

- [发行包布局与安装器](../process/2026-10-02-distribution-package-layout-and-installer.md)
  —— 把 `zhu-ye-updater.exe` 与设置程序同目录交付（T-078）；本页依赖该交付。
- [AI 服务契约与默认离线](../architecture/2026-09-21-ai-service-contract-and-offline-default.md)
  —— "未经显式开启即离线"的先例：P-03 把同一立场延伸到设置窗口。
- [TSF 注册与生命周期](../architecture/2026-09-18-tsf-registration-and-lifetime.md)
  —— 管理/修复页背后的注册所有权；诊断视图展示相关路径但不动注册表。
- [候选窗 GDI 渲染](../feature/2026-09-19-candidate-window-gdi-rendering.md)
  —— 设置窗口复用的主题/度量原语（T-081 将抽取为 `zhu-ye-ui`）。
