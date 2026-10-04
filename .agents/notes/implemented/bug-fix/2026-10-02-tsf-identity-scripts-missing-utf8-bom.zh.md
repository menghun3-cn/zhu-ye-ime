# Agent Note: T-079 身份脚本交付时漏加 UTF-8 BOM

Status: implemented

[English](2026-10-02-tsf-identity-scripts-missing-utf8-bom.md) | 中文

## 问题

T-079 的门禁脚本 `scripts/verify-tsf-identity.ps1` 与其点源加载的
`scripts/ime-identity.ps1` 提交时没有带 UTF-8 BOM，而
`2026-09-22-portable-scripts-windows-powershell-5-1-encoding` 已把 BOM 确立为
PowerShell 5.1 兼容的约定。PowerShell 5.1 对无 BOM 文件按 ANSI 解码，两个文件
里的中文注释与字符串扰乱了解析器：门禁在 GUID 哈希字面量行报
"Unexpected token"，比对从未真正执行。

## 决策

两个脚本重新以带 BOM 的 UTF-8 保存（仅重新编码，内容不变）。门禁现在可在宿主
上执行并输出 6/6 项身份比对。2026-09-22 笔记的约定不变、适用于 `scripts/`
下每一份新脚本；T-079 的两个文件正是缺口——当时门禁自身无法执行其子检查。

## 曾考虑的替代方案

**把两个脚本重写成纯 ASCII、不含中文的形式。**
否决：要比对的内容包含中文标签，双语对是审计线索的一部分；问题在文件编码
而非内容。

## 后果

`scripts/verify-tsf-identity.ps1` 按设计在 PowerShell 5.1 上执行。需要该门禁
的 T-076 变更（设置窗口注册表修复写入相同常量）得以在本地验证；今后任何触碰
身份常量的 PR 都必须保持两个文件为 BOM 编码。
