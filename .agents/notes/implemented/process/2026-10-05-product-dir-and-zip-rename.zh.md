# Agent Note: 产品目录与发布包命名去 `ai-` 前缀（T-108、T-109）

Status: implemented

[English](2026-10-05-product-dir-and-zip-rename.md) | 中文

## 问题

发布产物与产品目录路径带 `ai-` 前缀（`ai-zhu-ye-ime-<v>.zip`、
`%APPDATA%\ai-zhu-ye-ime`、`Program Files\ai-zhu-ye-ime`、
`%LOCALAPPDATA%\ai-zhu-ye-ime\logs`、TSF 安装目录 `ai-zhu-ye-ime\tsf`）。用户先要求
zip 名称去掉 `ai-` 前缀，随后要求产品目录同样去掉，并明确**不做向后兼容**：旧名不
保留、不做迁移。

## 决策

### 统一更名 `zhu-ye-ime`（用户直接指令，无兼容）

- 发布 zip：`zhu-ye-ime-<v>.zip`（v0.1.2 release 旧名资产已删除——资产 id
  612718330；同日补发新名 zip 19,699,745 B）。
- 便携包顶层目录：`zhu-ye-ime-<v>-test/`（`package-portable.ps1`
  `$packageName`）；staging 目录与开发期 test-zip 产物名同步。
- 产品目录：`%APPDATA%\zhu-ye-ime`（config/packs/themes/user_words.json/
  update_status.json）、`Program Files\zhu-ye-ime`（安装根，`\bin`、TSF `\tsf`
  经 `TSF_INSTALL_DIR_RELATIVE`）、`%LOCALAPPDATA%\zhu-ye-ime\logs`（产品日志）、
  HKCU per-user TSF DLL 位置 `%LOCALAPPDATA%\zhu-ye-ime\tsf`。
- **不变**：exe 名 `ai-zhu-ye-ime-setup-<v>.exe`（两轮指令均未点名）、产品/项目名
  竹叶输入法（ai-zhu-ye-ime）（README 标题、Cargo authors）、历史记录（todos-done、
  CHANGELOG 已发布段、验收证据行）、哨兵路径 `C:\zhu-ye-test`。

### 无迁移、无遗留处理

- install.ps1 不从 `%APPDATA%\ai-zhu-ye-ime` 复制；uninstall.ps1 只清理新目录；
  任何地方不做双路径读取。v0.1.2 已装用户升级需重装（旧目录仅不再被使用）。
- v0.1.2 release 仅保留新名 zip；SHA256SUMS.txt 随 staging 重算（新名行，无
  旧名行）。

## 后果

- `verify-tsf-identity.ps1` 门禁两侧一致：Rust `TSF_INSTALL_DIR_RELATIVE =
  "zhu-ye-ime\\tsf"` ↔ `ime-identity.ps1` `Get-TsfInstallDir` `'zhu-ye-ime\tsf'`。
- 路径单测随常量更新（settings `product_log_dir`、tsf `product_log_path`、基础包
  目录 fixture `C:\Program Files\zhu-ye-ime\tsf`）。
- zip 内容变化（顶层目录名），本变更合入后需 `workflow_dispatch`（upload=true）
  重建 release 资产。
- 文档/site/active Agent Notes 就地更新；CHANGELOG 已发布段与历史记录保留
  旧名为当时事实。

## 备选方案

- **新旧名并存（兼容）**——被用户明确否决；删除旧资产使 release 与 SHA256SUMS
  自洽。
- **安装时迁移旧数据目录**——被否决：用户要求无向后兼容；迁移代码会让两套路径
  永久共存于代码库。
- **exe 一并改名**——不在范围：两轮指令分别点名 zip 名与目录，exe 名未动以避免
  未被要求的破坏性变更。

## 交叉引用

- [发行包布局与安装器](../process/2026-10-02-distribution-package-layout-and-installer.zh.md)
- [TSF 注册与生命周期](../architecture/2026-09-18-tsf-registration-and-lifetime.zh.md)
- [v0.1.2 正式发布执行记录与排障](../process/2026-10-05-release-v0.1.2.zh.md)
