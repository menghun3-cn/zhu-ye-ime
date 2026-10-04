# Agent Note: 设置窗口输入法管理与两级修复

Status: implemented

[English](2026-10-02-settings-input-management-and-repair.md) | 中文

## 问题

FR-043 要求设置窗口在不重装的前提下完成输入法 TSF 注册的查看与修复。语言栏
或输入法失效时，用户没有任何产品内途径自查：注册状态（profile 键、`Enable`、
`InProcServer32`）不可见，唯一办法是重装或重跑安装器。本任务带三条约束决策：
先报告后动手（D-41）、绝不删除用户数据（只改名）、提权面尽量小（D-40）。

## 决策

「常用设置」页新增三个 `Ready` 条目，打开两个子视图与一个直接动作：

- **恢复状态栏**（`RestoreLangBar`）：确认弹窗后（D-43）经
  `taskkill /f /im ctfmon.exe` 终止 `ctfmon`，并提示系统按需自动重新加载
  语言栏。
- **管理输入法**（`OpenManage`）：只读探测子视图。`RegistryProbe` 从
  `HKLM\SOFTWARE\Microsoft\CTF\TIP\…` 与 `HKLM\SOFTWARE\Classes\CLSID\…`
  读取六项事实（语言配置文件键存在、`Enable` DWORD、`InProcServer32`
  默认值、`ThreadingModel`）；`evaluate_registration` 将其归并为
  `RegisterIssue` 清单与一行标题。子视图另提供**打开系统输入法设置**，
  经 `ShellExecuteW("open", …)` 打开 `ms-settings:keyboard` URI。
- **修复输入法**（`OpenRepair`）：先报告后动手子视图。进入即执行
  `scan_l1`（只读）并逐行展示 `summary_lines()`；底部按钮仅在显式点击后
  动作：
  - **一级修复（无需管理员）**：缺失的 `packs/` 目录重建；损坏的 `.zyct`
    包、`config.json`、`user_words.json` 改名隔离（`.bak`、`.bak.1` 递增，
    绝不删除）后重建；版本更高的 `user_words.json` 只报告（"请升级输入法"）
    不改名；基础词典损坏只报告。各项结果互不拖累（`Vec<L1Outcome>`）。
  - **二级修复（需管理员，UAC）**：`ShellExecuteExW` 以 `runas` 动词拉起
    `zhu-ye-settings --repair-registry`，`SEE_MASK_NOCLOSEPROCESS` 等待并
    读取退出码。子命令的 DLL 路径先读 `InProcServer32`（回退取
    `exe_dir\tsf` 下最新 `zhu-ye-ime*.dll`），删除 TIP 与 CLSID 两棵树后
    按 `ime-identity.ps1 New-TsfRegistration` 的同样值重建六个键；成功静默
    退出码 0，失败弹窗说明且退出码 1。绝不接受任意路径参数。成功后父窗口
    重探注册并再次询问是否重启 `ctfmon`（D-43）。

备份命名沿用 `UserDictStore::load` 先例（`user_words.json` → `user_words.bak`，
扩展名被替换），运行时自动恢复与修复入口产生同一种幸存文件名。

子命令在 `main.rs` 的单实例守卫之前短路。注册表写入值与
`scripts/ime-identity.ps1` 一致，该侧常量由 T-079 门禁
`scripts/verify-tsf-identity.ps1` 逐字守护。

## 曾考虑的替代方案

**整个设置进程提权**（应用清单 `requireAdministrator`）。
否决：每次启动都弹 UAC，且把管理员权限授予一个只有单个动作需要的 GUI
表面；D-40 要求最小面，只有 `--repair-registry` 子进程被提权。

**一级修复直接删除损坏文件。**
否决：D-41 的"改名而非删除"保证用户数据可恢复；隔离改名用 `.bak` 计数且
绝不覆盖既有备份。

**一级修复处理高版本 `user_words.json`。**
否决：改名会静默丢弃新版客户端写入的数据；探针只报"请升级输入法"。

**恢复状态栏改调安装器。**
否决：便携部署下设置窗口内不应依赖安装器；`ctfmon` 重载在进程内自包含
（开放项，见设计文档 §6.4，留待 VM 确认）。

**把注册状态内联到条目行。**
否决：六项事实加修复按钮放不下一行；子视图模式（「添加词库」已用）复用整
个内容区、底部按钮与返回导航。

## 后果

- 注册表写入仅发生在用户显式点击二级修复时；探测与一级扫描全部只读。
- TSF 常量仍双份维护；T-076 未新增常量，T-079 门禁仍 6/6 通过。
- `restart_ctfmon` 是设计 §6.4 的开放实现项：当前行为是终止 ctfmon 并依赖
  按需自动重载；VM 实录确认系统是否（以及多快）重新拉起后，再回填设计文档。
- 设置侧测试 84 个通过（model 子视图、manage/repair 布局几何、修复逻辑、
  注册表路径常量）；core 探针测试新增 7 个。宿主截图（1100×775）经 accent
  按钮像素扫描验证：常用设置页无底部按钮，管理子视图两个，修复子视图三列。
- 交叉引用
  [2026-09-18-tsf-registration-and-lifetime](../../implemented/architecture/2026-09-18-tsf-registration-and-lifetime.md)
  （注册结构与生命周期）与
  [2026-09-22-portable-scripts-windows-powershell-5-1-encoding](../../implemented/bug-fix/2026-09-22-portable-scripts-windows-powershell-5-1-encoding.md)
  （编码约定；T-079 的两个身份脚本起初漏加 BOM，随本变更一并修正，使门禁
  能在 PowerShell 5.1 上执行）。
