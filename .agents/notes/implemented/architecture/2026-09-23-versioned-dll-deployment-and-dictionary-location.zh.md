# Agent Note: 版本化 DLL 部署与词典按模块锚点定位

Status: implemented

[English](2026-09-23-versioned-dll-deployment-and-dictionary-location.md) | 中文

## Problem

VM 验收期间，`explorer.exe` 每个会话都持有输入法 DLL 且无法强制释放，原地覆盖
`zhu-ye-ime.dll` 要么失败要么静默保留旧代码。临时解法——每次构建以版本化文件名
复制（`zhu-ye-ime-v4.dll`、`zhu-ye-ime-v5.dll`）并在重启 `explorer` 前把 CLSID
注册表键指向新文件名——带来第二个副作用：旧词典加载器用
`GetModuleHandleW("zhu-ye-ime.dll")` 定位安装目录，与版本化文件名不再匹配，真实
词典被跳过、输入法回退到内置种子词典。

## Decision

- **部署：** 每次 VM 部署把新 DLL 复制为词典旁的 `zhu-ye-ime-v<N>.dll`，将
  `HKLM\SOFTWARE\Classes\CLSID\{E54D6682-…}\InProcServer32` 指向新文件，清理验收
  日志，杀 `explorer.exe`（约 25 秒自动重启），然后校验已加载模块路径。版本化
  文件名同时化解锁定问题：旧 DLL 原样留在磁盘。回退到旧构建只需改注册表一行 +
  重启 explorer；延迟清理与这套方案天然组合（见 T-026）。
- **词典定位：** `installed_dictionary_path` 改为锚定运行加载器的模块内部地址——
  专用 `dictionary_module_anchor` 函数——用
  `GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
  GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT)` 反查，再经
  `GetModuleFileNameW` 取目录。无论部署文件名是否版本化，该机制均有效。

## Alternatives considered

**覆盖前先停止占用进程。** 否决：没有可靠的单一占用者（`handle64.exe` 显示
多个且部分是瞬态的），停无关进程会破坏用于验收的交互会话。

**注册表切换后立即删除旧版本 DLL。**
否决：仍映射旧镜像的进程（如残留的记事本）会继续使用该文件；立即删除会失败
或破坏该会话。`MoveFileEx(MOVEFILE_DELAY_UNTIL_REBOOT)` 的延迟清理才是 T-026
的预期终态。

**保留无版本文件名并在首次加载时缓存安装目录。**
否决：首次加载可能来自旧镜像；模块锚点查找开销极小（引擎创建时一次），且
始终反映实际加载的模块。

## Consequences

VM 上 v4 → v5 版本化部署全程无锁冲突，`dict-ok path="C:\zhu-ye-test\tsf\
dictionary.zyct"` 证明锚点定位在版本化文件名下仍找到真实词典。explorer 自动重启
并加载注册表指向的构建，兼作升级/重载机制。T-026 已在任务清单登记：把该机制
固化进 `install.ps1`/`uninstall.ps1`（复制新版本、切换注册表、`MoveFileEx`
延迟清理、回滚路径）。
