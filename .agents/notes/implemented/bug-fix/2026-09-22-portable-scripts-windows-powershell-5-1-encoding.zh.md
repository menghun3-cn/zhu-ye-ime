# Agent Note: Portable package scripts compatible with Windows PowerShell 5.1

Status: implemented

[English](2026-09-22-portable-scripts-windows-powershell-5-1-encoding.md) | 中文

## 问题

便携包在 VM 验收时连 TSF 安装都没能开始：执行 `./scripts/install.ps1 -SkipBuild` 报 PowerShell 语法错误，中文消息显示为乱码。打包进便携包的 `.ps1` 是无 BOM 的 UTF-8；Windows 11 上的 Windows PowerShell 5.1 会按旧 ANSI 代码页解码，多字节中文注释与字符串字面量因此变成非法语法，字符串终止符也会丢失。本机 PowerShell 7 默认按 UTF-8 解码，所以一直没暴露这个问题。

## 决策

仓库内全部 PowerShell 脚本（`scripts/*.ps1` 与 `.agents/skills/git-publish/scripts/pre-publish-check.ps1`）现在都带 UTF-8 BOM。`scripts/package-portable.ps1` 在复制进便携包时用 `[System.Text.UTF8Encoding]::new($true)` 重新写出四个脚本，即使源文件未来丢失 BOM，新生成的便携包也保持兼容。`README-测试.txt` 同样改为 UTF-8 BOM 写出，Windows 工具打开测试说明不会再出现乱码。

重新打包完毕后做了替换前验证：解压出的脚本以 BOM 开头，`install.ps1` 可在 Windows PowerShell 5.1 下通过解析，并在需要管理员权限的错误路径中显示正确中文。

## 曾考虑的替代方案

**把脚本改成纯 ASCII，删除中文注释与提示。** 落选：项目规范要求中文注释与面向用户的提示；剥离这些内容会降低可维护性，而且只要再出现中文字符串，同样的坑就会回来。

**依赖 `chcp 65001` 或控制台字体设置。** 落选：解析发生在脚本正文执行之前，并由调用宿主决定；`chcp` 无法改变脚本文件本身被解码的方式。

**不加 BOM 并强制使用 PowerShell 7。** 落选：VM 和大量用户机器默认使用 Windows PowerShell 5.1，兼容性必须向可移植目标看齐。

## 后果

T-025 已完成：便携包内脚本可在 Windows PowerShell 5.1 下解析，Windows 11 VM 上的解析阻塞已被移除。除 BOM 字节外，既有脚本内容没有变化，未来 `package-portable.ps1` 的产物也会保持兼容。真实 TSF 注册仍需要在 VM 上以管理员权限执行；本次修复只清除了 `T-010`/`T-011` VM 验收前的编码障碍。
