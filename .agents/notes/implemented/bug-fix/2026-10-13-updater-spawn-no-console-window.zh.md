# Agent Note: GUI 设置窗口 spawn 更新器子进程不得闪现控制台窗口（T-153）

Status: implemented

[English](2026-10-13-updater-spawn-no-console-window.md) | 中文

## Problem

用户反馈（T-153）：「关于与更新」页启用在线更新后，点「检查更新」会弹出
cmd 命令行窗口。根因：`zhu-ye-updater.exe` 是**控制台子系统**程序，而设置
窗口进程是 **GUI 子系统**（本身无控制台）；`updater::run` 用普通
`Command::output()` spawn 它时，Windows 每次都会给该 CON 子进程**新建一个
可见控制台**——闪现的黑框 cmd。IME 侧的 `check-once` spawn（S-8，
`CREATE_NO_WINDOW`）与 T-133（对话框自身改 GUI 子系统）早已处理同族问题，
唯独设置窗口的手动检查/应用路径漏掉标志。

## Decision

在 `crates/zhu-ye-settings/src/updater.rs`，`run()` 在 `Command::output()`
之前增加 `suppress_console_window` 一步：

- **Windows**：经 `std::os::windows::process::CommandExt` 设
  `creation_flags(CREATE_NO_WINDOW)`——std 自带扩展、无第三方依赖，
  保住本模块"无 Windows API 依赖"的属性（扩展是 std 自留面）。
  一处覆盖 `check` 与 `apply` 两个入口。
- **非 Windows**：空操作，模块在任意平台仍可编译。

`CREATE_NO_WINDOW` 对 CON 子进程仍会创建控制台（保证 CRT 启动正常），
只是**不显示窗口**；stdout/stderr 依旧经管道收集，退出码语义不变。
更新器保持控制台程序是有意为之：它也支持手工诊断运行，那时可见控制台是
特性——所以这里刻意不做 T-133 式的 GUI 子系统转换。

## Alternatives considered

**把 `zhu-ye-updater` 改成 GUI 子系统（`windows_subsystem = "windows"`）。**
否决：手工诊断运行（管理员控制台）会失去控制台输出；而 GUI spawn 路径下
子进程本就不需要自己的控制台（设置窗口经管道收集）。机制上更贴近 IME 侧
`CREATE_NO_WINDOW` 先例（S-8），且保持二进制可独立使用。

**只在 `start_update_task`（window.rs）设 `creation_flags`。**
否决：标志属于进程 spawn 抽象（`run`）的启动语义；写进调用方会把 Windows
spawn 细节泄漏到窗口逻辑，还容易漏给未来的新调用方。

## Consequences

- 设置窗口手动「检查更新」「应用更新」不再闪现控制台；结果区仍显示更新器
  经管道收集的输出。
- 回归测试 `无窗口标志下子进程输出收集正常`（Windows-only）钉住输出路径；
  既有 `run` 测试断言改 ASCII——`CREATE_NO_WINDOW` 下子进程拥有独立新
  控制台（系统默认代码页 936），cmd echo 中文会以 GBK 字节写出；真实更新器
  用 Rust `println!` 经管道写 UTF-8，不受影响。
- 本仓库的重复性规则：任何 GUI 进程 spawn 控制台子进程必须加
  `CREATE_NO_WINDOW`；任何静默 spawn 的控制台子进程不得期望继承父控制台。

## Related

- IME 侧静默 `check-once` spawn 先例（S-8）：
  [2026-10-04-settings-window-five-upgrades.md](../../implemented/feature/2026-10-04-settings-window-five-upgrades.md)
- T-133 常驻窗口改 GUI 子系统（"托盘弹 cmd 窗口"）：
  [2026-10-07-system-tray-mode-icon.md](../../implemented/feature/2026-10-07-system-tray-mode-icon.md)
- 验收：[docs/验收标准.md](../../../docs/验收标准.md) §21（T-153）
