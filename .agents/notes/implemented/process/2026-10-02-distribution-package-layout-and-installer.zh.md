# Agent Note: 发行包布局与安装器语义

Status: implemented

[English](2026-10-02-distribution-package-layout-and-installer.md) | 中文

## 问题

FR-045 要求设置窗口能到达干净机：旧 `install.ps1` 从源码树取 DLL（必要时现场
`cargo build`），只复制 TSF 服务与一个词典，不建 `packs/`，也不建快捷方式。它既无法
交付 `zhu-ye-settings.exe`，也无法成为目标机上的修复/升级后端——目标机上没有任何设置
窗口日后可重新执行来修复自身的程序。

## 决策

`package-portable.ps1` 现在产出**发行包**，布局固定；`install.ps1` 从包内取文件而不再
依赖源码树：

```
ai-zhu-ye-ime-<version>-test/
  bin/zhu_ye_ime.dll              TSF 服务 DLL（安装时版本化复制）
  bin/zhu-ye-settings.exe         设置窗口
  bin/zhu-ye-updater.exe          词典更新器（唯一联网组件）
  bin/dictionary.zyct             基础词典（data/artifacts/base.zyct）
  packs/it.zyct med.zyct slang.zyct   预置领域包（D-46，可离线验收）
  scripts/ime-identity.ps1 install.ps1 uninstall.ps1
          verify-tsf-dll.ps1 verify-tsf-identity.ps1
  docs/licenses.md 数据清单.md
```

- `install.ps1`：解析 `-PackageRoot`（默认：脚本目录的上级），校验包内容，版本化复制并
  校验 DLL 导出，复制基础词典（`bin/dictionary.zyct`），把两个 exe 装到
  `%ProgramFiles%\ai-zhu-ye-ime\bin\`，把三个领域包预置到 `%APPDATA%\ai-zhu-ye-ime\packs\`，
  以原有回滚事务写入 TSF 注册树，创建开始菜单快捷方式（D-26 唤起入口）。
  `-SkipBuild` 现为无操作（发行模式从不构建）。新增 `-SkipRegistration` 开关跳过 HKLM
  注册与快捷方式，让复制/预置逻辑无需管理员 shell 或 TSF 写入即可演练。幂等：同哈希
  DLL 覆盖，exe 与领域包同名覆盖。
- `uninstall.ps1`：新增清理两个 exe（占用时延迟清理）、开始菜单快捷方式与空的
  `tsf\`/`bin\`/应用根目录。**`%APPDATA%\ai-zhu-ye-ime` 故意保留**——配置、领域包与
  用户词库属用户数据；卸载只清程序文件。
- 基础词典来源变化：旧 `install.ps1` 偏好 `data/artifacts/real.zyct`（开发产物）；发行
  模式改发 `base.zyct`（随安装只读基础包，见 architecture 文档布局）。

## 备选方案

**保留安装时构建、只让 package-portable 多打两个 exe。** 否决：FR-045 明确要求干净机
"不依赖源码目录与 cargo"；保留构建路径会让安装器同时存在源码树与发行包两条代码路径，
验收无法区分。

**卸载时整删 `%APPDATA%\ai-zhu-ye-ime`。** 否决：用户词库与配置是长期积累的用户数据，
卸载即删与一级修复"绝不删除用户数据"（D-41）同原则冲突。手动清理路径已写入
docs/安装与使用.md。

**演练模式也建快捷方式。** 否决：`-SkipRegistration` 演练会把指向临时目标的开始菜单
项残留到系统；该开关统一收拢所有系统级副作用。

## 后果

- 干净机无需 Rust 工具链：解压后直接 `install.ps1`（验收标准 13.1 FR-045 各行）。
- 安装器只写确定位置（`Program Files\ai-zhu-ye-ime\{tsf,bin}`、开始菜单、`%APPDATA%`），
  设置窗口可在"管理输入法"中复核。
- D-46 预置领域包使 FR-042 可离线验收；在线更新保留为升级途径（设计 §8）。
- `-SkipRegistration` 演练钩子：跳过 HKLM 写入与快捷方式，仍演练复制/预置/校验全程；
  VM 验收以管理员身份走完整路径。
- 卸载按设计保留用户数据；验收行"卸载后注册表与文件无残留"的范围是程序文件与注册表，
  已在 docs/安装与使用.md §9 说明。

## 关联笔记

- [TSF 注册与生命周期](../architecture/2026-09-18-tsf-registration-and-lifetime.md)：
  注册树、回滚事务与版本化 DLL 延迟清理语义不变；本笔记只改变安装器的输入来源。
- [便携脚本编码](../bug-fix/2026-09-22-portable-scripts-windows-powershell-5-1-encoding.md)：
  包内每个脚本仍须 UTF-8 BOM（package-portable 以 BOM 重写）。
