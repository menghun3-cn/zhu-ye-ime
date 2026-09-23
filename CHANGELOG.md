# 变更记录

本项目版本号遵循语义化版本，变更记录遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) 中文格式。
当前唯一版本主源为根 `Cargo.toml` 的 `workspace.package.version`。

## [Unreleased]

## [0.1.0] - 2026-09-23

### 新增

- 基于 TSF 的 Windows 中文输入法 DLL 与安装/卸载闭环（T-010）
- 全拼按键、组合与上屏闭环（T-011），支持中英模式切换
- 标准全拼音节表与动态规划切分核心（T-007）
- v2 二进制词典格式、构建管线与 mmap 加载（T-006）
- CC-CEDICT 与 FrequencyWords 中文词频真实词库导入管线（T-006），清洗校验后生成 12 万级双语词典
- OPUS GlobalVoices 中文分词语料真实 bigram 统计与导入（T-006、T-008），真实词典含 820,368 个共现词对
- unigram + bigram 静态候选排序与用户词学习持久化（T-008、T-009）
- 候选窗 Win32 自绘，支持深浅色、DPI 与高对比度（T-012）
- 键位交互：Shift、Tab、翻页、数字选择与候选窗 TSF 联动（T-013）
- 本地双语翻译层与译文切换、英文反查（T-014）
- `zhu-ye-cli` 自检、候选演示、词典检查与性能基准（T-014、T-015）
- `zhu-ye-cli rank` 真实词典候选排序验证（T-008），候选生成统一下沉 zhu-ye-core
- 安装与便携包随带 v2 词典，TSF 运行时从 DLL 同目录加载（T-022）
- 性能基准与阈值验收脚本 `scripts/bench.ps1`（T-015）
- 主机侧端到端回归检查器与 `scripts/e2e.ps1` 验收入口，覆盖切分、排序、用户词、译文层与正反查（T-024）
- 便携测试包脚本 `scripts/package-portable.ps1`（T-010）
- Agent Notes 双语校验与归档脚本（T-018、T-020）

### 修复

- 修复 TSF 组合写入路径崩溃：改用只读查询加组合写入，规避 `ITfInsertAtSelection` 写入分支访问冲突（T-010、T-011）
- 修复候选窗翻页后因视图二次切片越界被误判为无候选而隐藏（T-013）
- 整词拼音存在直接词典条目时抑制多音节切分噪声候选（T-021）
- 便携包脚本适配 Windows PowerShell 5.1 的 UTF-8 BOM 输出（T-025）

### 变更

- 词典格式确定为 v2：128 字节头部、拼音索引、词条表、bigram、中文译文索引、英文反查索引与内容 SHA-256（T-014）
- `AiService.translate` 对齐方案设计，明确 `TranslationDirection` 参数；离线实现保持零网络空结果（T-023）
