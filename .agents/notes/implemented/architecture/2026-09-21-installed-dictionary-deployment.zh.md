# Agent Note: Installed dictionary deployment for TSF

Status: implemented

[English](2026-09-21-installed-dictionary-deployment.md) | 中文

## 问题

安装器此前只复制 TSF DLL，运行时只在 `%APPDATA%\zhu-ye-ime\seed.zyct` 找词典；新安装或打包到其他机器后，输入法回退到 20 词内存演示词典，真实词典管线没有进入虚拟机/真机验收，便携包说明也已过时。

## 决策

安装器与便携包现在把 v2 词典作为 `dictionary.zyct` 随 DLL 一起部署。来源解析顺序：显式 `-DictionaryPath` > 便携包暂存根目录 > `data/artifacts/real.zyct` > `data/artifacts/seed.zyct`；都没有时构建演示种子。

`install.ps1` 把选定词典复制到 DLL 同目录，产品数据机器级可卸载；`uninstall.ps1` 删除 `dictionary.zyct`、旧版 `seed.zyct` 与 DLL。`package-portable.ps1` 随包携带选定词典、安装/卸载脚本、DLL 与许可证/数据清单文档。

TSF 运行时按 `ZHU_YE_DICT_PATH` 环境变量 > DLL 同目录 > `%APPDATA%\zhu-ye-ime\dictionary.zyct` > 工作目录 `dictionary.zyct` 的顺序解析；文件缺失、损坏或校验失败时仍回退内置演示词典。

## 曾考虑的替代方案

**只保留用户级 `%APPDATA%` 副本。** 否决：机器级 TSF 安装需要为每个用户复制，且卸载无法在不触碰用户数据的情况下清理产品词典。

**把词典嵌入 DLL。** 否决：会膨胀二进制，让数据更新依赖整体重建，破坏数据与引擎分离。

**让测试人员手动放置词典。** 否决：对虚拟机测试脆弱，也违背“解压即用”的便携流程。

## 后果

测试机解压便携包安装后即可直接使用真实词库（已生成时）。包内含派生词典的来源与许可证说明；卸载清理产品数据但保留 `%APPDATA%\zhu-ye-ime\user_words.json`。环境变量保留开发调试出口，不改变已安装行为。
